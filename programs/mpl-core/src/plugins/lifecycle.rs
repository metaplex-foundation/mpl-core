use borsh::{BorshDeserialize, BorshSerialize};
use modular_bitfield::{bitfield, specifiers::B5};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};
use std::collections::BTreeMap;

use crate::{
    error::MplCoreError,
    state::{Authority, Key},
};

use super::{
    ExternalPlugin, ExternalPluginKey, ExternalRegistryRecord, Plugin, PluginType, RegistryRecord,
};

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

/// Lifecycle permissions for external, third party plugins.
/// Third party plugins use this field to indicate their permission to listen, approve, and/or
/// deny a lifecycle event.
#[derive(BorshDeserialize, BorshSerialize, Eq, PartialEq, Copy, Clone, Debug)]
pub struct ExternalCheckResult {
    /// Bitfield for external check results.
    pub flags: u8,
}

impl ExternalCheckResult {
    pub(crate) fn none() -> Self {
        Self { flags: 0 }
    }
}

/// Bitfield representation of lifecycle permissions for external, third party plugins.
#[bitfield(bits = 8)]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ExternalCheckResultBits {
    pub can_listen: bool,
    pub can_approve: bool,
    pub can_reject: bool,
    pub empty_bits: B5,
}

impl From<ExternalCheckResult> for ExternalCheckResultBits {
    fn from(check_result: ExternalCheckResult) -> Self {
        ExternalCheckResultBits::from_bytes(check_result.flags.to_le_bytes())
    }
}

impl From<ExternalCheckResultBits> for ExternalCheckResult {
    fn from(bits: ExternalCheckResultBits) -> Self {
        ExternalCheckResult {
            flags: bits.into_bytes()[0],
        }
    }
}

impl PluginType {
    /// Check permissions for the add plugin lifecycle event.
    pub fn check_add_plugin(plugin_type: &PluginType) -> CheckResult {
        match plugin_type {
            PluginType::Royalties => CheckResult::CanReject,
            PluginType::UpdateDelegate => CheckResult::CanApprove,
            PluginType::PermanentFreezeDelegate => CheckResult::CanReject,
            PluginType::PermanentTransferDelegate => CheckResult::CanReject,
            PluginType::PermanentBurnDelegate => CheckResult::CanReject,
            PluginType::Edition => CheckResult::CanReject,
            _ => CheckResult::None,
        }
    }

