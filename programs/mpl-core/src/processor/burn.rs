use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::BurnAccounts,
    plugins::{CheckResult, Plugin, ValidationResult},
    state::{Asset, Compressible, CompressionProof, Key},
    utils::{close_program_account, fetch_core_data, load_key},
};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct BurnArgs {
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

    match load_key(ctx.accounts.asset_address, 0)? {
        Key::HashedAsset => {
            let compression_proof = args
                .compression_proof
                .ok_or(MplCoreError::MissingCompressionProof)?;
            let asset = Asset::verify_proof(ctx.accounts.asset_address, compression_proof)?;

            if ctx.accounts.authority.key != &asset.owner {
                return Err(MplCoreError::InvalidAuthority.into());
            }

            // TODO: Check delegates in compressed case.

            asset.wrap()?;
        }
        Key::Asset => {
            let (asset, _, plugin_registry) = fetch_core_data(ctx.accounts.asset_address)?;

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
                        let result = Plugin::load(ctx.accounts.asset_address, record.offset)?
                            .validate_burn(&ctx.accounts, &args, &record.authorities)?;
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

    close_program_account(ctx.accounts.asset_address, ctx.accounts.authority)
}
