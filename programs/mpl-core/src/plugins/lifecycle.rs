use std::collections::BTreeMap;

use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    error::MplCoreError,
    processor::{
        ApprovePluginAuthorityArgs, CompressArgs, CreateArgs, DecompressArgs,
        RevokePluginAuthorityArgs,
    },
    state::{Asset, Authority, Key},
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
}

impl PluginType {
    /// Check if a plugin is permitted to approve or deny a create action.
    pub fn check_create(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
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
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny a transfer action.
    pub fn check_transfer(plugin_type: &PluginType) -> CheckResult {
        match plugin_type {
            PluginType::Royalties => CheckResult::CanReject,
            PluginType::Freeze => CheckResult::CanReject,
            PluginType::Transfer => CheckResult::CanApprove,
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
    /// Route the validation of the create action to the appropriate plugin.
    pub(crate) fn validate_create(
        plugin: &Plugin,
        authority: &AccountInfo,
        args: &CreateArgs,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_create(authority, args, authorities),
            Plugin::Freeze(freeze) => freeze.validate_create(authority, args, authorities),
            Plugin::Burn(burn) => burn.validate_create(authority, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_create(authority, args, authorities),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_create(authority, args, authorities)
            }
        }
    }

    /// Route the validation of the update action to the appropriate plugin.
    pub(crate) fn validate_update(
        plugin: &Plugin,
        authority: &AccountInfo,
        _unused: Option<&AccountInfo>,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_update(authority, authorities),
            Plugin::Freeze(freeze) => freeze.validate_update(authority, authorities),
            Plugin::Burn(burn) => burn.validate_update(authority, authorities),
            Plugin::Transfer(transfer) => transfer.validate_update(authority, authorities),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_update(authority, authorities)
            }
        }
    }