    /// Check permissions for the remove plugin lifecycle event.
    pub fn check_remove_plugin(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            PluginType::UpdateDelegate => CheckResult::CanApprove,
            PluginType::FreezeDelegate => CheckResult::CanReject,
            PluginType::PermanentFreezeDelegate => CheckResult::CanReject,
            PluginType::Edition => CheckResult::CanReject,
            _ => CheckResult::None,
        }
    }

    /// Check permissions for the update plugin lifecycle event.
    pub fn check_update_plugin(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => CheckResult::CanApprove,
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
            PluginType::UpdateDelegate => CheckResult::CanApprove,
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny an update action.
    pub fn check_update(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            PluginType::UpdateDelegate => CheckResult::CanApprove,
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny a burn action.
    pub fn check_burn(plugin_type: &PluginType) -> CheckResult {
        match plugin_type {
            PluginType::FreezeDelegate => CheckResult::CanReject,
            PluginType::BurnDelegate => CheckResult::CanApprove,
            PluginType::PermanentFreezeDelegate => CheckResult::CanReject,
            PluginType::PermanentBurnDelegate => CheckResult::CanApprove,
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny a transfer action.
    pub fn check_transfer(plugin_type: &PluginType) -> CheckResult {
        match plugin_type {
            PluginType::Royalties => CheckResult::CanReject,
            PluginType::FreezeDelegate => CheckResult::CanReject,
            PluginType::TransferDelegate => CheckResult::CanApprove,
            PluginType::PermanentFreezeDelegate => CheckResult::CanReject,
            PluginType::PermanentTransferDelegate => CheckResult::CanApprove,
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

    /// Check permissions for the add external plugin lifecycle event.
    pub fn check_add_external_plugin(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => CheckResult::None,
        }
    }

    /// Check permissions for the remove external plugin lifecycle event.
    pub fn check_remove_external_plugin(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => CheckResult::None,
        }
    }

    /// Check permissions for the update external plugin lifecycle event.
    pub fn check_update_external_plugin(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => CheckResult::None,
        }
    }
}

impl Plugin {
    /// Validate the add plugin lifecycle event.
    pub(crate) fn validate_add_plugin(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_add_plugin(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_add_plugin(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_add_plugin(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_add_plugin(ctx),
            Plugin::UpdateDelegate(update_delegate) => update_delegate.validate_add_plugin(ctx),
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_add_plugin(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_add_plugin(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_add_plugin(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_add_plugin(ctx)
            }
            Plugin::Edition(edition) => edition.validate_add_plugin(ctx),
        }
    }

    /// Validate the remove plugin lifecycle event.
    pub(crate) fn validate_remove_plugin(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_remove_plugin(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_remove_plugin(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_remove_plugin(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_remove_plugin(ctx),
            Plugin::UpdateDelegate(update_delegate) => update_delegate.validate_remove_plugin(ctx),
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_remove_plugin(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_remove_plugin(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_remove_plugin(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_remove_plugin(ctx)
            }
            Plugin::Edition(edition) => edition.validate_remove_plugin(ctx),
        }
    }

    /// Validate the approve plugin authority lifecycle event.
    pub(crate) fn validate_approve_plugin_authority(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // Universally, we cannot delegate a plugin authority if it's already delegated, even if
        // we're the manager.
        if let Some(plugin_to_approve) = ctx.target_plugin {
            if plugin_to_approve == plugin && &plugin_to_approve.manager() != ctx.self_authority {
                return Err(MplCoreError::CannotRedelegate.into());
            }
        } else {
            return Err(MplCoreError::InvalidPlugin.into());
        }

        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_approve_plugin_authority(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_approve_plugin_authority(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_approve_plugin_authority(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_approve_plugin_authority(ctx),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_approve_plugin_authority(ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_approve_plugin_authority(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_approve_plugin_authority(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_approve_plugin_authority(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_approve_plugin_authority(ctx)
            }
            Plugin::Edition(edition) => edition.validate_approve_plugin_authority(ctx),
        }
    }

    /// Validate the revoke plugin authority lifecycle event.
    pub(crate) fn validate_revoke_plugin_authority(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.self_authority == &Authority::None {
            solana_program::msg!("Base: Rejected");
            return Ok(ValidationResult::Rejected);
        }

        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_revoke_plugin_authority(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_revoke_plugin_authority(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_revoke_plugin_authority(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_revoke_plugin_authority(ctx),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_revoke_plugin_authority(ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_revoke_plugin_authority(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_revoke_plugin_authority(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_revoke_plugin_authority(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_revoke_plugin_authority(ctx)
            }
            Plugin::Edition(edition) => edition.validate_revoke_plugin_authority(ctx),
        }
    }

    /// Route the validation of the create action to the appropriate plugin.
    pub(crate) fn validate_create(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_create(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_create(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_create(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_create(ctx),
            Plugin::UpdateDelegate(update_delegate) => update_delegate.validate_create(ctx),
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_create(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_create(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_create(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => permanent_burn.validate_create(ctx),
            Plugin::Edition(edition) => edition.validate_create(ctx),
        }
    }

    /// Route the validation of the update action to the appropriate plugin.
    pub(crate) fn validate_update(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_update(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_update(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_update(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_update(ctx),
            Plugin::UpdateDelegate(update_delegate) => update_delegate.validate_update(ctx),
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_update(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_update(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_update(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => permanent_burn.validate_update(ctx),
            Plugin::Edition(edition) => edition.validate_update(ctx),
        }
    }

    /// Route the validation of the update_plugin action to the appropriate plugin.
    /// There is no check for updating a plugin because the plugin itself MUST validate the change.
    pub(crate) fn validate_update_plugin(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let resolved_authorities = ctx
            .resolved_authorities
            .ok_or(MplCoreError::InvalidAuthority)?;
        let base_result = if resolved_authorities.contains(ctx.self_authority) {
            solana_program::msg!("Base: Approved");
            ValidationResult::Approved
        } else {
            ValidationResult::Pass
        };

        let result = match plugin {
            Plugin::Royalties(royalties) => royalties.validate_update_plugin(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_update_plugin(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_update_plugin(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_update_plugin(ctx),
            Plugin::UpdateDelegate(update_delegate) => update_delegate.validate_update_plugin(ctx),
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_update_plugin(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_update_plugin(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_update_plugin(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_update_plugin(ctx)
            }
            Plugin::Edition(edition) => edition.validate_update_plugin(ctx),
        }?;

        match (&base_result, &result) {
            (ValidationResult::Approved, ValidationResult::Approved) => {
                Ok(ValidationResult::Approved)
            }
            (ValidationResult::Approved, ValidationResult::Rejected) => {
                Ok(ValidationResult::Rejected)
            }
            (ValidationResult::Rejected, ValidationResult::Approved) => {
                Ok(ValidationResult::Rejected)
            }
            (ValidationResult::Rejected, ValidationResult::Rejected) => {
                Ok(ValidationResult::Rejected)
            }
            (ValidationResult::Pass, _) => Ok(result),
            (ValidationResult::ForceApproved, _) => Ok(ValidationResult::ForceApproved),
            (_, ValidationResult::Pass) => Ok(base_result),
            (_, ValidationResult::ForceApproved) => Ok(ValidationResult::ForceApproved),
        }
    }

    /// Route the validation of the burn action to the appropriate plugin.
    pub(crate) fn validate_burn(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_burn(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_burn(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_burn(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_burn(ctx),
            Plugin::UpdateDelegate(update_delegate) => update_delegate.validate_burn(ctx),
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_burn(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_burn(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_burn(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => permanent_burn.validate_burn(ctx),
            Plugin::Edition(edition) => edition.validate_burn(ctx),
        }
    }

    /// Route the validation of the transfer action to the appropriate plugin.
    pub(crate) fn validate_transfer(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_transfer(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_transfer(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_transfer(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_transfer(ctx),
            Plugin::UpdateDelegate(update_delegate) => update_delegate.validate_transfer(ctx),
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_transfer(ctx)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_transfer(ctx)
            }
            Plugin::Attributes(attributes_transfer) => attributes_transfer.validate_transfer(ctx),
            Plugin::PermanentBurnDelegate(burn_transfer) => burn_transfer.validate_transfer(ctx),
            Plugin::Edition(edition) => edition.validate_transfer(ctx),
        }
    }

    /// Route the validation of the compress action to the appropriate plugin.
    pub(crate) fn validate_compress(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_compress(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_compress(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_compress(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_compress(ctx),
            Plugin::UpdateDelegate(update_delegate) => update_delegate.validate_compress(ctx),
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_compress(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_compress(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_compress(ctx)
            }
            Plugin::PermanentBurnDelegate(burn_transfer) => burn_transfer.validate_compress(ctx),
            Plugin::Edition(edition) => edition.validate_compress(ctx),
        }
    }

    /// Route the validation of the decompress action to the appropriate plugin.
    pub(crate) fn validate_decompress(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_decompress(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_decompress(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_decompress(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_decompress(ctx),
            Plugin::UpdateDelegate(update_delegate) => update_delegate.validate_decompress(ctx),
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_decompress(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_decompress(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_decompress(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_decompress(ctx)
            }
            Plugin::Edition(edition) => edition.validate_decompress(ctx),
        }
    }

    /// Validate the add external plugin lifecycle event.
    pub(crate) fn validate_add_external_plugin(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_add_external_plugin(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_add_external_plugin(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_add_external_plugin(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_add_external_plugin(ctx),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_add_external_plugin(ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_add_external_plugin(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_add_external_plugin(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_add_external_plugin(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_add_external_plugin(ctx)
            }
            Plugin::Edition(edition) => edition.validate_add_external_plugin(ctx),
        }
    }

    /// Validate the remove plugin lifecycle event.
    pub(crate) fn validate_remove_external_plugin(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_remove_external_plugin(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_remove_external_plugin(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_remove_external_plugin(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_remove_external_plugin(ctx),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_remove_external_plugin(ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_remove_external_plugin(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_remove_external_plugin(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_remove_external_plugin(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_remove_external_plugin(ctx)
            }
            Plugin::Edition(edition) => edition.validate_remove_external_plugin(ctx),
        }
    }

    /// Route the validation of the update_plugin action to the appropriate plugin.
    /// There is no check for updating a plugin because the plugin itself MUST validate the change.
    pub(crate) fn validate_update_external_plugin(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let resolved_authorities = ctx
            .resolved_authorities
            .ok_or(MplCoreError::InvalidAuthority)?;
        let base_result = if resolved_authorities.contains(ctx.self_authority) {
            solana_program::msg!("Base: Approved");
            ValidationResult::Approved
        } else {
            ValidationResult::Pass
        };

        let result = match plugin {
            Plugin::Royalties(royalties) => royalties.validate_update_external_plugin(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_update_external_plugin(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_update_external_plugin(ctx),
            Plugin::TransferDelegate(transfer) => transfer.validate_update_external_plugin(ctx),
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_update_external_plugin(ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_update_external_plugin(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_update_external_plugin(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_update_external_plugin(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_update_external_plugin(ctx)
            }
            Plugin::Edition(edition) => edition.validate_update_external_plugin(ctx),
        }?;

        match (&base_result, &result) {
            (ValidationResult::Approved, ValidationResult::Approved) => {
                Ok(ValidationResult::Approved)
            }
            (ValidationResult::Approved, ValidationResult::Rejected) => {
                Ok(ValidationResult::Rejected)
            }
            (ValidationResult::Rejected, ValidationResult::Approved) => {
                Ok(ValidationResult::Rejected)
            }
            (ValidationResult::Rejected, ValidationResult::Rejected) => {
                Ok(ValidationResult::Rejected)
            }
            (ValidationResult::Pass, _) => Ok(result),
            (ValidationResult::ForceApproved, _) => Ok(ValidationResult::ForceApproved),
            (_, ValidationResult::Pass) => Ok(base_result),
            (_, ValidationResult::ForceApproved) => Ok(ValidationResult::ForceApproved),
        }
    }
}

/// Lifecycle validations
/// Plugins utilize this to indicate whether they approve or reject a lifecycle action.
#[derive(Eq, PartialEq, Debug, Clone, BorshDeserialize, BorshSerialize)]
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

/// External plugins lifecycle validations
/// External plugins utilize this to indicate whether they approve or reject a lifecycle action.
#[derive(Eq, PartialEq, Debug, Clone, BorshDeserialize, BorshSerialize)]
pub enum ExternalValidationResult {
    /// The plugin approves the lifecycle action.
    Approved,
    /// The plugin rejects the lifecycle action.
    Rejected,
    /// The plugin abstains from approving or rejecting the lifecycle action.
    Pass,
}

/// The required context for a plugin validation.
#[allow(dead_code)]
pub(crate) struct PluginValidationContext<'a, 'b> {
    /// The authority.
    pub self_authority: &'b Authority,
    /// The authority account.
    pub authority_info: &'a AccountInfo<'a>,
    /// The resolved authority.
    pub resolved_authorities: Option<&'b [Authority]>,
    /// The new owner account.
    pub new_owner: Option<&'a AccountInfo<'a>>,
    /// The new plugin.
    pub target_plugin: Option<&'b Plugin>,
}

/// Plugin validation trait which is implemented by each plugin.
pub(crate) trait PluginValidation {
    /// Validate the add plugin lifecycle action.
    fn validate_add_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the remove plugin lifecycle action.
    fn validate_remove_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the add plugin lifecycle action.
    fn validate_add_external_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the remove plugin lifecycle action.
    fn validate_remove_external_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the approve plugin authority lifecycle action.
    fn validate_approve_plugin_authority(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the create lifecycle action.
    fn validate_create(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the update lifecycle action.
    fn validate_update(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the update_plugin lifecycle action.
    fn validate_update_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the burn lifecycle action.
    fn validate_burn(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the transfer lifecycle action.
    fn validate_transfer(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the compress lifecycle action.
    fn validate_compress(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the decompress lifecycle action.
    fn validate_decompress(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the add_authority lifecycle action.
    fn validate_add_authority(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the add_authority lifecycle action.
    fn validate_remove_authority(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the update_plugin lifecycle action.
    fn validate_update_external_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
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
    authority: &'a AccountInfo<'a>,
    new_owner: Option<&'a AccountInfo<'a>>,
    new_plugin: Option<&Plugin>,
    asset: Option<&AccountInfo<'a>>,
    collection: Option<&AccountInfo<'a>>,
    resolved_authorities: &[Authority],
    plugin_validate_fp: fn(
        &Plugin,
        &PluginValidationContext,
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
            let account = match key {
                Key::CollectionV1 => collection.ok_or(MplCoreError::InvalidCollection)?,
                Key::AssetV1 => asset.ok_or(MplCoreError::InvalidAsset)?,
                _ => unreachable!(),
            };

            let ctx = PluginValidationContext {
                self_authority: &registry_record.authority,
                authority_info: authority,
                resolved_authorities: Some(resolved_authorities),
                new_owner,
                target_plugin: new_plugin,
            };

            let result = plugin_validate_fp(&Plugin::load(account, registry_record.offset)?, &ctx)?;
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

/// This function iterates through all external plugin checks passed in and performs the validation
/// by deserializing and calling validate on the plugin.
/// The STRONGEST result is returned.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn validate_external_plugin_checks<'a>(
    key: Key,
    external_checks: &BTreeMap<
        ExternalPluginKey,
        (Key, ExternalCheckResultBits, ExternalRegistryRecord),
    >,
    authority: &'a AccountInfo<'a>,
    new_owner: Option<&'a AccountInfo<'a>>,
    new_plugin: Option<&Plugin>,
    asset: Option<&AccountInfo<'a>>,
    collection: Option<&AccountInfo<'a>>,
    resolved_authorities: &[Authority],
    external_plugin_validate_fp: fn(
        &ExternalPlugin,
        &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError>,
) -> Result<ValidationResult, ProgramError> {
    let mut approved = false;
    for (check_key, check_result, external_registry_record) in external_checks.values() {
        if *check_key == key
            && (check_result.can_listen()
                || check_result.can_approve()
                || check_result.can_reject())
        {
            let account = match key {
                Key::CollectionV1 => collection.ok_or(MplCoreError::InvalidCollection)?,
                Key::AssetV1 => asset.ok_or(MplCoreError::InvalidAsset)?,
                _ => unreachable!(),
            };

            let ctx = PluginValidationContext {
                self_authority: &external_registry_record.authority,
                authority_info: authority,
                resolved_authorities: Some(resolved_authorities),
                new_owner,
                target_plugin: new_plugin,
            };

            let result = external_plugin_validate_fp(
                &ExternalPlugin::load(account, external_registry_record.offset)?,
                &ctx,
            )?;
            match result {
                ValidationResult::Rejected => return Ok(ValidationResult::Rejected),
                ValidationResult::Approved => approved = true,
                ValidationResult::Pass => continue,
                // Force approved will not be possible from external plugins.
                ValidationResult::ForceApproved => unreachable!(),
            }
        }
    }

    if approved {
        Ok(ValidationResult::Approved)
    } else {
        Ok(ValidationResult::Pass)
    }
}
