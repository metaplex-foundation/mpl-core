use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::{BurnAccounts, BurnCollectionAccounts},
    plugins::{CheckResult, Plugin, ValidationResult},
    state::{Asset, CollectionData, Compressible, CompressionProof, Key},
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
            let (asset, _, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

            let mut approved = false;
            match Asset::check_transfer() {
                CheckResult::CanApprove | CheckResult::CanReject => {
                    match asset.validate_burn(&ctx.accounts)? {
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
            };

            if let Some(plugin_registry) = plugin_registry {
                for record in plugin_registry.registry {
                    if matches!(
                        record.plugin_type.check_transfer(),
                        CheckResult::CanApprove | CheckResult::CanReject
                    ) {
                        let result = Plugin::load(ctx.accounts.asset, record.offset)?
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

    let (collection, _, plugin_registry) =
        fetch_core_data::<CollectionData>(ctx.accounts.collection)?;

    let mut approved = false;
    match CollectionData::check_burn() {
        CheckResult::CanApprove | CheckResult::CanReject => {
            match collection.validate_burn(&ctx.accounts)? {
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