    /// Route the validation of the update_plugin action to the appropriate plugin.
    /// There is no check for updating a plugin because the plugin itself MUST validate the change.
    pub(crate) fn validate_update_plugin(
        plugin: &Plugin,
        asset: &Asset,
        authority: &AccountInfo,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => {
                royalties.validate_update_plugin(asset, authority, authorities)
            }
            Plugin::Freeze(freeze) => freeze.validate_update_plugin(asset, authority, authorities),
            Plugin::Burn(burn) => burn.validate_update_plugin(asset, authority, authorities),
            Plugin::Transfer(transfer) => {
                transfer.validate_update_plugin(asset, authority, authorities)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_update_plugin(asset, authority, authorities)
            }
        }
    }

    /// Route the validation of the burn action to the appropriate plugin.
    pub(crate) fn validate_burn(
        plugin: &Plugin,
        authority: &AccountInfo,
        _unused: Option<&AccountInfo>,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_burn(authority, authorities),
            Plugin::Freeze(freeze) => freeze.validate_burn(authority, authorities),
            Plugin::Burn(burn) => burn.validate_burn(authority, authorities),
            Plugin::Transfer(transfer) => transfer.validate_burn(authority, authorities),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_burn(authority, authorities)
            }
        }
    }

    /// Route the validation of the transfer action to the appropriate plugin.
    pub(crate) fn validate_transfer(
        plugin: &Plugin,
        authority: &AccountInfo,
        new_owner: Option<&AccountInfo>,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        let new_owner = new_owner.ok_or(MplCoreError::MissingNewOwner)?;
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => {
                royalties.validate_transfer(authority, new_owner, authorities)
            }
            Plugin::Freeze(freeze) => freeze.validate_transfer(authority, new_owner, authorities),
            Plugin::Burn(burn) => burn.validate_transfer(authority, new_owner, authorities),
            Plugin::Transfer(transfer) => {
                transfer.validate_transfer(authority, new_owner, authorities)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_transfer(authority, new_owner, authorities)
            }
        }
    }

    /// Route the validation of the compress action to the appropriate plugin.
    pub(crate) fn validate_compress(
        plugin: &Plugin,
        authority: &AccountInfo,
        args: &CompressArgs,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => {
                royalties.validate_compress(authority, args, authorities)
            }
            Plugin::Freeze(freeze) => freeze.validate_compress(authority, args, authorities),
            Plugin::Burn(burn) => burn.validate_compress(authority, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_compress(authority, args, authorities),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_compress(authority, args, authorities)
            }
        }
    }

    /// Route the validation of the decompress action to the appropriate plugin.
    pub(crate) fn validate_decompress(
        plugin: &Plugin,
        authority: &AccountInfo,
        args: &DecompressArgs,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => {
                royalties.validate_decompress(authority, args, authorities)
            }
            Plugin::Freeze(freeze) => freeze.validate_decompress(authority, args, authorities),
            Plugin::Burn(burn) => burn.validate_decompress(authority, args, authorities),
            Plugin::Transfer(transfer) => {
                transfer.validate_decompress(authority, args, authorities)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_decompress(authority, args, authorities)
            }
        }
    }

    /// Route the validation of the add_authority action to the appropriate plugin.
    /// There is no check for adding to a plugin because the plugin itself MUST validate the change.
    pub(crate) fn validate_add_plugin_authority(
        plugin: &Plugin,
        authority: &AccountInfo,
        args: &ApprovePluginAuthorityArgs,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => {
                royalties.validate_add_authority(authority, args, authorities)
            }
            Plugin::Freeze(freeze) => freeze.validate_add_authority(authority, args, authorities),
            Plugin::Burn(burn) => burn.validate_add_authority(authority, args, authorities),
            Plugin::Transfer(transfer) => {
                transfer.validate_add_authority(authority, args, authorities)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_add_authority(authority, args, authorities)
            }
        }
    }

    /// Route the validation of the add_authority action to the appropriate plugin.
    /// There is no check for adding to a plugin because the plugin itself MUST validate the change.
    pub(crate) fn validate_remove_plugin_authority(
        plugin: &Plugin,
        authority: &AccountInfo,
        args: &RevokePluginAuthorityArgs,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => {
                royalties.validate_remove_authority(authority, args, authorities)
            }
            Plugin::Freeze(freeze) => {
                freeze.validate_remove_authority(authority, args, authorities)
            }
            Plugin::Burn(burn) => burn.validate_remove_authority(authority, args, authorities),
            Plugin::Transfer(transfer) => {
                transfer.validate_remove_authority(authority, args, authorities)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_remove_authority(authority, args, authorities)
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
}

/// Plugin validation trait which is implemented by each plugin.
pub(crate) trait PluginValidation {
    /// Validate the create lifecycle action.
    fn validate_create(
        &self,
        authority: &AccountInfo,
        args: &CreateArgs,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError>;

    /// Validate the update lifecycle action.
    fn validate_update(
        &self,
        authority: &AccountInfo,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError>;

    /// Validate the update_plugin lifecycle action.
    fn validate_update_plugin(
        &self,
        asset: &Asset,
        authority_info: &AccountInfo,
        authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        if (authority_info.key == &asset.owner && authority == &Authority::Owner)
            || (authority_info.key == &asset.update_authority.key()
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
        authority: &AccountInfo,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError>;

    /// Validate the transfer lifecycle action.
    fn validate_transfer(
        &self,
        authority: &AccountInfo,
        new_owner: &AccountInfo,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError>;

    /// Validate the compress lifecycle action.
    fn validate_compress(
        &self,
        authority: &AccountInfo,
        args: &CompressArgs,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError>;

    /// Validate the decompress lifecycle action.
    fn validate_decompress(
        &self,
        authority: &AccountInfo,
        args: &DecompressArgs,
        authorities: &Authority,
    ) -> Result<ValidationResult, ProgramError>;

    /// Validate the add_authority lifecycle action.
    fn validate_add_authority(
        &self,
        _authority: &AccountInfo,
        _args: &ApprovePluginAuthorityArgs,
        _authorities: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the add_authority lifecycle action.
    fn validate_remove_authority(
        &self,
        _authority: &AccountInfo,
        _args: &RevokePluginAuthorityArgs,
        _authorities: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }
}

pub(crate) fn validate_plugin_checks<'a>(
    key: Key,
    checks: &BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    authority: &AccountInfo<'a>,
    new_owner: Option<&AccountInfo>,
    asset: &AccountInfo<'a>,
    collection: Option<&AccountInfo<'a>>,
    validate_fp: fn(
        &Plugin,
        &AccountInfo<'a>,
        Option<&AccountInfo>,
        &Authority,
    ) -> Result<ValidationResult, ProgramError>,
) -> Result<bool, ProgramError> {
    for (_, (check_key, check_result, registry_record)) in checks {
        if *check_key == key
            && matches!(
                check_result,
                CheckResult::CanApprove | CheckResult::CanReject
            )
        {
            let account = match key {
                Key::Collection => collection.ok_or(MplCoreError::InvalidCollection)?,
                Key::Asset => asset,
                _ => unreachable!(),
            };
            // let result = Plugin::load(account, registry_record.offset)?
            //     .validate_burn(authority, &registry_record.authorities)?;
            let result = validate_fp(
                &Plugin::load(account, registry_record.offset)?,
                authority,
                new_owner,
                &registry_record.authority,
            )?;
            match result {
                ValidationResult::Rejected => return Err(MplCoreError::InvalidAuthority.into()),
                ValidationResult::Approved => return Ok(true),
                ValidationResult::Pass => continue,
            }
        }
    }
    Ok(false)
}
