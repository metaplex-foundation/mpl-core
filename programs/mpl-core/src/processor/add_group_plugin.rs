use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::AddGroupPluginV1Accounts,
    plugins::{
        create_meta_idempotent, initialize_plugin, Plugin, PluginType, PluginValidationContext,
        ValidationResult,
    },
    state::{Authority, GroupV1},
    utils::{is_valid_group_authority, resolve_authority},
};

/// Arguments for the `AddGroupPluginV1` instruction.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddGroupPluginV1Args {
    pub(crate) plugin: Plugin,
    /// Optional authority to initialize for the plugin. Defaults to the plugin's manager.
    pub(crate) init_authority: Option<Authority>,
}

/// Processor for `AddGroupPluginV1`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_group_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddGroupPluginV1Args,
) -> ProgramResult {
    // Expected accounts:
    // 0. [writable] Group account
    // 1. [writable, signer] Payer – also acts as default authority if none provided
    // 2. [signer] Optional authority (update authority or its delegate)
    // 3. [] System program
    // 4. [] Optional SPL Noop (log wrapper)
    let ctx = AddGroupPluginV1Accounts::context(accounts)?;
    let group_info = ctx.accounts.group;
    let payer_info = ctx.accounts.payer;
    let authority_info_opt = ctx.accounts.authority;
    let system_program_info = ctx.accounts.system_program;

    // Optional log wrapper validation
    if let Some(wrapper_info) = ctx.accounts.log_wrapper {
        if wrapper_info.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    // Basic guards
    assert_signer(payer_info)?;
    let authority_info = resolve_authority(payer_info, authority_info_opt)?;

    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if !is_valid_group_authority(group_info, authority_info)? {
        msg!("Error: Invalid authority for group account");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // ------------------------------------------------------------------
    //  ALLOW-LIST ENFORCEMENT
    // ------------------------------------------------------------------
    // Only the following plugin types are permitted on Group accounts.
    // • Attributes
    // • VerifiedCreators
    // • Autograph

    let plugin_type = PluginType::from(&args.plugin);

    let is_allowed_plugin = matches!(
        plugin_type,
        PluginType::Attributes | PluginType::VerifiedCreators | PluginType::Autograph
    );

    // Reject any plugin that is not in the allow-list.
    if !is_allowed_plugin {
        return Err(MplCoreError::InvalidPlugin.into());
    }

    // Plugin-specific validation (best-effort, mirrors AddCollectionPlugin)
    let default_authority = args.plugin.manager();
    let target_plugin_authority = args.init_authority.as_ref().unwrap_or(&default_authority);

    let validation_ctx = PluginValidationContext {
        accounts,
        asset_info: None,
        collection_info: None,
        self_authority: target_plugin_authority,
        authority_info,
        resolved_authorities: None,
        new_owner: None,
        new_asset_authority: None,
        new_collection_authority: None,
        target_plugin: Some(&args.plugin),
        target_plugin_authority: Some(target_plugin_authority),
        target_external_plugin: None,
        target_external_plugin_authority: None,
    };
    if Plugin::validate_add_plugin(&args.plugin, &validation_ctx)? == ValidationResult::Rejected {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Persist plugin – create meta if needed then initialize.
    let (_group, header_offset, mut plugin_header, mut plugin_registry) =
        create_meta_idempotent::<GroupV1>(group_info, payer_info, system_program_info)?;

    initialize_plugin::<GroupV1>(
        &args.plugin,
        target_plugin_authority,
        header_offset,
        &mut plugin_header,
        &mut plugin_registry,
        group_info,
        payer_info,
        system_program_info,
    )?;

    // No other state changes required on the Group struct.

    Ok(())
}
