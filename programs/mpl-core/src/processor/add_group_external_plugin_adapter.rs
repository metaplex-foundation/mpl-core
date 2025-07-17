use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};

use crate::{
    error::MplCoreError,
    plugins::{
        create_meta_idempotent, initialize_external_plugin_adapter, ExternalPluginAdapter,
        ExternalPluginAdapterInitInfo, PluginValidationContext, ValidationResult,
    },
    state::{Authority, DataBlob, GroupV1, SolanaAccount},
    utils::{is_valid_group_authority, resolve_authority},
};

/// Arguments for the `AddGroupExternalPluginAdapterV1` instruction.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddGroupExternalPluginAdapterV1Args {
    /// Initialization info for the external plugin adapter. Only `AppData` is permitted.
    pub init_info: ExternalPluginAdapterInitInfo,
}

/// Processor for `AddGroupExternalPluginAdapterV1`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_group_external_plugin_adapter<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddGroupExternalPluginAdapterV1Args,
) -> ProgramResult {
    // Expected accounts:
    // 0. [writable] Group account
    // 1. [writable, signer] Payer account (also default authority)
    // 2. [signer] Optional authority (update authority or update delegate of group)
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

    // Basic guards.
    assert_signer(payer_info)?;
    let authority_info = resolve_authority(payer_info, authority_info_opt)?;

    // Validate system & log wrapper programs.
    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }
    if let Some(wrapper_info) = log_wrapper_info_opt {
        if wrapper_info.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    // Only AppData external plugin adapters are permitted on Group accounts.
    match args.init_info {
        ExternalPluginAdapterInitInfo::AppData(_) => {}
        _ => return Err(MplCoreError::InvalidPluginAdapterTarget.into()),
    }

    // Authority check against the group's update authority or update delegate.
    let group = GroupV1::load(group_info, 0)?;
    if !is_valid_group_authority(group_info, authority_info)? {
        msg!("Error: Invalid authority for group account");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Build the adapter and perform adapter-level validation.
    let external_plugin_adapter = ExternalPluginAdapter::from(&args.init_info);
    // The plugin authority defaults to UpdateAuthority if none supplied.
    let external_plugin_adapter_authority = match &args.init_info {
        ExternalPluginAdapterInitInfo::AppData(app_data) => app_data
            .init_plugin_authority
            .unwrap_or(Authority::UpdateAuthority),
        _ => unreachable!(),
    };

    // Best-effort plugin validation (mirrors collection implementation).
    let validation_ctx = PluginValidationContext {
        accounts,
        asset_info: None,
        collection_info: None,
        self_authority: &Authority::UpdateAuthority,
        authority_info,
        resolved_authorities: None,
        new_owner: None,
        new_asset_authority: None,
        new_collection_authority: None,
        target_plugin: None,
        target_plugin_authority: None,
        target_external_plugin: Some(&external_plugin_adapter),
        target_external_plugin_authority: Some(&external_plugin_adapter_authority),
    };
    if crate::plugins::ExternalPluginAdapter::validate_add_external_plugin_adapter(
        &external_plugin_adapter,
        &validation_ctx,
    )? == ValidationResult::Rejected
    {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Persist the adapter.
    let (_group_core, header_offset, mut plugin_header, mut plugin_registry) =
        create_meta_idempotent::<GroupV1>(group_info, payer_info, system_program_info)?;

    initialize_external_plugin_adapter::<GroupV1>(
        &args.init_info,
        header_offset,
        &mut plugin_header,
        &mut plugin_registry,
        group_info,
        payer_info,
        system_program_info,
        None,
    )?;

    Ok(())
}
