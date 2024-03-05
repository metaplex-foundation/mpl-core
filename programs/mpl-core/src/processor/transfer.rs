use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::TransferAccounts,
    plugins::{CheckResult, Plugin, ValidationResult},
    state::{Asset, Compressible, CompressionProof, HashedAsset, Key, SolanaAccount},
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
            // let mut asset = Asset::load(ctx.accounts.asset, 0)?;

            // let mut authority_check: Result<(), ProgramError> =
            //     Err(MplCoreError::InvalidAuthority.into());
            // if asset.get_size() != ctx.accounts.asset.data_len() {
            //     solana_program::msg!("Fetch Plugin");
            //     let (authorities, plugin, _) =
            //         fetch_plugin(ctx.accounts.asset, PluginType::Delegate)?;

            //     solana_program::msg!("Assert authority");
            //     authority_check = assert_authority(&asset, ctx.accounts.authority, &authorities);

            //     if let Plugin::Delegate(delegate) = plugin {
            //         if delegate.frozen {
            //             return Err(MplCoreError::AssetIsFrozen.into());
            //         }
            //     }
            // }

            // match authority_check {
            //     Ok(_) => Ok::<(), ProgramError>(()),
            //     Err(_) => {
            //         if ctx.accounts.authority.key != &asset.owner {
            //             Err(MplCoreError::InvalidAuthority.into())
            //         } else {
            //             Ok(())
            //         }
            //     }
            // }?;

            let (mut asset, _, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

            let mut approved = false;
            match Asset::check_transfer() {
                CheckResult::CanApprove | CheckResult::CanReject => {
                    match asset.validate_transfer(&ctx.accounts)? {
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
                            .validate_transfer(
                                ctx.accounts.authority,
                                ctx.accounts.new_owner,
                                &args,
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

            asset.owner = *ctx.accounts.new_owner.key;
            asset.save(ctx.accounts.asset, 0)
        }
        _ => Err(MplCoreError::IncorrectAccount.into()),
    }
}
