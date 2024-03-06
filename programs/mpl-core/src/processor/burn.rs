use std::collections::BTreeMap;

use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::{BurnAccounts, BurnCollectionAccounts},
    plugins::{CheckResult, Plugin, PluginType, RegistryRecord, ValidationResult},
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
            let core_check = if asset_approval != CheckResult::None {
                (Key::Asset, asset_approval)
            } else {
                (Key::Collection, Collection::check_burn())
            };

            // Check the collection plugins first.
            let collection_plugin_registry = if let Some(collection_info) = ctx.accounts.collection
            {
                let (_, _, collection_plugin_registry) =
                    fetch_core_data::<Collection>(collection_info)?;

                if let Some(collection_plugin_registry) = collection_plugin_registry {
                    collection_plugin_registry.check_burn(Key::Collection, &mut checks);

                    Some(collection_plugin_registry)
                } else {
                    None
                }
            } else {
                None
            };

            // Next check the asset plugins. Plugins on the asset override the collection plugins,
            // so we don't need to validate the collection plugins if the asset has a plugin.
            if let Some(plugin_registry) = &plugin_registry {
                plugin_registry.check_burn(Key::Asset, &mut checks);
            };

            solana_program::msg!("checks: {:#?}", checks);

            let mut approved = false;

            // Do the core check.
            match core_check.0 {
                Key::Collection => match core_check.1 {
                    CheckResult::CanApprove | CheckResult::CanReject => match Collection::load(
                        ctx.accounts
                            .collection
                            .ok_or(MplCoreError::InvalidCollection)?,
                        0,
                    )?
                    .validate_burn(ctx.accounts.authority)?
                    {
                        ValidationResult::Approved => {
                            approved = true;
                        }
                        ValidationResult::Rejected => {
                            return Err(MplCoreError::InvalidAuthority.into())
                        }
                        ValidationResult::Pass => (),
                    },
                    CheckResult::None => (),
                },
                Key::Asset => match core_check.1 {
                    CheckResult::CanApprove | CheckResult::CanReject => {
                        match Asset::load(ctx.accounts.asset, 0)?
                            .validate_burn(ctx.accounts.authority)?
                        {
                            ValidationResult::Approved => {
                                approved = true;
                            }
                            ValidationResult::Rejected => {
                                return Err(MplCoreError::InvalidAuthority.into())
                            }
                            ValidationResult::Pass => (),
                        }
                    }
                    CheckResult::None => (),
                },
                _ => return Err(MplCoreError::IncorrectAccount.into()),
            }

            if let Some(collection_plugin_registry) = collection_plugin_registry {
                for check in &checks {
                    if check.1 .0 == Key::Collection
                        && matches!(check.1 .1, CheckResult::CanApprove | CheckResult::CanReject)
                    {
                        let result = Plugin::load(
                            ctx.accounts
                                .collection
                                .ok_or(MplCoreError::InvalidCollection)?,
                            check.1 .2.offset,
                        )?
                        .validate_burn(ctx.accounts.authority, &check.1 .2.authorities)?;
                        if result == ValidationResult::Rejected {
                            return Err(MplCoreError::InvalidAuthority.into());
                        } else if result == ValidationResult::Approved {
                            approved = true;
                        }
                    }
                }
            };

            if let Some(plugin_registry) = &plugin_registry {
                for check in &checks {
                    if check.1 .0 == Key::Asset
                        && matches!(check.1 .1, CheckResult::CanApprove | CheckResult::CanReject)
                    {
                        let result = Plugin::load(ctx.accounts.asset, check.1 .2.offset)?
                            .validate_burn(ctx.accounts.authority, &check.1 .2.authorities)?;
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
                record.plugin_type.check_transfer(),
                CheckResult::CanApprove | CheckResult::CanReject
            ) {
                let result = Plugin::load(ctx.accounts.collection, record.offset)?
                    .validate_burn(ctx.accounts.authority, &record.authorities)?;
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
