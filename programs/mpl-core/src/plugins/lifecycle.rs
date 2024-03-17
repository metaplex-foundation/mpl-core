use std::collections::BTreeMap;

use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    error::MplCoreError,
    state::{Authority, CoreAsset, Key},
};

use super::{Plugin, PluginType, RegistryRecord};

/// Lifecycle permissions
/// Plugins use this field to indicate their permission to approve or deny
/// a lifecycle action.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum CheckResult {
    /// A plugin is permitted to approve a lifecycle action.
    CanApprove,
    /// A plugin is permitted to reject a lifecycle action.
    CanReject,
    /// A plugin is not permitted to approve or reject a lifecycle action.
    None,
    /// Certain plugins can force approve a lifecycle action.
    CanForceApprove,
}

impl PluginType {
    /// Check permissions for the add plugin lifecycle event.
    pub fn check_add_plugin(plugin_type: &PluginType) -> CheckResult {
        match plugin_type {
            PluginType::Royalties => CheckResult::CanReject,
            PluginType::UpdateDelegate => CheckResult::CanApprove,
            PluginType::PermanentFreeze => CheckResult::CanReject,
            PluginType::PermanentTransfer => CheckResult::CanReject,
            _ => CheckResult::None,
        }
    }

    /// Check permissions for the remove plugin lifecycle event.
    pub fn check_remove_plugin(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            PluginType::UpdateDelegate => CheckResult::CanApprove,
            _ => CheckResult::None,
        }
    }

    /// Check permissions for the approve plugin authority lifecycle event.
    pub fn check_approve_plugin_authority(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => CheckResult::CanApprove,
        }
    }

    /// Check permissions for the revoke plugin authority lifecycle event.
    pub fn check_revoke_plugin_authority(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            //TODO: This isn't very efficient because it requires every plugin to be deserialized
            // to check if it's the plugin whose authority is being revoked.
            _ => CheckResult::CanApprove,
        }
    }

    /// Check if a plugin is permitted to approve or deny a create action.
    pub fn check_create(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            PluginType::Royalties => CheckResult::CanReject,
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny an update action.
    pub fn check_update(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny a burn action.
    pub fn check_burn(plugin_type: &PluginType) -> CheckResult {
        match plugin_type {
            PluginType::Freeze => CheckResult::CanReject,
            PluginType::Burn => CheckResult::CanApprove,
            PluginType::PermanentFreeze => CheckResult::CanReject,
            PluginType::PermanentBurn => CheckResult::CanApprove,
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny a transfer action.
    pub fn check_transfer(plugin_type: &PluginType) -> CheckResult {
        match plugin_type {
            PluginType::Royalties => CheckResult::CanReject,
            PluginType::Freeze => CheckResult::CanReject,
            PluginType::Transfer => CheckResult::CanApprove,
            PluginType::PermanentFreeze => CheckResult::CanReject,
            PluginType::PermanentTransfer => CheckResult::CanApprove,
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny a compress action.
    pub fn check_compress(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny a decompress action.
    pub fn check_decompress(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => CheckResult::None,
        }
    }
}

impl Plugin {
    /// Validate the add plugin lifecycle event.
    pub fn validate_add_plugin(
        plugin: &Plugin,
        authority_info: &AccountInfo,
        _: Option<&AccountInfo>,
        authority: &Authority,
        new_plugin: Option<&Plugin>,
        _: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => {
                royalties.validate_add_plugin(authority_info, authority, new_plugin)
            }
            Plugin::Freeze(freeze) => {
                freeze.validate_add_plugin(authority_info, authority, new_plugin)
            }
            Plugin::Burn(burn) => burn.validate_add_plugin(authority_info, authority, new_plugin),
            Plugin::Transfer(transfer) => {
                transfer.validate_add_plugin(authority_info, authority, new_plugin)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_add_plugin(authority_info, authority, new_plugin)
            }
            Plugin::PermanentFreeze(permanent_freeze) => {
                permanent_freeze.validate_add_plugin(authority_info, authority, new_plugin)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_add_plugin(authority_info, authority, new_plugin)
            }
            Plugin::PermanentTransfer(permanent_transfer) => {
                permanent_transfer.validate_add_plugin(authority_info, authority, new_plugin)
            }
            Plugin::PermanentBurn(permanent_burn) => {
                permanent_burn.validate_add_plugin(authority_info, authority, new_plugin)
            }
        }
    }

    /// Validate the remove plugin lifecycle event.
    pub fn validate_remove_plugin(
        plugin: &Plugin,
        authority_info: &AccountInfo,
        _: Option<&AccountInfo>,
        authority: &Authority,
        plugin_to_remove: Option<&Plugin>,
        _: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => {
                royalties.validate_remove_plugin(authority_info, authority, plugin_to_remove)
            }
            Plugin::Freeze(freeze) => {
                freeze.validate_remove_plugin(authority_info, authority, plugin_to_remove)
            }
            Plugin::Burn(burn) => {
                burn.validate_remove_plugin(authority_info, authority, plugin_to_remove)
            }
            Plugin::Transfer(transfer) => {
                transfer.validate_remove_plugin(authority_info, authority, plugin_to_remove)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_remove_plugin(authority_info, authority, plugin_to_remove)
            }
            Plugin::PermanentFreeze(permanent_freeze) => {
                permanent_freeze.validate_remove_plugin(authority_info, authority, plugin_to_remove)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_remove_plugin(authority_info, authority, plugin_to_remove)
            }
            Plugin::PermanentTransfer(permanent_transfer) => permanent_transfer
                .validate_remove_plugin(authority_info, authority, plugin_to_remove),
            Plugin::PermanentBurn(permanent_burn) => {
                permanent_burn.validate_remove_plugin(authority_info, authority, plugin_to_remove)
            }
        }
    }

    /// Validate the approve plugin authority lifecycle event.
    pub fn validate_approve_plugin_authority(
        plugin: &Plugin,
        authority_info: &AccountInfo,
        _: Option<&AccountInfo>,
        authority: &Authority,
        plugin_to_approve: Option<&Plugin>,
        _: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_approve_plugin_authority(
                authority_info,
                authority,
                plugin_to_approve,
            ),
            Plugin::Freeze(freeze) => freeze.validate_approve_plugin_authority(
                authority_info,
                authority,
                plugin_to_approve,
            ),
            Plugin::Burn(burn) => {
                burn.validate_approve_plugin_authority(authority_info, authority, plugin_to_approve)
            }
            Plugin::Transfer(transfer) => transfer.validate_approve_plugin_authority(
                authority_info,
                authority,
                plugin_to_approve,
            ),
            Plugin::UpdateDelegate(update_delegate) => update_delegate
                .validate_approve_plugin_authority(authority_info, authority, plugin_to_approve),
            Plugin::PermanentFreeze(permanent_freeze) => permanent_freeze
                .validate_approve_plugin_authority(authority_info, authority, plugin_to_approve),
            Plugin::Attributes(attributes) => attributes.validate_approve_plugin_authority(
                authority_info,
                authority,
                plugin_to_approve,
            ),
            Plugin::PermanentTransfer(permanent_transfer) => permanent_transfer
                .validate_approve_plugin_authority(authority_info, authority, plugin_to_approve),
            Plugin::PermanentBurn(permanent_burn) => permanent_burn
                .validate_approve_plugin_authority(authority_info, authority, plugin_to_approve),
        }
    }

    /// Validate the revoke plugin authority lifecycle event.
    pub fn validate_revoke_plugin_authority(
        plugin: &Plugin,
        authority_info: &AccountInfo,
        _: Option<&AccountInfo>,
        authority: &Authority,
        plugin_to_revoke: Option<&Plugin>,
        _: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority == &Authority::None {
            return Ok(ValidationResult::Rejected);
        }

        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_revoke_plugin_authority(
                authority_info,
                authority,
                plugin_to_revoke,
            ),
            Plugin::Freeze(freeze) => {
                freeze.validate_revoke_plugin_authority(authority_info, authority, plugin_to_revoke)
            }
            Plugin::Burn(burn) => {
                burn.validate_revoke_plugin_authority(authority_info, authority, plugin_to_revoke)
            }
            Plugin::Transfer(transfer) => transfer.validate_revoke_plugin_authority(
                authority_info,
                authority,
                plugin_to_revoke,
            ),
            Plugin::UpdateDelegate(update_delegate) => update_delegate
                .validate_revoke_plugin_authority(authority_info, authority, plugin_to_revoke),
            Plugin::PermanentFreeze(permanent_freeze) => permanent_freeze
                .validate_revoke_plugin_authority(authority_info, authority, plugin_to_revoke),
            Plugin::Attributes(attributes) => attributes.validate_revoke_plugin_authority(
                authority_info,
                authority,
                plugin_to_revoke,
            ),
            Plugin::PermanentTransfer(permanent_transfer) => permanent_transfer
                .validate_revoke_plugin_authority(authority_info, authority, plugin_to_revoke),
            Plugin::PermanentBurn(permanent_burn) => permanent_burn
                .validate_revoke_plugin_authority(authority_info, authority, plugin_to_revoke),
        }
    }

    /// Route the validation of the create action to the appropriate plugin.
    pub(crate) fn validate_create(
        plugin: &Plugin,
        authority_info: &AccountInfo,
        _: Option<&AccountInfo>,
        authority: &Authority,
        _: Option<&Plugin>,
        _: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_create(authority_info, authority),
            Plugin::Freeze(freeze) => freeze.validate_create(authority_info, authority),
            Plugin::Burn(burn) => burn.validate_create(authority_info, authority),
            Plugin::Transfer(transfer) => transfer.validate_create(authority_info, authority),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_create(authority_info, authority)
            }
            Plugin::PermanentFreeze(permanent_freeze) => {
                permanent_freeze.validate_create(authority_info, authority)
            }
            Plugin::Attributes(attributes) => attributes.validate_create(authority_info, authority),
            Plugin::PermanentTransfer(permanent_transfer) => {
                permanent_transfer.validate_create(authority_info, authority)
            }
            Plugin::PermanentBurn(permanent_burn) => {
                permanent_burn.validate_create(authority_info, authority)
            }
        }
    }

    /// Route the validation of the update action to the appropriate plugin.
    pub(crate) fn validate_update(
        plugin: &Plugin,
        authority_info: &AccountInfo,
        _: Option<&AccountInfo>,
        authority: &Authority,
        _: Option<&Plugin>,
        _: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_update(authority_info, authority),
            Plugin::Freeze(freeze) => freeze.validate_update(authority_info, authority),
            Plugin::Burn(burn) => burn.validate_update(authority_info, authority),
            Plugin::Transfer(transfer) => transfer.validate_update(authority_info, authority),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_update(authority_info, authority)
            }
            Plugin::PermanentFreeze(permanent_freeze) => {
                permanent_freeze.validate_update(authority_info, authority)
            }
            Plugin::Attributes(attributes) => attributes.validate_update(authority_info, authority),
            Plugin::PermanentTransfer(permanent_transfer) => {
                permanent_transfer.validate_update(authority_info, authority)
            }
            Plugin::PermanentBurn(permanent_burn) => {
                permanent_burn.validate_update(authority_info, authority)
            }
        }
    }

    /// Route the validation of the update_plugin action to the appropriate plugin.
    /// There is no check for updating a plugin because the plugin itself MUST validate the change.
    pub(crate) fn validate_update_plugin<T: CoreAsset>(
        plugin: &Plugin,
        core_asset: &T,
        authority_info: &AccountInfo,
        _: Option<&AccountInfo>,
        authority: &Authority,
        _: Option<&Plugin>,
        _: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => {
                royalties.validate_update_plugin(core_asset, authority_info, authority)
            }
            Plugin::Freeze(freeze) => {
                freeze.validate_update_plugin(core_asset, authority_info, authority)
            }
            Plugin::Burn(burn) => {
                burn.validate_update_plugin(core_asset, authority_info, authority)
            }
            Plugin::Transfer(transfer) => {
                transfer.validate_update_plugin(core_asset, authority_info, authority)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_update_plugin(core_asset, authority_info, authority)
            }
            Plugin::PermanentFreeze(permanent_freeze) => {
                permanent_freeze.validate_update_plugin(core_asset, authority_info, authority)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_update_plugin(core_asset, authority_info, authority)
            }
            Plugin::PermanentTransfer(permanent_transfer) => {
                permanent_transfer.validate_update_plugin(core_asset, authority_info, authority)
            }
            Plugin::PermanentBurn(permanent_burn) => {
                permanent_burn.validate_update_plugin(core_asset, authority_info, authority)
            }
        }
    }

    /// Route the validation of the burn action to the appropriate plugin.
    pub(crate) fn validate_burn(
        plugin: &Plugin,
        authority_info: &AccountInfo,
        _: Option<&AccountInfo>,
        authority: &Authority,
        _: Option<&Plugin>,
        resolved_authority: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => {
                royalties.validate_burn(authority_info, authority, resolved_authority)
            }
            Plugin::Freeze(freeze) => {
                freeze.validate_burn(authority_info, authority, resolved_authority)
            }
            Plugin::Burn(burn) => burn.validate_burn(authority_info, authority, resolved_authority),
            Plugin::Transfer(transfer) => {
                transfer.validate_burn(authority_info, authority, resolved_authority)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_burn(authority_info, authority, resolved_authority)
            }
            Plugin::PermanentFreeze(permanent_freeze) => {
                permanent_freeze.validate_burn(authority_info, authority, resolved_authority)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_burn(authority_info, authority, resolved_authority)
            }
            Plugin::PermanentTransfer(permanent_transfer) => {
                permanent_transfer.validate_burn(authority_info, authority, resolved_authority)
            }
            Plugin::PermanentBurn(permanent_burn) => {
                permanent_burn.validate_burn(authority_info, authority, resolved_authority)
            }
        }
    }

    /// Route the validation of the transfer action to the appropriate plugin.
    pub(crate) fn validate_transfer(
        plugin: &Plugin,
        authority_info: &AccountInfo,
        new_owner: Option<&AccountInfo>,
        authority: &Authority,
        _: Option<&Plugin>,
        resolved_authority: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        let new_owner = new_owner.ok_or(MplCoreError::MissingNewOwner)?;
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_transfer(
                authority_info,
                new_owner,
                authority,
                resolved_authority,
            ),
            Plugin::Freeze(freeze) => {
                freeze.validate_transfer(authority_info, new_owner, authority, resolved_authority)
            }
            Plugin::Burn(burn) => {
                burn.validate_transfer(authority_info, new_owner, authority, resolved_authority)
            }
            Plugin::Transfer(transfer) => {
                transfer.validate_transfer(authority_info, new_owner, authority, resolved_authority)
            }
            Plugin::UpdateDelegate(update_delegate) => update_delegate.validate_transfer(
                authority_info,
                new_owner,
                authority,
                resolved_authority,
            ),
            Plugin::PermanentFreeze(permanent_freeze) => permanent_freeze.validate_transfer(
                authority_info,
                new_owner,
                authority,
                resolved_authority,
            ),
            Plugin::PermanentTransfer(permanent_transfer) => permanent_transfer.validate_transfer(
                authority_info,
                new_owner,
                authority,
                resolved_authority,
            ),
            Plugin::Attributes(attributes_transfer) => attributes_transfer.validate_transfer(
                authority_info,
                new_owner,
                authority,
                resolved_authority,
            ),
            Plugin::PermanentBurn(burn_transfer) => burn_transfer.validate_transfer(
                authority_info,
                new_owner,
                authority,
                resolved_authority,
            ),
        }
    }

    /// Route the validation of the compress action to the appropriate plugin.
    pub(crate) fn validate_compress(
        plugin: &Plugin,
        authority_info: &AccountInfo,
        _: Option<&AccountInfo>,
        authority: &Authority,
        _: Option<&Plugin>,
        _: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_compress(authority_info, authority),
            Plugin::Freeze(freeze) => freeze.validate_compress(authority_info, authority),
            Plugin::Burn(burn) => burn.validate_compress(authority_info, authority),
            Plugin::Transfer(transfer) => transfer.validate_compress(authority_info, authority),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_compress(authority_info, authority)
            }
            Plugin::PermanentFreeze(permanent_freeze) => {
                permanent_freeze.validate_compress(authority_info, authority)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_compress(authority_info, authority)
            }
            Plugin::PermanentTransfer(permanent_transfer) => {
                permanent_transfer.validate_compress(authority_info, authority)
            }
            Plugin::PermanentBurn(burn_transfer) => {
                burn_transfer.validate_compress(authority_info, authority)
            }
        }
    }

    /// Route the validation of the decompress action to the appropriate plugin.
    pub(crate) fn validate_decompress(
        plugin: &Plugin,
        authority_info: &AccountInfo,
        _: Option<&AccountInfo>,
        authority: &Authority,
        _: Option<&Plugin>,
        _: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => {
                royalties.validate_decompress(authority_info, authority)
            }
            Plugin::Freeze(freeze) => freeze.validate_decompress(authority_info, authority),
            Plugin::Burn(burn) => burn.validate_decompress(authority_info, authority),
            Plugin::Transfer(transfer) => transfer.validate_decompress(authority_info, authority),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_decompress(authority_info, authority)
            }
            Plugin::PermanentFreeze(permanent_freeze) => {
                permanent_freeze.validate_decompress(authority_info, authority)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_decompress(authority_info, authority)
            }
            Plugin::PermanentTransfer(permanent_transfer) => {
                permanent_transfer.validate_decompress(authority_info, authority)
            }
            Plugin::PermanentBurn(permanent_burn) => {
                permanent_burn.validate_decompress(authority_info, authority)
            }
        }
    }
}

