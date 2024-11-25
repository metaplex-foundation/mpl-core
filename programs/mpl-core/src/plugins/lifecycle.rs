use borsh::{BorshDeserialize, BorshSerialize};
use modular_bitfield::{bitfield, specifiers::B29};
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use std::collections::BTreeMap;

use crate::{
    error::MplCoreError,
    state::{Authority, Key, UpdateAuthority},
};

use super::{
    ExternalPluginAdapter, ExternalPluginAdapterKey, ExternalRegistryRecord, Plugin, PluginType,
    RegistryRecord,
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

/// Lifecycle permissions for adapter, third party plugins.
/// Third party plugins use this field to indicate their permission to listen, approve, and/or
/// deny a lifecycle event.
#[derive(BorshDeserialize, BorshSerialize, Eq, PartialEq, Copy, Clone, Debug)]
pub struct ExternalCheckResult {
    /// Bitfield for external plugin adapter check results.
    pub flags: u32,
}

impl ExternalCheckResult {
    pub(crate) fn none() -> Self {
        Self { flags: 0 }
    }

    pub(crate) fn can_reject_only() -> Self {
        Self { flags: 0x4 }
    }
}

/// Bitfield representation of lifecycle permissions for external plugin adapter, third party plugins.
#[bitfield(bits = 32)]
#[derive(Eq, PartialEq, Copy, Clone, Debug, Default)]
pub struct ExternalCheckResultBits {
    pub can_listen: bool,
    pub can_approve: bool,
    pub can_reject: bool,
    pub empty_bits: B29,
}

impl From<ExternalCheckResult> for ExternalCheckResultBits {
    fn from(check_result: ExternalCheckResult) -> Self {
        ExternalCheckResultBits::from_bytes(check_result.flags.to_le_bytes())
    }
}

impl From<ExternalCheckResultBits> for ExternalCheckResult {
    fn from(bits: ExternalCheckResultBits) -> Self {
        ExternalCheckResult {
            flags: u32::from_le_bytes(bits.into_bytes()),
        }
    }
}

impl PluginType {
    /// Check permissions for the add plugin lifecycle event.
    pub fn check_add_plugin(plugin_type: &PluginType) -> CheckResult {
        match plugin_type {
            PluginType::AddBlocker => CheckResult::CanReject,
            PluginType::Royalties => CheckResult::CanReject,
            PluginType::UpdateDelegate => CheckResult::CanApprove,
            PluginType::PermanentFreezeDelegate => CheckResult::CanReject,
            PluginType::PermanentTransferDelegate => CheckResult::CanReject,
            PluginType::PermanentBurnDelegate => CheckResult::CanReject,
            PluginType::Edition => CheckResult::CanReject,
            PluginType::Autograph => CheckResult::CanReject,
            PluginType::VerifiedCreators => CheckResult::CanReject,
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
            // We default to CanReject because Plugins with Authority::None cannot be removed.
            _ => CheckResult::CanReject,
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
            PluginType::Autograph => CheckResult::CanReject,
            PluginType::VerifiedCreators => CheckResult::CanReject,
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny an update action.
    pub fn check_update(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            PluginType::ImmutableMetadata => CheckResult::CanReject,
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

    /// Check permissions for the add external plugin adapter lifecycle event.
    pub fn check_add_external_plugin_adapter(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => CheckResult::None,
        }
    }

    /// Check permissions for the remove external plugin adapter lifecycle event.
    pub fn check_remove_external_plugin_adapter(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => CheckResult::None,
        }
    }

    /// Check permissions for the update external plugin adapter lifecycle event.
    pub fn check_update_external_plugin_adapter(plugin_type: &PluginType) -> CheckResult {
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
            Plugin::MasterEdition(master_edition) => master_edition.validate_add_plugin(ctx),
            Plugin::AddBlocker(add_blocker) => add_blocker.validate_add_plugin(ctx),
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_add_plugin(ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_add_plugin(ctx)
            }
            Plugin::Autograph(autograph) => autograph.validate_add_plugin(ctx),
        }
    }

    /// Validate the remove plugin lifecycle event.
    pub(crate) fn validate_remove_plugin(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.self_authority == &Authority::None
            && ctx.target_plugin.is_some()
            && PluginType::from(ctx.target_plugin.unwrap()) == PluginType::from(plugin)
        {
            return reject!();
        }

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
            Plugin::MasterEdition(master_edition) => master_edition.validate_remove_plugin(ctx),
            Plugin::AddBlocker(add_blocker) => add_blocker.validate_remove_plugin(ctx),
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_remove_plugin(ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_remove_plugin(ctx)
            }
            Plugin::Autograph(autograph) => autograph.validate_remove_plugin(ctx),
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
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_approve_plugin_authority(ctx)
            }
            Plugin::AddBlocker(add_blocker) => add_blocker.validate_approve_plugin_authority(ctx),
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_approve_plugin_authority(ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_approve_plugin_authority(ctx)
            }
            Plugin::Autograph(autograph) => autograph.validate_approve_plugin_authority(ctx),
        }
    }

    /// Validate the revoke plugin authority lifecycle event.
    pub(crate) fn validate_revoke_plugin_authority(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let target_plugin = ctx.target_plugin.ok_or(MplCoreError::InvalidPlugin)?;

        // If the plugin being checked is Authority::None then it can't be revoked.
        if ctx.self_authority == &Authority::None
            && PluginType::from(target_plugin) == PluginType::from(plugin)
        {
            return reject!();
        }

        let base_result = if PluginType::from(target_plugin) == PluginType::from(plugin)
            && ctx.resolved_authorities.is_some()
            && ctx
                .resolved_authorities
                .unwrap()
                .contains(ctx.self_authority)
        {
            solana_program::msg!("Base: Approved");
            ValidationResult::Approved
        } else {
            ValidationResult::Pass
        };

        let result = match plugin {
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
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_revoke_plugin_authority(ctx)
            }
            Plugin::AddBlocker(add_blocker) => add_blocker.validate_revoke_plugin_authority(ctx),
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_revoke_plugin_authority(ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_revoke_plugin_authority(ctx)
            }
            Plugin::Autograph(autograph) => autograph.validate_revoke_plugin_authority(ctx),
        }?;

        if result == ValidationResult::Pass {
            Ok(base_result)
        } else {
            Ok(result)
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
            Plugin::MasterEdition(master_edition) => master_edition.validate_create(ctx),
            Plugin::AddBlocker(add_blocker) => add_blocker.validate_create(ctx),
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_create(ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => verified_creators.validate_create(ctx),
            Plugin::Autograph(autograph) => autograph.validate_create(ctx),
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
            Plugin::MasterEdition(master_edition) => master_edition.validate_update(ctx),
            Plugin::AddBlocker(add_blocker) => add_blocker.validate_update(ctx),
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_update(ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => verified_creators.validate_update(ctx),
            Plugin::Autograph(autograph) => autograph.validate_update(ctx),
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
            Plugin::MasterEdition(master_edition) => master_edition.validate_update_plugin(ctx),
            Plugin::AddBlocker(add_blocker) => add_blocker.validate_update_plugin(ctx),
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_update_plugin(ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_update_plugin(ctx)
            }
            Plugin::Autograph(autograph) => autograph.validate_update_plugin(ctx),
        }?;

        match (&base_result, &result) {
            (ValidationResult::Approved, ValidationResult::Approved) => {
                approve!()
            }
            (ValidationResult::Approved, ValidationResult::Rejected) => {
                reject!()
            }
            (ValidationResult::Rejected, ValidationResult::Approved) => {
                reject!()
            }
            (ValidationResult::Rejected, ValidationResult::Rejected) => {
                reject!()
            }
            (ValidationResult::Pass, _) => Ok(result),
            (ValidationResult::ForceApproved, _) => force_approve!(),
            (_, ValidationResult::Pass) => Ok(base_result),
            (_, ValidationResult::ForceApproved) => force_approve!(),
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
            Plugin::MasterEdition(master_edition) => master_edition.validate_burn(ctx),
            Plugin::AddBlocker(add_blocker) => add_blocker.validate_burn(ctx),
            Plugin::ImmutableMetadata(immutable_metadata) => immutable_metadata.validate_burn(ctx),
            Plugin::VerifiedCreators(verified_creators) => verified_creators.validate_burn(ctx),
            Plugin::Autograph(autograph) => autograph.validate_burn(ctx),
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
            Plugin::MasterEdition(master_edition) => master_edition.validate_transfer(ctx),
            Plugin::AddBlocker(add_blocker) => add_blocker.validate_transfer(ctx),
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_transfer(ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => verified_creators.validate_transfer(ctx),
            Plugin::Autograph(autograph) => autograph.validate_transfer(ctx),
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
            Plugin::MasterEdition(master_edition) => master_edition.validate_compress(ctx),
            Plugin::AddBlocker(add_blocker) => add_blocker.validate_compress(ctx),
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_compress(ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => verified_creators.validate_compress(ctx),
            Plugin::Autograph(autograph) => autograph.validate_compress(ctx),
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
            Plugin::MasterEdition(master_edition) => master_edition.validate_decompress(ctx),
            Plugin::AddBlocker(add_blocker) => add_blocker.validate_decompress(ctx),
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_decompress(ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_decompress(ctx)
            }
            Plugin::Autograph(autograph) => autograph.validate_decompress(ctx),
        }
    }

    /// Validate the add external plugin adapter lifecycle event.
    pub(crate) fn validate_add_external_plugin_adapter(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_add_external_plugin_adapter(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_add_external_plugin_adapter(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_add_external_plugin_adapter(ctx),
            Plugin::TransferDelegate(transfer) => {
                transfer.validate_add_external_plugin_adapter(ctx)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_add_external_plugin_adapter(ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_add_external_plugin_adapter(ctx)
            }
            Plugin::Attributes(attributes) => attributes.validate_add_external_plugin_adapter(ctx),
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_add_external_plugin_adapter(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_add_external_plugin_adapter(ctx)
            }
            Plugin::Edition(edition) => edition.validate_add_external_plugin_adapter(ctx),
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_add_external_plugin_adapter(ctx)
            }
            Plugin::AddBlocker(add_blocker) => {
                add_blocker.validate_add_external_plugin_adapter(ctx)
            }
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_add_external_plugin_adapter(ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_add_external_plugin_adapter(ctx)
            }
            Plugin::Autograph(autograph) => autograph.validate_add_external_plugin_adapter(ctx),
        }
    }

    /// Validate the remove plugin lifecycle event.
    pub(crate) fn validate_remove_external_plugin_adapter(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_remove_external_plugin_adapter(ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_remove_external_plugin_adapter(ctx),
            Plugin::BurnDelegate(burn) => burn.validate_remove_external_plugin_adapter(ctx),
            Plugin::TransferDelegate(transfer) => {
                transfer.validate_remove_external_plugin_adapter(ctx)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_remove_external_plugin_adapter(ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_remove_external_plugin_adapter(ctx)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_remove_external_plugin_adapter(ctx)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_remove_external_plugin_adapter(ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_remove_external_plugin_adapter(ctx)
            }
            Plugin::Edition(edition) => edition.validate_remove_external_plugin_adapter(ctx),
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_remove_external_plugin_adapter(ctx)
            }
            Plugin::AddBlocker(add_blocker) => {
                add_blocker.validate_remove_external_plugin_adapter(ctx)
            }
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_remove_external_plugin_adapter(ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_remove_external_plugin_adapter(ctx)
            }
            Plugin::Autograph(autograph) => autograph.validate_remove_external_plugin_adapter(ctx),
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

// Create a shortcut macro for passing on a lifecycle action.
macro_rules! abstain {
    () => {{
        Ok(ValidationResult::Pass)
    }};
}
pub(crate) use abstain;

// Create a shortcut macro for rejecting a lifecycle action.
macro_rules! reject {
    () => {{
        solana_program::msg!("{}:{}:Reject", std::file!(), std::line!());
        Ok(ValidationResult::Rejected)
    }};
}
pub(crate) use reject;

// Create a shortcut macro for approving a lifecycle action.
macro_rules! approve {
    () => {{
        solana_program::msg!("{}:{}:Approve", std::file!(), std::line!());
        Ok(ValidationResult::Approved)
    }};
}
pub(crate) use approve;

// Create a shortcut macro for force-approving a lifecycle action.
macro_rules! force_approve {
    () => {{
        solana_program::msg!("{}:{}:ForceApprove", std::file!(), std::line!());
        Ok(ValidationResult::ForceApproved)
    }};
}
pub(crate) use force_approve;

/// External plugin adapters lifecycle validations
/// External plugin adapters utilize this to indicate whether they approve or reject a lifecycle action.
#[derive(Eq, PartialEq, Debug, Clone, BorshDeserialize, BorshSerialize)]
pub enum ExternalValidationResult {
    /// The plugin approves the lifecycle action.
    Approved,
    /// The plugin rejects the lifecycle action.
    Rejected,
    /// The plugin abstains from approving or rejecting the lifecycle action.
    Pass,
}

impl From<ExternalValidationResult> for ValidationResult {
    fn from(result: ExternalValidationResult) -> Self {
        match result {
            ExternalValidationResult::Approved => Self::Approved,
            ExternalValidationResult::Rejected => Self::Rejected,
            ExternalValidationResult::Pass => Self::Pass,
        }
    }
}

/// The required context for a plugin validation.
#[allow(dead_code)]
pub(crate) struct PluginValidationContext<'a, 'b> {
    /// This list of all the accounts passed into the instruction.
    pub accounts: &'a [AccountInfo<'a>],
    /// The asset account.
    pub asset_info: Option<&'a AccountInfo<'a>>,
    /// The collection account.
    pub collection_info: Option<&'a AccountInfo<'a>>,
    /// The authority of the current (self) plugin
    pub self_authority: &'b Authority,
    /// The authority account info of ix `authority` signer
    pub authority_info: &'a AccountInfo<'a>,
    /// The authorities types which match the authority signer
    pub resolved_authorities: Option<&'b [Authority]>,
    /// The new owner account for transfers
    pub new_owner: Option<&'a AccountInfo<'a>>,
    /// The new asset authority address.
    pub new_asset_authority: Option<&'b UpdateAuthority>,
    /// The new collection authority address.
    pub new_collection_authority: Option<&'b Pubkey>,
    /// The plugin being acted upon with new data from the ix if any. This None for create.
    pub target_plugin: Option<&'b Plugin>,
}

/// Plugin validation trait which is implemented by each plugin.
pub(crate) trait PluginValidation {
    /// Validate the add plugin lifecycle action.
    /// This gets called on all existing plugins when a new plugin is added.
    fn validate_add_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the remove plugin lifecycle action.
    /// This gets called on all existing plugins when the target plugin is removed.
    fn validate_remove_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the add plugin lifecycle action.
    /// This gets called on all existing plugins when a new external plugin is added.
    fn validate_add_external_plugin_adapter(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the remove plugin lifecycle action.
    /// This gets called on all existing plugins when a new external plugin is removed.
    fn validate_remove_external_plugin_adapter(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the approve plugin authority lifecycle action.
    fn validate_approve_plugin_authority(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the create lifecycle action.
    /// This ONLY gets called to validate the self plugin
    fn validate_create(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the update lifecycle action.
    /// This gets called on all existing plugins when an asset or collection is updated.
    fn validate_update(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the update_plugin lifecycle action.
    /// This gets called on all existing plugins when a plugin is updated.
    fn validate_update_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the burn lifecycle action.
    /// This gets called on all existing plugins when an asset is burned.
    fn validate_burn(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the transfer lifecycle action.
    /// This gets called on all existing plugins when an asset is transferred.
    fn validate_transfer(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the compress lifecycle action.
    fn validate_compress(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the decompress lifecycle action.
    fn validate_decompress(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the update_plugin lifecycle action.
    fn validate_update_external_plugin_adapter(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }
}

/// This function iterates through all plugin checks passed in and performs the validation
/// by deserializing and calling validate on the plugin.
/// The STRONGEST result is returned.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn validate_plugin_checks<'a>(
    key: Key,
    accounts: &'a [AccountInfo<'a>],
    checks: &BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    authority: &'a AccountInfo<'a>,
    new_owner: Option<&'a AccountInfo<'a>>,
    new_asset_authority: Option<&UpdateAuthority>,
    new_collection_authority: Option<&Pubkey>,
    new_plugin: Option<&Plugin>,
    asset: Option<&'a AccountInfo<'a>>,
    collection: Option<&'a AccountInfo<'a>>,
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

            let validation_ctx = PluginValidationContext {
                accounts,
                asset_info: asset,
                collection_info: collection,
                self_authority: &registry_record.authority,
                authority_info: authority,
                resolved_authorities: Some(resolved_authorities),
                new_owner,
                new_asset_authority,
                new_collection_authority,
                target_plugin: new_plugin,
            };

            let result = plugin_validate_fp(
                &Plugin::load(account, registry_record.offset)?,
                &validation_ctx,
            )?;
            match result {
                ValidationResult::Rejected => rejected = true,
                ValidationResult::Approved => approved = true,
                ValidationResult::Pass => continue,
                ValidationResult::ForceApproved => return force_approve!(),
            }
        }
    }

    if rejected {
        reject!()
    } else if approved {
        approve!()
    } else {
        abstain!()
    }
}

/// This function iterates through all external plugin adapter checks passed in and performs the validation
/// by deserializing and calling validate on the plugin.
/// The STRONGEST result is returned.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn validate_external_plugin_adapter_checks<'a>(
    key: Key,
    accounts: &'a [AccountInfo<'a>],
    external_checks: &BTreeMap<
        ExternalPluginAdapterKey,
        (Key, ExternalCheckResultBits, ExternalRegistryRecord),
    >,
    authority: &'a AccountInfo<'a>,
    new_owner: Option<&'a AccountInfo<'a>>,
    new_asset_authority: Option<&UpdateAuthority>,
    new_collection_authority: Option<&Pubkey>,
    new_plugin: Option<&Plugin>,
    asset: Option<&'a AccountInfo<'a>>,
    collection: Option<&'a AccountInfo<'a>>,
    resolved_authorities: &[Authority],
    external_plugin_adapter_validate_fp: fn(
        &ExternalPluginAdapter,
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

            let validation_ctx = PluginValidationContext {
                accounts,
                asset_info: asset,
                collection_info: collection,
                self_authority: &external_registry_record.authority,
                authority_info: authority,
                resolved_authorities: Some(resolved_authorities),
                new_owner,
                new_asset_authority,
                new_collection_authority,
                target_plugin: new_plugin,
            };

            let result = external_plugin_adapter_validate_fp(
                &ExternalPluginAdapter::load(account, external_registry_record.offset)?,
                &validation_ctx,
            )?;
            match result {
                ValidationResult::Rejected => {
                    if check_result.can_reject() {
                        return reject!();
                    }
                }
                ValidationResult::Approved => {
                    if check_result.can_approve() {
                        approved = true;
                    }
                }
                ValidationResult::Pass => continue,
                // Force approved will not be possible from external plugin adapters.
                ValidationResult::ForceApproved => unreachable!(),
            }
        }
    }

    if approved {
        approve!()
    } else {
        abstain!()
    }
}
