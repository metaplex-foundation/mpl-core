use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::RemovePluginAccounts,
    plugins::{delete_plugin, PluginType},
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

    let (asset, plugin_header, plugin_registry) = fetch_core_data(ctx.accounts.asset_address)?;

    // We don't have anything to delete if there's no plugin meta.
    if plugin_header.is_none() || plugin_registry.is_none() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    delete_plugin(
        &args.plugin_type,
        &asset,
        ctx.accounts.authority,
        ctx.accounts.asset_address,
        payer,
        ctx.accounts.system_program,
    )?;

    Ok(())
}
