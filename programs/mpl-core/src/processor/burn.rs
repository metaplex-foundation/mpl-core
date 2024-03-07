use std::collections::BTreeMap;

use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::{BurnAccounts, BurnCollectionAccounts},
    plugins::{
        validate_plugin_checks, CheckResult, Plugin, PluginType, RegistryRecord, ValidationResult,
    },
    state::{Asset, Collection, Compressible, CompressionProof, Key, SolanaAccount},
    utils::{close_program_account, fetch_core_data, load_key, verify_proof},
};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct BurnArgs {
    compression_proof: Option<CompressionProof>,
}

pub(crate) fn burn<'a>(accounts: &'a [AccountInfo<'a>], args: BurnArgs) -> ProgramResult {
    // Accounts.
    let ctx = BurnAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
    }

    match load_key(ctx.accounts.asset, 0)? {
        Key::HashedAsset => {
            let compression_proof = args
                .compression_proof
                .ok_or(MplCoreError::MissingCompressionProof)?;
            let (asset, _) = verify_proof(ctx.accounts.asset, &compression_proof)?;

            if ctx.accounts.authority.key != &asset.owner {
                return Err(MplCoreError::InvalidAuthority.into());
            }

            // TODO: Check delegates in compressed case.

            asset.wrap()?;
        }
        Key::Asset => {
            let (_, _, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

            let mut checks: BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)> =
                BTreeMap::new();

            // The asset approval overrides the collection approval.
            let asset_approval = Asset::check_burn();
            let core_check = match asset_approval {
                CheckResult::None => (Key::Collection, Collection::check_burn()),
                _ => (Key::Asset, asset_approval),
            };

            // Check the collection plugins first.
            ctx.accounts.collection.and_then(|collection_info| {
                fetch_core_data::<Collection>(collection_info)
                    .map(|(_, _, registry)| {
                        registry.map(|r| {
                            r.check_registry(Key::Collection, PluginType::check_burn, &mut checks);
                            r
                        })
                    })
                    .ok()?
            });

            // Next check the asset plugins. Plugins on the asset override the collection plugins,
            // so we don't need to validate the collection plugins if the asset has a plugin.
            if let Some(registry) = plugin_registry.as_ref() {
                registry.check_registry(Key::Asset, PluginType::check_burn, &mut checks);
            }

            solana_program::msg!("checks: {:#?}", checks);

            // Do the core validation.
            let mut approved = matches!(
                core_check,
                (
                    Key::Asset | Key::Collection,
                    CheckResult::CanApprove | CheckResult::CanReject
                )
            ) && {
                (match core_check.0 {
                    Key::Collection => Collection::load(
                        ctx.accounts
                            .collection
                            .ok_or(MplCoreError::InvalidCollection)?,
                        0,
                    )?
                    .validate_burn(ctx.accounts.authority)?,
                    Key::Asset => {
                        Asset::load(ctx.accounts.asset, 0)?.validate_burn(ctx.accounts.authority)?
                    }
                    _ => return Err(MplCoreError::IncorrectAccount.into()),
                }) == ValidationResult::Approved
            };

            approved = validate_plugin_checks(
                Key::Collection,
                &checks,
                ctx.accounts.authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                Box::new(Plugin::validate_burn),
            )? || approved;

            approved = validate_plugin_checks(
                Key::Asset,
                &checks,
                ctx.accounts.authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                Box::new(Plugin::validate_burn),
            )? || approved;

            if !approved {
                return Err(MplCoreError::InvalidAuthority.into());
            }
        }
        _ => return Err(MplCoreError::IncorrectAccount.into()),
    }

    process_burn(ctx.accounts.asset, ctx.accounts.authority)
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct BurnCollectionArgs {
    compression_proof: Option<CompressionProof>,
}

pub(crate) fn burn_collection<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: BurnCollectionArgs,
) -> ProgramResult {
    // Accounts.
    let ctx = BurnCollectionAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
    }

    let (collection, _, plugin_registry) = fetch_core_data::<Collection>(ctx.accounts.collection)?;

    // let checks: [(Key, CheckResult); PluginType::COUNT + 2];

    let mut approved = false;
    match Collection::check_burn() {
        CheckResult::CanApprove | CheckResult::CanReject => {
            match collection.validate_burn(ctx.accounts.authority)? {
                ValidationResult::Approved => {
                    approved = true;
                }
                ValidationResult::Rejected => return Err(MplCoreError::InvalidAuthority.into()),
                ValidationResult::Pass => (),
            }
        }
        CheckResult::None => (),
    };

    if let Some(plugin_registry) = plugin_registry {
        for record in plugin_registry.registry {
            if matches!(
                PluginType::check_burn(&record.plugin_type),
                CheckResult::CanApprove | CheckResult::CanReject
            ) {
                let result = Plugin::validate_burn(
                    &Plugin::load(ctx.accounts.collection, record.offset)?,
                    ctx.accounts.authority,
                    &record.authorities,
                )?;
                if result == ValidationResult::Rejected {
                    return Err(MplCoreError::InvalidAuthority.into());
                } else if result == ValidationResult::Approved {
                    approved = true;
                }
            }
        }
    };

    if !approved {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    process_burn(ctx.accounts.collection, ctx.accounts.authority)
}

fn process_burn<'a>(core_info: &AccountInfo<'a>, authority: &AccountInfo<'a>) -> ProgramResult {
    close_program_account(core_info, authority)
}
