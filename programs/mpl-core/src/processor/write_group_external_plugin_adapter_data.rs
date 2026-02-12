use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};

use crate::{
    error::MplCoreError,
    plugins::{
        create_meta_idempotent, fetch_wrapped_external_plugin_adapter,
        initialize_external_plugin_adapter, update_external_plugin_adapter_data, AppData,
        DataSectionInitInfo, ExternalPluginAdapter, ExternalPluginAdapterInitInfo,
        ExternalPluginAdapterKey, ExternalRegistryRecord, LinkedDataKey, PluginHeaderV1,
        PluginRegistryV1,
    },
    state::{Authority, DataBlob, GroupV1, SolanaAccount},
    utils::{fetch_core_data, is_valid_group_authority, resolve_authority},
};

/// Arguments for `WriteGroupExternalPluginAdapterDataV1`.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct WriteGroupExternalPluginAdapterDataV1Args {
    /// External plugin adapter key (must be `AppData`).
    pub key: ExternalPluginAdapterKey,
    /// Data to write.  If `None`, data will be sourced from the `buffer` account.
    pub data: Option<Vec<u8>>,
}

pub(crate) fn write_group_external_plugin_adapter_data<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: WriteGroupExternalPluginAdapterDataV1Args,
) -> ProgramResult {
    // Expected accounts:
    // 0. [writable] Group account
    // 1. [writable, signer] Payer
    // 2. [signer] Optional authority (data authority or update authority)
    // 3. [optional] Buffer account containing data
    // 4. [] System program
    // 5. [] Optional SPL Noop

    if accounts.len() < 5 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let group_info = &accounts[0];
    let payer_info = &accounts[1];
    let authority_info_opt = accounts.get(2);
    let buffer_info_opt = accounts.get(3);
    let system_program_info = &accounts[4];
    let log_wrapper_info_opt = accounts.get(5);

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

    // Only AppData plugins are supported for Groups.
    match args.key {
        ExternalPluginAdapterKey::AppData(_) => {}
        _ => return Err(MplCoreError::UnsupportedOperation.into()),
    }

    // Fetch core data for the group account.
    let (group_core, mut header_opt, mut registry_opt) = fetch_core_data::<GroupV1>(group_info)?;

    // Fetch plugin record & check existence.
    let (record, plugin) =
        fetch_wrapped_external_plugin_adapter::<GroupV1>(group_info, Some(&group_core), &args.key)?;

    // Copy plugin's data authority locally to avoid holding a borrow on `plugin`.
    let data_authority = match &plugin {
        ExternalPluginAdapter::AppData(AppData { data_authority, .. }) => *data_authority,
        _ => return Err(MplCoreError::UnsupportedOperation.into()),
    };

    // Verify authority (update authority or delegate).
    if !is_valid_group_authority(group_info, authority_info)? {
        let signer_key = *authority_info.key;

        let authorized = if let Authority::Address {
            address: plugin_key,
        } = data_authority
        {
            plugin_key == signer_key
        } else {
            false
        };

        if !authorized {
            msg!("Error: Invalid authority for group account or plugin data authority");
            return Err(MplCoreError::InvalidAuthority.into());
        }
    }

    // Ensure plugin meta exists.
    let header = header_opt
        .as_mut()
        .ok_or(MplCoreError::PluginsNotInitialized)?;
    let registry = registry_opt
        .as_mut()
        .ok_or(MplCoreError::PluginsNotInitialized)?;

    // Determine data source (avoid returning a reference to a temporary Ref).
    let mut owned_data: Vec<u8> = Vec::new();
    let data_slice: &[u8] = match (args.data.as_deref(), buffer_info_opt) {
        (Some(bytes), None) => bytes,
        (None, Some(buffer)) => {
            owned_data.extend_from_slice(&buffer.data.borrow());
            &owned_data
        }
        (Some(_), Some(_)) => return Err(MplCoreError::TwoDataSources.into()),
        (None, None) => return Err(MplCoreError::NoDataSources.into()),
    };

    // Write data after the plugin header using existing utility.
    update_external_plugin_adapter_data(
        &record,
        Some(&group_core),
        header,
        registry,
        group_info,
        payer_info,
        system_program_info,
        data_slice,
    )
}
