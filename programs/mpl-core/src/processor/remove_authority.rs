use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplAssetError,
    instruction::accounts::RemoveAuthorityAccounts,
    plugins::{remove_authority_from_plugin, PluginType},
    state::Authority,
    utils::fetch_core_data,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct RemoveAuthorityArgs {
    pub plugin_type: PluginType,
    pub authority_to_remove: Authority,
}

pub(crate) fn remove_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveAuthorityArgs,
) -> ProgramResult {
    let ctx = RemoveAuthorityAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = match ctx.accounts.payer {
        Some(payer) => {
            assert_signer(payer)?;
            payer
        }
        None => ctx.accounts.authority,
    };

    let (asset, plugin_header, plugin_registry) = fetch_core_data(ctx.accounts.asset_address)?;

    let plugin_header = match plugin_header {
        Some(header) => header,
        None => return Err(MplAssetError::PluginsNotInitialized.into()),
    };

    let mut plugin_registry = match plugin_registry {
        Some(registry) => registry,
        None => return Err(MplAssetError::PluginsNotInitialized.into()),
    };

    remove_authority_from_plugin(
        &args.plugin_type,
        ctx.accounts.authority,
        &args.authority_to_remove,
        &asset,
        ctx.accounts.asset_address,
        &plugin_header,
        &mut plugin_registry,
        payer,
        ctx.accounts.system_program,
    )?;

    Ok(())
}
