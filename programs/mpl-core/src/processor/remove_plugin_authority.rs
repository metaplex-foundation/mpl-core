use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        RemoveCollectionPluginAuthorityAccounts, RemovePluginAuthorityAccounts,
    },
    plugins::{remove_authority_from_plugin, PluginHeader, PluginRegistry, PluginType},
    state::{Asset, Authority, CollectionData, SolanaAccount, UpdateAuthority},
    utils::fetch_core_data,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemovePluginAuthorityArgs {
    pub plugin_type: PluginType,
    pub authority_to_remove: Authority,
}

pub(crate) fn remove_plugin_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemovePluginAuthorityArgs,
) -> ProgramResult {
    let ctx = RemovePluginAuthorityAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = match ctx.accounts.payer {
        Some(payer) => {
            assert_signer(payer)?;
            payer
        }
        None => ctx.accounts.authority,
    };

    let (asset, plugin_header, mut plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

    //TODO: Make this better.
    let authority_type = if ctx.accounts.authority.key == &asset.owner {
        Authority::Owner
    } else if asset.update_authority == UpdateAuthority::Address(*ctx.accounts.authority.key) {
        Authority::UpdateAuthority
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
            None => return Err(MplCoreError::InvalidCollection.into()),
        }
    } else {
        Authority::Pubkey {
            address: *ctx.accounts.authority.key,
        }
    };

    process_remove_plugin_authority(
        ctx.accounts.asset,
        payer,
        ctx.accounts.system_program,
        &authority_type,
        &args.plugin_type,
        &args.authority_to_remove,
        plugin_header.as_ref(),
        plugin_registry.as_mut(),
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveCollectionPluginAuthorityArgs {
    pub plugin_type: PluginType,
    pub authority_to_remove: Authority,
}

pub(crate) fn remove_collection_plugin_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveCollectionPluginAuthorityArgs,
) -> ProgramResult {
    let ctx = RemoveCollectionPluginAuthorityAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = match ctx.accounts.payer {
        Some(payer) => {
            assert_signer(payer)?;
            payer
        }
        None => ctx.accounts.authority,
    };

    let (_, plugin_header, mut plugin_registry) =
        fetch_core_data::<CollectionData>(ctx.accounts.collection)?;

    process_remove_plugin_authority(
        ctx.accounts.collection,
        payer,
        ctx.accounts.system_program,
        &Authority::UpdateAuthority,
        &args.plugin_type,
        &args.authority_to_remove,
        plugin_header.as_ref(),
        plugin_registry.as_mut(),
    )
}

#[allow(clippy::too_many_arguments)]
fn process_remove_plugin_authority<'a>(
    core_info: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    authority_type: &Authority,
    plugin_type: &PluginType,
    authority_to_remove: &Authority,
    plugin_header: Option<&PluginHeader>,
    plugin_registry: Option<&mut PluginRegistry>,
) -> ProgramResult {
    let plugin_header = match plugin_header {
        Some(header) => header,
        None => return Err(MplCoreError::PluginsNotInitialized.into()),
    };

    let plugin_registry = match plugin_registry {
        Some(registry) => registry,
        None => return Err(MplCoreError::PluginsNotInitialized.into()),
    };

    remove_authority_from_plugin(
        plugin_type,
        authority_type,
        authority_to_remove,
        core_info,
        plugin_header,
        plugin_registry,
        payer,
        system_program,
    )?;

    Ok(())
}
