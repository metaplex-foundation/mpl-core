use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::{RemoveCollectionPluginAccounts, RemovePluginAccounts},
    plugins::{delete_plugin, PluginType},
    state::{Asset, Authority, CollectionData, SolanaAccount, UpdateAuthority},
    utils::fetch_core_data,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct RemovePluginArgs {
    plugin_type: PluginType,
}

pub(crate) fn remove_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemovePluginArgs,
) -> ProgramResult {
    let ctx = RemovePluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = match ctx.accounts.payer {
        Some(payer) => {
            assert_signer(payer)?;
            payer
        }
        None => ctx.accounts.authority,
    };

    let (asset, plugin_header, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

    // We don't have anything to delete if there's no plugin meta.
    if plugin_header.is_none() || plugin_registry.is_none() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    //TODO: Make this better.
    let authority_type = if ctx.accounts.authority.key == &asset.owner {
        Authority::Owner
    } else if let UpdateAuthority::Address(update_authority) = asset.update_authority {
        if ctx.accounts.authority.key == &update_authority {
            Authority::UpdateAuthority
        } else {
            return Err(MplCoreError::InvalidAuthority.into());
        }
    } else if let UpdateAuthority::Collection(collection_address) = asset.update_authority {
        match ctx.accounts.collection {
            Some(collection_info) => {
                if collection_info.key != &collection_address {
                    return Err(MplCoreError::InvalidCollection.into());
                }
                let collection = CollectionData::load(collection_info, 0)?;
                if ctx.accounts.authority.key == &collection.update_authority {
                    Authority::UpdateAuthority
                } else {
                    return Err(MplCoreError::InvalidAuthority.into());
                }
            }
            None => return Err(MplCoreError::InvalidAuthority.into()),
        }
    } else {
        return Err(MplCoreError::InvalidAuthority.into());
    };

    delete_plugin(
        &args.plugin_type,
        &asset,
        &authority_type,
        ctx.accounts.asset,
        payer,
        ctx.accounts.system_program,
    )?;

    process_remove_plugin()
}

pub(crate) fn remove_collection_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemovePluginArgs,
) -> ProgramResult {
    let ctx = RemoveCollectionPluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = match ctx.accounts.payer {
        Some(payer) => {
            assert_signer(payer)?;
            payer
        }
        None => ctx.accounts.authority,
    };

    let (collection, plugin_header, plugin_registry) =
        fetch_core_data::<CollectionData>(ctx.accounts.collection)?;

    // We don't have anything to delete if there's no plugin meta.
    if plugin_header.is_none() || plugin_registry.is_none() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    if ctx.accounts.authority.key != &collection.update_authority {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    delete_plugin(
        &args.plugin_type,
        &collection,
        &Authority::UpdateAuthority,
        ctx.accounts.collection,
        payer,
        ctx.accounts.system_program,
    )?;

    process_remove_plugin()
}

//TODO
fn process_remove_plugin() -> ProgramResult {
    Ok(())
}
