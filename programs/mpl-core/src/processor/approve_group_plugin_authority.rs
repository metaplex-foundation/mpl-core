use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};

use crate::{
    error::MplCoreError,
    plugins::{approve_authority_on_plugin, fetch_wrapped_plugin, PluginType},
    state::{Authority, GroupV1, SolanaAccount},
    utils::{fetch_core_data, is_valid_group_authority, resolve_authority},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct ApproveGroupPluginAuthorityV1Args {
    pub plugin_type: PluginType,
    pub new_authority: Authority,
}

pub(crate) fn approve_group_plugin_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: ApproveGroupPluginAuthorityV1Args,
) -> ProgramResult {
    // Accounts:
    // 0. [writable] Group
    // 1. [writable, signer] Payer
    // 2. [signer] Optional authority
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

    // Authority check (update authority or delegate)
    let group = GroupV1::load(group_info, 0)?;
    if !is_valid_group_authority(group_info, authority_info)? {
        msg!("Error: Invalid authority for group account");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Ensure plugin exists
    let (_current_authority, _plugin) =
        fetch_wrapped_plugin::<GroupV1>(group_info, None, args.plugin_type)?;

    // Fetch plugin meta
    let (_core_stub, plugin_header_opt, plugin_registry_opt) =
        fetch_core_data::<GroupV1>(group_info)?;
    let plugin_header = plugin_header_opt.ok_or(MplCoreError::PluginsNotInitialized)?;
    let mut plugin_registry = plugin_registry_opt.ok_or(MplCoreError::PluginsNotInitialized)?;

    approve_authority_on_plugin::<GroupV1>(
        &args.plugin_type,
        &args.new_authority,
        group_info,
        &plugin_header,
        &mut plugin_registry,
        payer_info,
        system_program_info,
    )
}
