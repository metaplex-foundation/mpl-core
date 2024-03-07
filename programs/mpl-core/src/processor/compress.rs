use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::CompressAccounts,
    plugins::{CheckResult, Plugin, RegistryRecord, ValidationResult},
    state::{Asset, Compressible, HashablePluginSchema, HashedAsset, HashedAssetSchema, Key},
    utils::{fetch_core_data, load_key, resize_or_reallocate_account},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CompressArgs {}

pub(crate) fn compress<'a>(accounts: &'a [AccountInfo<'a>], args: CompressArgs) -> ProgramResult {
    // Accounts.
    let ctx = CompressAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.owner)?;
    let payer = if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
        payer
    } else {
        ctx.accounts.owner
    };

    match load_key(ctx.accounts.asset, 0)? {
        Key::Asset => {
            let (asset, _, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

            let mut approved = false;
            match Asset::check_compress() {
                CheckResult::CanApprove | CheckResult::CanReject => {
                    match asset.validate_compress(&ctx.accounts)? {
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

            if let Some(plugin_registry) = &plugin_registry {
                for record in &plugin_registry.registry {
                    if matches!(
                        record.plugin_type.check_compress(),
                        CheckResult::CanApprove | CheckResult::CanReject
                    ) {
                        let result = Plugin::validate_compress(
                            &Plugin::load(ctx.accounts.asset, record.offset)?,
                            ctx.accounts.owner,
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

            let mut plugin_hashes = vec![];
            if let Some(plugin_registry) = plugin_registry {
                let mut registry_records = plugin_registry.registry;

                // It should already be sorted but we just want to make sure.
                registry_records.sort_by(RegistryRecord::compare_offsets);

                for (i, record) in registry_records.into_iter().enumerate() {
                    let plugin = Plugin::deserialize(
                        &mut &(*ctx.accounts.asset.data).borrow()[record.offset..],
                    )?;

                    let hashable_plugin_schema = HashablePluginSchema {
                        index: i,
                        authorities: record.authorities,
                        plugin,
                    };

                    let plugin_hash = hashable_plugin_schema.hash()?;
                    plugin_hashes.push(plugin_hash);
                }
            }

            let asset_hash = asset.hash()?;
            let hashed_asset_schema = HashedAssetSchema {
                asset_hash,
                plugin_hashes,
            };

            let hashed_asset = HashedAsset::new(hashed_asset_schema.hash()?);
            let serialized_data = hashed_asset.try_to_vec()?;

            resize_or_reallocate_account(
                ctx.accounts.asset,
                payer,
                ctx.accounts.system_program,
                serialized_data.len(),
            )?;

            sol_memcpy(
                &mut ctx.accounts.asset.try_borrow_mut_data()?,
                &serialized_data,
                serialized_data.len(),
            );
        }
        Key::HashedAsset => return Err(MplCoreError::AlreadyCompressed.into()),
        _ => return Err(MplCoreError::IncorrectAccount.into()),
    }

    Ok(())
}
