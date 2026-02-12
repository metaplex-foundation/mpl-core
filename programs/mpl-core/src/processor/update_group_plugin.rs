use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::UpdateGroupPluginV1Accounts,
    plugins::{
        fetch_wrapped_plugin, CheckResult, Plugin, PluginType, PluginValidationContext,
        ValidationResult,
    },
    state::{Authority, DataBlob, GroupV1, SolanaAccount},
    utils::{
        fetch_core_data, is_valid_group_authority, resize_or_reallocate_account, resolve_authority,
    },
};

/// Args for `UpdateGroupPluginV1`.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateGroupPluginV1Args {
    pub plugin: Plugin,
}

pub(crate) fn update_group_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdateGroupPluginV1Args,
) -> ProgramResult {
    let ctx = UpdateGroupPluginV1Accounts::context(accounts)?;
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

    // ------------------------------------------------------------------
    //  ALLOW-LIST ENFORCEMENT (same rules as AddGroupPlugin)
    // ------------------------------------------------------------------
    let incoming_plugin_type = PluginType::from(&args.plugin);

    let is_allowed_plugin = matches!(
        incoming_plugin_type,
        PluginType::Attributes | PluginType::VerifiedCreators | PluginType::Autograph
    );

    if !is_allowed_plugin {
        return Err(MplCoreError::InvalidPlugin.into());
    }

    // Authority check against group update authority or delegate
    if !is_valid_group_authority(group_info, authority_info)? {
        msg!("Error: Invalid authority for group account");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Locate existing plugin
    let (_plugin_authority, _existing_plugin) =
        fetch_wrapped_plugin::<GroupV1>(group_info, None, PluginType::from(&args.plugin))?;

    // Fetch plugin meta
    let (core_stub, plugin_header_opt, plugin_registry_opt) =
        fetch_core_data::<GroupV1>(group_info)?;
    let mut plugin_header = plugin_header_opt.ok_or(MplCoreError::PluginsNotInitialized)?;
    let mut plugin_registry = plugin_registry_opt.ok_or(MplCoreError::PluginsNotInitialized)?;

    // Find registry record
    let registry_record = plugin_registry
        .registry
        .iter()
        .find(|r| r.plugin_type == PluginType::from(&args.plugin))
        .ok_or(MplCoreError::PluginNotFound)?
        .clone();

    // Serialize old and new plugin to calculate diff
    let old_plugin = Plugin::load(group_info, registry_record.offset)?;

    let mut resolved_authorities = vec![Authority::Address {
        address: *authority_info.key,
    }];
    if authority_info.key == &core_stub.update_authority {
        resolved_authorities.push(Authority::Owner);
        resolved_authorities.push(Authority::UpdateAuthority);
    } else {
        // Group delegates can update plugins that are managed by update authority.
        resolved_authorities.push(Authority::UpdateAuthority);
    }

    if !matches!(
        PluginType::check_update_plugin(&incoming_plugin_type),
        CheckResult::CanApprove | CheckResult::CanReject | CheckResult::CanForceApprove
    ) {
        msg!("Error: Update lifecycle checks are disabled for this plugin");
        return Err(MplCoreError::NoApprovals.into());
    }

    let validation_ctx = PluginValidationContext {
        accounts,
        asset_info: None,
        collection_info: None,
        self_authority: &registry_record.authority,
        authority_info,
        resolved_authorities: Some(&resolved_authorities),
        new_owner: None,
        new_asset_authority: None,
        new_collection_authority: None,
        target_plugin: Some(&args.plugin),
        target_plugin_authority: Some(&registry_record.authority),
        target_external_plugin: None,
        target_external_plugin_authority: None,
    };

    match Plugin::validate_update_plugin(&old_plugin, &validation_ctx)? {
        ValidationResult::Approved | ValidationResult::ForceApproved => (),
        ValidationResult::Rejected => {
            msg!("Error: Group plugin update rejected by lifecycle validation");
            return Err(MplCoreError::InvalidAuthority.into());
        }
        ValidationResult::Pass => {
            msg!("Error: Group plugin update has no lifecycle approvals");
            return Err(MplCoreError::NoApprovals.into());
        }
    }

    let old_data = old_plugin.try_to_vec()?;
    let new_data = args.plugin.try_to_vec()?;
    let size_diff = (new_data.len() as isize) - (old_data.len() as isize);

    // If size changes, adjust offsets and account size
    if size_diff != 0 {
        plugin_registry.bump_offsets(registry_record.offset, size_diff)?;
        let new_registry_offset = (plugin_header.plugin_registry_offset as isize + size_diff)
            .try_into()
            .map_err(|_| MplCoreError::NumericalOverflow)?;
        plugin_header.plugin_registry_offset = new_registry_offset;

        let new_account_size = (group_info.data_len() as isize + size_diff)
            .try_into()
            .map_err(|_| MplCoreError::NumericalOverflow)?;
        resize_or_reallocate_account(
            group_info,
            payer_info,
            system_program_info,
            new_account_size,
        )?;
    }

    // Save header, registry, and new plugin
    plugin_header.save(group_info, core_stub.len())?;
    plugin_registry.save(group_info, plugin_header.plugin_registry_offset)?;
    args.plugin.save(group_info, registry_record.offset)?;

    Ok(())
}
