use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::{AddCollectionPluginAuthorityAccounts, AddPluginAuthorityAccounts},
    plugins::{add_authority_to_plugin, PluginType},
    state::{Asset, Authority, Collection, CoreAsset, DataBlob, SolanaAccount},
    utils::fetch_core_data,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddPluginAuthorityArgs {
    pub plugin_type: PluginType,
    pub new_authority: Authority,
}

pub(crate) fn add_plugin_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddPluginAuthorityArgs,
) -> ProgramResult {
    let ctx = AddPluginAuthorityAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = match ctx.accounts.payer {
        Some(payer) => {
            assert_signer(payer)?;
            payer
        }
        None => ctx.accounts.authority,
    };

    process_add_plugin_authority::<Asset>(
        ctx.accounts.asset,
        ctx.accounts.authority,
        payer,
        ctx.accounts.system_program,
        &args.plugin_type,
        &args.new_authority,
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddCollectionPluginAuthorityArgs {
    pub plugin_type: PluginType,
    pub new_authority: Authority,
}

pub(crate) fn add_collection_plugin_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddCollectionPluginAuthorityArgs,
) -> ProgramResult {
    let ctx = AddCollectionPluginAuthorityAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = match ctx.accounts.payer {
        Some(payer) => {
            assert_signer(payer)?;
            payer
        }
        None => ctx.accounts.authority,
    };

    process_add_plugin_authority::<Collection>(
        ctx.accounts.collection,
        ctx.accounts.authority,
        payer,
        ctx.accounts.system_program,
        &args.plugin_type,
        &args.new_authority,
    )
}

fn process_add_plugin_authority<'a, T: CoreAsset + DataBlob + SolanaAccount>(
    core_info: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    plugin_type: &PluginType,
    new_authority: &Authority,
) -> ProgramResult {
    let (core, plugin_header, plugin_registry) = fetch_core_data::<T>(core_info)?;

    let plugin_header = match plugin_header {
        Some(header) => header,
        None => return Err(MplCoreError::PluginsNotInitialized.into()),
    };

    let mut plugin_registry = match plugin_registry {
        Some(registry) => registry,
        None => return Err(MplCoreError::PluginsNotInitialized.into()),
    };

    add_authority_to_plugin(
        plugin_type,
        authority,
        new_authority,
        &core,
        core_info,
        &plugin_header,
        &mut plugin_registry,
        payer,
        system_program,
    )?;

    Ok(())
}
