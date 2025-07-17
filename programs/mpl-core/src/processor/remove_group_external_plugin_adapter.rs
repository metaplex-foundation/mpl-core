use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};

use crate::{
    error::MplCoreError,
    plugins::{
        delete_external_plugin_adapter, fetch_wrapped_external_plugin_adapter,
        ExternalPluginAdapterKey,
    },
    state::{DataBlob, GroupV1, SolanaAccount},
    utils::{fetch_core_data, is_valid_group_authority, resolve_authority},
};

/// Arguments for `RemoveGroupExternalPluginAdapterV1`.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveGroupExternalPluginAdapterV1Args {
    /// External plugin adapter key to remove.
    pub key: ExternalPluginAdapterKey,
}

pub(crate) fn remove_group_external_plugin_adapter<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveGroupExternalPluginAdapterV1Args,
) -> ProgramResult {
    // Expected accounts:
    // 0. [writable] Group account
    // 1. [writable, signer] Payer account
    // 2. [signer] Optional authority (update authority or delegate)
    // 3. [] System program
    // 4. [] Optional SPL Noop (log wrapper)

    if accounts.len() < 4 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let group_info = &accounts[0];
    let payer_info = &accounts[1];
    let authority_info_opt = accounts.get(2);
    let system_program_info = &accounts[3];
    let log_wrapper_info_opt = accounts.get(4);

    // Guards.
    assert_signer(payer_info)?;
    let authority_info = resolve_authority(payer_info, authority_info_opt)?;

    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }
    if let Some(wrapper_info) = log_wrapper_info_opt {
        if wrapper_info.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    // Fetch core + meta and check authority/delegate.
    let (group_core, plugin_header_opt, plugin_registry_opt) =
        fetch_core_data::<GroupV1>(group_info)?;

    if !is_valid_group_authority(group_info, authority_info)? {
        msg!("Error: Invalid authority for group account");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Nothing to delete if no plugin meta exists.
    if plugin_header_opt.is_none() || plugin_registry_opt.is_none() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    // Ensure the plugin exists.
    let _ =
        fetch_wrapped_external_plugin_adapter::<GroupV1>(group_info, Some(&group_core), &args.key)?;

    // Delete plugin.
    delete_external_plugin_adapter(
        &args.key,
        &group_core,
        group_info,
        payer_info,
        system_program_info,
    )
}
