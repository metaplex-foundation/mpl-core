use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use mpl_utils::resize_or_reallocate_account_raw;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::CompressAccounts,
    plugins::{fetch_plugins, Plugin, RegistryRecord},
    state::{
        Asset, Compressible, DataBlob, HashablePluginSchema, HashedAsset, HashedAssetSchema, Key,
        SolanaAccount,
    },
    utils::load_key,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CompressArgs {}

pub(crate) fn compress<'a>(accounts: &'a [AccountInfo<'a>], _args: CompressArgs) -> ProgramResult {
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

    match load_key(ctx.accounts.asset_address, 0)? {
        Key::Asset => {
            let asset = Asset::load(ctx.accounts.asset_address, 0)?;

            if ctx.accounts.owner.key != &asset.owner {
                return Err(MplCoreError::InvalidAuthority.into());
            }

            // TODO: Delegated compress/decompress authority.

            let mut plugin_hashes = vec![];
            if asset.get_size() != ctx.accounts.asset_address.data_len() {
                let mut registry_records = fetch_plugins(ctx.accounts.asset_address)?;

                // It should already be sorted but we just want to make sure.
                registry_records.sort_by(RegistryRecord::compare_offsets);

                for (i, record) in registry_records.into_iter().enumerate() {
                    let plugin = Plugin::deserialize(
                        &mut &(*ctx.accounts.asset_address.data).borrow()[record.offset..],
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

            resize_or_reallocate_account_raw(
                ctx.accounts.asset_address,
                payer,
                ctx.accounts.system_program,
                serialized_data.len(),
            )?;

            sol_memcpy(
                &mut ctx.accounts.asset_address.try_borrow_mut_data()?,
                &serialized_data,
                serialized_data.len(),
            );
        }
        Key::HashedAsset => return Err(MplCoreError::AlreadyCompressed.into()),
        _ => return Err(MplCoreError::IncorrectAccount.into()),
    }

    Ok(())
}
