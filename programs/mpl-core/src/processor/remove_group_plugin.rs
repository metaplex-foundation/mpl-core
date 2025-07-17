use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};

use crate::{
    error::MplCoreError,
    plugins::{delete_plugin, fetch_wrapped_plugin, PluginType},
    state::{DataBlob, GroupV1, SolanaAccount},
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

    if accounts.len() < 4 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let group_info = &accounts[0];
    let payer_info = &accounts[1];
    let authority_info_opt = accounts.get(2);
    let system_program_info = &accounts[3];

    if let Some(wrapper_info) = accounts.get(4) {
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
    let group_core = GroupV1::load(group_info, 0)?;
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
