use std::collections::BTreeMap;

use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::TransferAccounts,
    plugins::{
        validate_plugin_checks, CheckResult, Plugin, PluginType, RegistryRecord, ValidationResult,
    },
    state::{
        Asset, Authority, Collection, Compressible, CompressionProof, HashedAsset, Key,
        SolanaAccount,
    },
    utils::{fetch_core_data, load_key, verify_proof},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct TransferArgs {
    compression_proof: Option<CompressionProof>,
}

pub(crate) fn transfer<'a>(accounts: &'a [AccountInfo<'a>], args: TransferArgs) -> ProgramResult {
    // Accounts.
    let ctx = TransferAccounts::context(accounts)?;

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
            let (mut asset, _) = verify_proof(ctx.accounts.asset, &compression_proof)?;

            if ctx.accounts.authority.key != &asset.owner {
                return Err(MplCoreError::InvalidAuthority.into());
            }

            // TODO: Check delegates in compressed case.

            asset.owner = *ctx.accounts.new_owner.key;

            asset.wrap()?;

            // Make a new hashed asset with updated owner and save to account.
            HashedAsset::new(asset.hash()?).save(ctx.accounts.asset, 0)
        }
        Key::Asset => {
            let (mut asset, _, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

            let mut checks: BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)> =
                BTreeMap::new();

            // The asset approval overrides the collection approval.
            let asset_approval = Asset::check_transfer();
            let core_check = match asset_approval {
                CheckResult::None => (Key::Collection, Collection::check_transfer()),
                _ => (Key::Asset, asset_approval),
            };

            // Check the collection plugins first.
            if let Some(collection_info) = ctx.accounts.collection {
                fetch_core_data::<Collection>(collection_info).map(|(_, _, registry)| {
                    registry.map(|r| {
                        r.check_transfer(Key::Collection, &mut checks);
                        r
                    })
                })?;
            }

            // Next check the asset plugins. Plugins on the asset override the collection plugins,
            // so we don't need to validate the collection plugins if the asset has a plugin.
            if let Some(registry) = plugin_registry.as_ref() {
                registry.check_transfer(Key::Asset, &mut checks);
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
                    .validate_transfer()?,
                    Key::Asset => Asset::load(ctx.accounts.asset, 0)?
                        .validate_transfer(ctx.accounts.authority)?,
                    _ => return Err(MplCoreError::IncorrectAccount.into()),
                }) == ValidationResult::Approved
            };
            solana_program::msg!("approved: {:#?}", approved);

            let custom_args = |plugin: &Plugin,
                               authority: &AccountInfo<'a>,
                               authorities: &[Authority]| {
                Plugin::validate_transfer(plugin, authority, ctx.accounts.new_owner, authorities)
            };

            approved = validate_plugin_checks(
                Key::Collection,
                &checks,
                ctx.accounts.authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                Box::new(custom_args),
            )? || approved;

            approved = validate_plugin_checks(
                Key::Asset,
                &checks,
                ctx.accounts.authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                Box::new(custom_args),
            )? || approved;

            if !approved {
                return Err(MplCoreError::InvalidAuthority.into());
            }
            asset.owner = *ctx.accounts.new_owner.key;
            asset.save(ctx.accounts.asset, 0)
        }
        _ => Err(MplCoreError::IncorrectAccount.into()),
    }
}
