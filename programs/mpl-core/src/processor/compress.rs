use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::CompressAccounts,
    plugins::{Plugin, PluginType, RegistryRecord},
    state::{
        Asset, Collection, Compressible, HashablePluginSchema, HashedAsset, HashedAssetSchema, Key,
    },
    utils::{fetch_core_data, load_key, resize_or_reallocate_account, validate_asset_permissions},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CompressArgs {}

pub(crate) fn compress<'a>(accounts: &'a [AccountInfo<'a>], _args: CompressArgs) -> ProgramResult {
    // Accounts.
    let ctx = CompressAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
        payer
    } else {
        ctx.accounts.authority
    };

    match load_key(ctx.accounts.asset, 0)? {
        Key::Asset => {
            let (asset, _, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

            let _ = validate_asset_permissions(
                ctx.accounts.authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                None,
                Asset::check_compress,
                Collection::check_compress,
                PluginType::check_compress,
                Asset::validate_compress,
                Collection::validate_compress,
                Plugin::validate_compress,
            )?;

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
                        authority: record.authority,
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
