use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::RemoveGroupPluginV1Accounts,
    plugins::{delete_plugin, fetch_wrapped_plugin, PluginType},
    state::GroupV1,
    utils::{fetch_core_data, is_valid_group_authority, resolve_authority},
};

/// Arguments for `RemoveGroupPluginV1`.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveGroupPluginV1Args {
    pub(crate) plugin_type: PluginType,
}

pub(crate) fn remove_group_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveGroupPluginV1Args,
) -> ProgramResult {
    // Expected accounts:
    // 0. [writable] Group account
    // 1. [writable, signer] Payer
    // 2. [signer] Optional authority (update authority or delegate)
    // 3. [] System program
    // 4. [] Optional log wrapper
    let ctx = RemoveGroupPluginV1Accounts::context(accounts)?;
    let group_info = ctx.accounts.group;
    let payer_info = ctx.accounts.payer;
    let authority_info_opt = ctx.accounts.authority;
    let system_program_info = ctx.accounts.system_program;

    if let Some(wrapper_info) = ctx.accounts.log_wrapper {
        if wrapper_info.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    assert_signer(payer_info)?;
    let authority_info = resolve_authority(payer_info, authority_info_opt)?;

    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    // Load group and permission check
    if !is_valid_group_authority(group_info, authority_info)? {
        msg!("Error: Invalid authority for group account");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Ensure plugins initialized
    let (group_core, plugin_header_opt, plugin_registry_opt) =
        fetch_core_data::<GroupV1>(group_info)?;

    if plugin_header_opt.is_none() || plugin_registry_opt.is_none() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    // Fetch the plugin to make sure it exists (also returns its authority, unused here)
    let _ = fetch_wrapped_plugin::<GroupV1>(group_info, Some(&group_core), args.plugin_type)?;

    // Remove plugin
    delete_plugin::<GroupV1>(
        &args.plugin_type,
        &group_core,
        group_info,
        payer_info,
        system_program_info,
    )
}