/// Lifecycle validations
/// Plugins utilize this to indicate whether they approve or reject a lifecycle action.
#[derive(Eq, PartialEq, Debug)]
pub enum ValidationResult {
    /// The plugin approves the lifecycle action.
    Approved,
    /// The plugin rejects the lifecycle action.
    Rejected,
    /// The plugin abstains from approving or rejecting the lifecycle action.
    Pass,
    /// The plugin force approves the lifecycle action.
    ForceApproved,
}

/// The required context for a plugin validation.
#[allow(dead_code)]
pub(crate) struct PluginValidationContext<'a, 'b> {
    /// The authority.
    pub self_authority: &'b Authority,
    /// The authority account.
    pub authority_info: &'a AccountInfo<'a>,
    /// The resolved authority.
    pub resolved_authority: Option<&'a Authority>,
    /// The new owner account.
    pub new_owner: Option<&'a AccountInfo<'a>>,
    /// The new plugin.
    pub target_plugin: Option<&'a Plugin>,
}

/// Plugin validation trait which is implemented by each plugin.
pub(crate) trait PluginValidation {
    /// Validate the add plugin lifecycle action.
    fn validate_add_plugin(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        _new_plugin: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the remove plugin lifecycle action.
    fn validate_remove_plugin(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        _plugin_to_remove: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the approve plugin authority lifecycle action.
    fn validate_approve_plugin_authority(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        _plugin_to_approve: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        _plugin_to_revoke: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the create lifecycle action.
    fn validate_create(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the update lifecycle action.
    fn validate_update(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the update_plugin lifecycle action.
    fn validate_update_plugin<T: CoreAsset>(
        &self,
        core_asset: &T,
        authority_info: &AccountInfo,
        authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        if (authority_info.key == core_asset.owner() && authority == &Authority::Owner)
            || (authority_info.key == &core_asset.update_authority().key()
                && authority == &Authority::UpdateAuthority)
            || authority
                == (&Authority::Pubkey {
                    address: *authority_info.key,
                })
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the burn lifecycle action.
    fn validate_burn(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        _resolved_authority: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the transfer lifecycle action.
    fn validate_transfer(
        &self,
        _authority_info: &AccountInfo,
        _new_owner: &AccountInfo,
        _authority: &Authority,
        _resolved_authority: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the compress lifecycle action.
    fn validate_compress(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the decompress lifecycle action.
    fn validate_decompress(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the add_authority lifecycle action.
    fn validate_add_authority(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the add_authority lifecycle action.
    fn validate_remove_authority(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }
}

/// This function iterates through all plugin checks passed in and performs the validation
/// by deserializing and calling validate on the plugin.
/// The STRONGEST result is returned.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn validate_plugin_checks<'a>(
    key: Key,
    checks: &BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    authority: &AccountInfo<'a>,
    new_owner: Option<&AccountInfo>,
    new_plugin: Option<&Plugin>,
    asset: Option<&AccountInfo<'a>>,
    collection: Option<&AccountInfo<'a>>,
    resolved_authority: &Authority,
    validate_fp: fn(
        &Plugin,
        &AccountInfo<'a>,
        Option<&AccountInfo>,
        &Authority,
        Option<&Plugin>,
        Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError>,
) -> Result<ValidationResult, ProgramError> {
    let mut approved = false;
    let mut rejected = false;
    for (check_key, check_result, registry_record) in checks.values() {
        if *check_key == key
            && matches!(
                check_result,
                CheckResult::CanApprove | CheckResult::CanReject
            )
        {
            solana_program::msg!("Validating plugin checks");
            let account = match key {
                Key::Collection => collection.ok_or(MplCoreError::InvalidCollection)?,
                Key::Asset => asset.ok_or(MplCoreError::InvalidAsset)?,
                _ => unreachable!(),
            };

            let result = validate_fp(
                &Plugin::load(account, registry_record.offset)?,
                authority,
                new_owner,
                &registry_record.authority,
                new_plugin,
                Some(resolved_authority),
            )?;
            match result {
                ValidationResult::Rejected => rejected = true,
                ValidationResult::Approved => approved = true,
                ValidationResult::Pass => continue,
                ValidationResult::ForceApproved => return Ok(ValidationResult::ForceApproved),
            }
        }
    }

    if rejected {
        Ok(ValidationResult::Rejected)
    } else if approved {
        Ok(ValidationResult::Approved)
    } else {
        Ok(ValidationResult::Pass)
    }
}
