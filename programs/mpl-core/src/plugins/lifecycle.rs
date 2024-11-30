use borsh::{BorshDeserialize, BorshSerialize};
use modular_bitfield::{bitfield, specifiers::B29};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};
use std::collections::BTreeMap;
use strum::EnumCount;

use crate::{
    error::MplCoreError,
    state::{AssetV1, Authority, CollectionV1, Key, UpdateAuthority},
    utils::load_key,
};

use super::{
    ExternalPluginAdapter, ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey,
    ExternalPluginAdapterUpdateInfo, ExternalRegistryRecord, Plugin, PluginType, RegistryRecord,
};

/// Lifecycle Events
#[derive(Eq, PartialEq, Copy, Clone, Debug, EnumCount)]
pub(crate) enum LifecycleEvent {
    AddExternalPluginAdapter,
    AddPlugin,
    ApprovePluginAuthority,
    Burn,
    Compress,
    Create,
    Decompress,
    RemoveExternalPluginAdapter,
    RemovePlugin,
    RevokePluginAuthority,
    Transfer,
    // UpdateExternalPluginAdapter,
    UpdatePlugin,
    Update,
}

impl LifecycleEvent {
    const ASSET_CHECKS: [fn() -> CheckResult; Self::COUNT] = [
        AssetV1::check_add_external_plugin_adapter,
        AssetV1::check_add_plugin,
        AssetV1::check_approve_plugin_authority,
        AssetV1::check_burn,
        AssetV1::check_compress,
        AssetV1::check_create,
        AssetV1::check_decompress,
        AssetV1::check_remove_external_plugin_adapter,
        AssetV1::check_remove_plugin,
        AssetV1::check_revoke_plugin_authority,
        AssetV1::check_transfer,
        AssetV1::check_update_plugin,
        AssetV1::check_update,
    ];

    const COLLECTION_CHECKS: [fn() -> CheckResult; Self::COUNT] = [
        CollectionV1::check_add_external_plugin_adapter,
        CollectionV1::check_add_plugin,
        CollectionV1::check_approve_plugin_authority,
        CollectionV1::check_burn,
        CollectionV1::check_compress,
        CollectionV1::check_create,
        CollectionV1::check_decompress,
        CollectionV1::check_remove_external_plugin_adapter,
        CollectionV1::check_remove_plugin,
        CollectionV1::check_revoke_plugin_authority,
        CollectionV1::check_transfer,
        CollectionV1::check_update_plugin,
        CollectionV1::check_update,
    ];

    const ASSET_VALIDATORS: [fn(
        &AssetV1,
        // &AccountInfo,
        // Option<&Plugin>,
        // Option<&ExternalPluginAdapter>,
        common: &AssetValidationCommon,
        ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError>; Self::COUNT] = [
        AssetV1::validate_add_external_plugin_adapter,
        AssetV1::validate_add_plugin,
        AssetV1::validate_approve_plugin_authority,
        AssetV1::validate_burn,
        AssetV1::validate_compress,
        AssetV1::validate_create,
        AssetV1::validate_decompress,
        AssetV1::validate_remove_external_plugin_adapter,
        AssetV1::validate_remove_plugin,
        AssetV1::validate_revoke_plugin_authority,
        AssetV1::validate_transfer,
        AssetV1::validate_update_plugin,
        AssetV1::validate_update,
    ];

    const COLLECTION_VALIDATORS: [fn(
        &CollectionV1,
        // &AccountInfo,
        // Option<&Plugin>,
        // Option<&ExternalPluginAdapter>,
        &AssetValidationCommon,
        &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError>; Self::COUNT] = [
        CollectionV1::validate_add_external_plugin_adapter,
        CollectionV1::validate_add_plugin,
        CollectionV1::validate_approve_plugin_authority,
        CollectionV1::validate_burn,
        CollectionV1::validate_compress,
        CollectionV1::validate_create,
        CollectionV1::validate_decompress,
        CollectionV1::validate_remove_external_plugin_adapter,
        CollectionV1::validate_remove_plugin,
        CollectionV1::validate_revoke_plugin_authority,
        CollectionV1::validate_transfer,
        CollectionV1::validate_update_plugin,
        CollectionV1::validate_update,
    ];

    const PLUGIN_VALIDATORS: [fn(
        &Plugin,
        &PluginValidationContext,
        &AssetValidationCommon,
        &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError>; Self::COUNT] = [
        Plugin::validate_add_external_plugin_adapter,
        Plugin::validate_add_plugin,
        Plugin::validate_approve_plugin_authority,
        Plugin::validate_burn,
        Plugin::validate_compress,
        Plugin::validate_create,
        Plugin::validate_decompress,
        Plugin::validate_remove_external_plugin_adapter,
        Plugin::validate_remove_plugin,
        Plugin::validate_revoke_plugin_authority,
        Plugin::validate_transfer,
        Plugin::validate_update_plugin,
        Plugin::validate_update,
    ];
}

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
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => {
                royalties.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::FreezeDelegate(freeze) => {
                freeze.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::BurnDelegate(burn) => burn.validate_add_plugin(plugin_ctx, common, asset_ctx),
            Plugin::TransferDelegate(transfer) => {
                transfer.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::Edition(edition) => edition.validate_add_plugin(plugin_ctx, common, asset_ctx),
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::AddBlocker(add_blocker) => {
                add_blocker.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::Autograph(autograph) => {
                autograph.validate_add_plugin(plugin_ctx, common, asset_ctx)
            }
        }
    }

    /// Validate the remove plugin lifecycle event.
    pub(crate) fn validate_remove_plugin(
        plugin: &Plugin,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::RemovePlugin { plugin_to_remove } = asset_ctx {
            if plugin_ctx.self_authority == &Authority::None
                && PluginType::from(plugin_to_remove) == PluginType::from(plugin)
            {
                return reject!();
            }

            match plugin {
                Plugin::Royalties(royalties) => {
                    royalties.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::FreezeDelegate(freeze) => {
                    freeze.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::BurnDelegate(burn) => {
                    burn.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::TransferDelegate(transfer) => {
                    transfer.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::UpdateDelegate(update_delegate) => {
                    update_delegate.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                    permanent_freeze.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::Attributes(attributes) => {
                    attributes.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::PermanentTransferDelegate(permanent_transfer) => {
                    permanent_transfer.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::PermanentBurnDelegate(permanent_burn) => {
                    permanent_burn.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::Edition(edition) => {
                    edition.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::MasterEdition(master_edition) => {
                    master_edition.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::AddBlocker(add_blocker) => {
                    add_blocker.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::ImmutableMetadata(immutable_metadata) => {
                    immutable_metadata.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::VerifiedCreators(verified_creators) => {
                    verified_creators.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
                Plugin::Autograph(autograph) => {
                    autograph.validate_remove_plugin(plugin_ctx, common, asset_ctx)
                }
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    /// Validate the approve plugin authority lifecycle event.
    pub(crate) fn validate_approve_plugin_authority(
        plugin: &Plugin,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::ApprovePluginAuthority {
            plugin: plugin_to_approve,
        } = asset_ctx
        {
            // Universally, we cannot delegate a plugin authority if it's already delegated, even if
            // we're the manager.
            if plugin_to_approve == plugin
                && &plugin_to_approve.manager() != plugin_ctx.self_authority
            {
                return Err(MplCoreError::CannotRedelegate.into());
            }

            match plugin {
                Plugin::Royalties(royalties) => {
                    royalties.validate_approve_plugin_authority(plugin_ctx, common, asset_ctx)
                }
                Plugin::FreezeDelegate(freeze) => {
                    freeze.validate_approve_plugin_authority(plugin_ctx, common, asset_ctx)
                }
                Plugin::BurnDelegate(burn) => {
                    burn.validate_approve_plugin_authority(plugin_ctx, common, asset_ctx)
                }
                Plugin::TransferDelegate(transfer) => {
                    transfer.validate_approve_plugin_authority(plugin_ctx, common, asset_ctx)
                }
                Plugin::UpdateDelegate(update_delegate) => {
                    update_delegate.validate_approve_plugin_authority(plugin_ctx, common, asset_ctx)
                }
                Plugin::PermanentFreezeDelegate(permanent_freeze) => permanent_freeze
                    .validate_approve_plugin_authority(plugin_ctx, common, asset_ctx),
                Plugin::Attributes(attributes) => {
                    attributes.validate_approve_plugin_authority(plugin_ctx, common, asset_ctx)
                }
                Plugin::PermanentTransferDelegate(permanent_transfer) => permanent_transfer
                    .validate_approve_plugin_authority(plugin_ctx, common, asset_ctx),
                Plugin::PermanentBurnDelegate(permanent_burn) => {
                    permanent_burn.validate_approve_plugin_authority(plugin_ctx, common, asset_ctx)
                }
                Plugin::Edition(edition) => {
                    edition.validate_approve_plugin_authority(plugin_ctx, common, asset_ctx)
                }
                Plugin::MasterEdition(master_edition) => {
                    master_edition.validate_approve_plugin_authority(plugin_ctx, common, asset_ctx)
                }
                Plugin::AddBlocker(add_blocker) => {
                    add_blocker.validate_approve_plugin_authority(plugin_ctx, common, asset_ctx)
                }
                Plugin::ImmutableMetadata(immutable_metadata) => immutable_metadata
                    .validate_approve_plugin_authority(plugin_ctx, common, asset_ctx),
                Plugin::VerifiedCreators(verified_creators) => verified_creators
                    .validate_approve_plugin_authority(plugin_ctx, common, asset_ctx),
                Plugin::Autograph(autograph) => {
                    autograph.validate_approve_plugin_authority(plugin_ctx, common, asset_ctx)
                }
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    /// Validate the revoke plugin authority lifecycle event.
    pub(crate) fn validate_revoke_plugin_authority(
        plugin: &Plugin,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::RevokePluginAuthority {
            plugin: plugin_to_revoke,
        } = asset_ctx
        {
            // If the plugin being checked is Authority::None then it can't be revoked.
            if plugin_ctx.self_authority == &Authority::None
                && PluginType::from(plugin_to_revoke) == PluginType::from(plugin)
            {
                return reject!();
            }

            let base_result = if PluginType::from(plugin_to_revoke) == PluginType::from(plugin)
                && plugin_ctx.resolved_authorities.is_some()
                && plugin_ctx
                    .resolved_authorities
                    .unwrap()
                    .contains(plugin_ctx.self_authority)
            {
                solana_program::msg!("Base: Approved");
                ValidationResult::Approved
            } else {
                ValidationResult::Pass
            };

            let result =
                match plugin {
                    Plugin::Royalties(royalties) => {
                        royalties.validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx)
                    }
                    Plugin::FreezeDelegate(freeze) => {
                        freeze.validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx)
                    }
                    Plugin::BurnDelegate(burn) => {
                        burn.validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx)
                    }
                    Plugin::TransferDelegate(transfer) => {
                        transfer.validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx)
                    }
                    Plugin::UpdateDelegate(update_delegate) => update_delegate
                        .validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx),
                    Plugin::PermanentFreezeDelegate(permanent_freeze) => permanent_freeze
                        .validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx),
                    Plugin::Attributes(attributes) => {
                        attributes.validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx)
                    }
                    Plugin::PermanentTransferDelegate(permanent_transfer) => permanent_transfer
                        .validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx),
                    Plugin::PermanentBurnDelegate(permanent_burn) => permanent_burn
                        .validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx),
                    Plugin::Edition(edition) => {
                        edition.validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx)
                    }
                    Plugin::MasterEdition(master_edition) => master_edition
                        .validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx),
                    Plugin::AddBlocker(add_blocker) => {
                        add_blocker.validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx)
                    }
                    Plugin::ImmutableMetadata(immutable_metadata) => immutable_metadata
                        .validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx),
                    Plugin::VerifiedCreators(verified_creators) => verified_creators
                        .validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx),
                    Plugin::Autograph(autograph) => {
                        autograph.validate_revoke_plugin_authority(plugin_ctx, common, asset_ctx)
                    }
                }?;

            if result == ValidationResult::Pass {
                Ok(base_result)
            } else {
                Ok(result)
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    /// Route the validation of the create action to the appropriate plugin.
    pub(crate) fn validate_create(
        plugin: &Plugin,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => {
                royalties.validate_create(plugin_ctx, common, asset_ctx)
            }
            Plugin::FreezeDelegate(freeze) => freeze.validate_create(plugin_ctx, common, asset_ctx),
            Plugin::BurnDelegate(burn) => burn.validate_create(plugin_ctx, common, asset_ctx),
            Plugin::TransferDelegate(transfer) => {
                transfer.validate_create(plugin_ctx, common, asset_ctx)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_create(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_create(plugin_ctx, common, asset_ctx)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_create(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_create(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_create(plugin_ctx, common, asset_ctx)
            }
            Plugin::Edition(edition) => edition.validate_create(plugin_ctx, common, asset_ctx),
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_create(plugin_ctx, common, asset_ctx)
            }
            Plugin::AddBlocker(add_blocker) => {
                add_blocker.validate_create(plugin_ctx, common, asset_ctx)
            }
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_create(plugin_ctx, common, asset_ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_create(plugin_ctx, common, asset_ctx)
            }
            Plugin::Autograph(autograph) => {
                autograph.validate_create(plugin_ctx, common, asset_ctx)
            }
        }
    }

    /// Route the validation of the update action to the appropriate plugin.
    pub(crate) fn validate_update(
        plugin: &Plugin,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => {
                royalties.validate_update(plugin_ctx, common, asset_ctx)
            }
            Plugin::FreezeDelegate(freeze) => freeze.validate_update(plugin_ctx, common, asset_ctx),
            Plugin::BurnDelegate(burn) => burn.validate_update(plugin_ctx, common, asset_ctx),
            Plugin::TransferDelegate(transfer) => {
                transfer.validate_update(plugin_ctx, common, asset_ctx)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_update(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_update(plugin_ctx, common, asset_ctx)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_update(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_update(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_update(plugin_ctx, common, asset_ctx)
            }
            Plugin::Edition(edition) => edition.validate_update(plugin_ctx, common, asset_ctx),
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_update(plugin_ctx, common, asset_ctx)
            }
            Plugin::AddBlocker(add_blocker) => {
                add_blocker.validate_update(plugin_ctx, common, asset_ctx)
            }
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_update(plugin_ctx, common, asset_ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_update(plugin_ctx, common, asset_ctx)
            }
            Plugin::Autograph(autograph) => {
                autograph.validate_update(plugin_ctx, common, asset_ctx)
            }
        }
    }

    /// Route the validation of the update_plugin action to the appropriate plugin.
    /// There is no check for updating a plugin because the plugin itself MUST validate the change.
    pub(crate) fn validate_update_plugin(
        plugin: &Plugin,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let resolved_authorities = plugin_ctx
            .resolved_authorities
            .ok_or(MplCoreError::InvalidAuthority)?;
        let base_result = if resolved_authorities.contains(plugin_ctx.self_authority) {
            solana_program::msg!("Base: Approved");
            ValidationResult::Approved
        } else {
            ValidationResult::Pass
        };

        let result = match plugin {
            Plugin::Royalties(royalties) => {
                royalties.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::FreezeDelegate(freeze) => {
                freeze.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::BurnDelegate(burn) => {
                burn.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::TransferDelegate(transfer) => {
                transfer.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::Edition(edition) => {
                edition.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::AddBlocker(add_blocker) => {
                add_blocker.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
            Plugin::Autograph(autograph) => {
                autograph.validate_update_plugin(plugin_ctx, common, asset_ctx)
            }
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
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => royalties.validate_burn(plugin_ctx, common, asset_ctx),
            Plugin::FreezeDelegate(freeze) => freeze.validate_burn(plugin_ctx, common, asset_ctx),
            Plugin::BurnDelegate(burn) => burn.validate_burn(plugin_ctx, common, asset_ctx),
            Plugin::TransferDelegate(transfer) => {
                transfer.validate_burn(plugin_ctx, common, asset_ctx)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_burn(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_burn(plugin_ctx, common, asset_ctx)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_burn(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_burn(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_burn(plugin_ctx, common, asset_ctx)
            }
            Plugin::Edition(edition) => edition.validate_burn(plugin_ctx, common, asset_ctx),
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_burn(plugin_ctx, common, asset_ctx)
            }
            Plugin::AddBlocker(add_blocker) => {
                add_blocker.validate_burn(plugin_ctx, common, asset_ctx)
            }
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_burn(plugin_ctx, common, asset_ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_burn(plugin_ctx, common, asset_ctx)
            }
            Plugin::Autograph(autograph) => autograph.validate_burn(plugin_ctx, common, asset_ctx),
        }
    }

    /// Route the validation of the transfer action to the appropriate plugin.
    pub(crate) fn validate_transfer(
        plugin: &Plugin,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => {
                royalties.validate_transfer(plugin_ctx, common, asset_ctx)
            }
            Plugin::FreezeDelegate(freeze) => {
                freeze.validate_transfer(plugin_ctx, common, asset_ctx)
            }
            Plugin::BurnDelegate(burn) => burn.validate_transfer(plugin_ctx, common, asset_ctx),
            Plugin::TransferDelegate(transfer) => {
                transfer.validate_transfer(plugin_ctx, common, asset_ctx)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_transfer(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_transfer(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_transfer(plugin_ctx, common, asset_ctx)
            }
            Plugin::Attributes(attributes_transfer) => {
                attributes_transfer.validate_transfer(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentBurnDelegate(burn_transfer) => {
                burn_transfer.validate_transfer(plugin_ctx, common, asset_ctx)
            }
            Plugin::Edition(edition) => edition.validate_transfer(plugin_ctx, common, asset_ctx),
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_transfer(plugin_ctx, common, asset_ctx)
            }
            Plugin::AddBlocker(add_blocker) => {
                add_blocker.validate_transfer(plugin_ctx, common, asset_ctx)
            }
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_transfer(plugin_ctx, common, asset_ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_transfer(plugin_ctx, common, asset_ctx)
            }
            Plugin::Autograph(autograph) => {
                autograph.validate_transfer(plugin_ctx, common, asset_ctx)
            }
        }
    }

    /// Route the validation of the compress action to the appropriate plugin.
    pub(crate) fn validate_compress(
        plugin: &Plugin,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => {
                royalties.validate_compress(plugin_ctx, common, asset_ctx)
            }
            Plugin::FreezeDelegate(freeze) => {
                freeze.validate_compress(plugin_ctx, common, asset_ctx)
            }
            Plugin::BurnDelegate(burn) => burn.validate_compress(plugin_ctx, common, asset_ctx),
            Plugin::TransferDelegate(transfer) => {
                transfer.validate_compress(plugin_ctx, common, asset_ctx)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_compress(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_compress(plugin_ctx, common, asset_ctx)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_compress(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_compress(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentBurnDelegate(burn_transfer) => {
                burn_transfer.validate_compress(plugin_ctx, common, asset_ctx)
            }
            Plugin::Edition(edition) => edition.validate_compress(plugin_ctx, common, asset_ctx),
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_compress(plugin_ctx, common, asset_ctx)
            }
            Plugin::AddBlocker(add_blocker) => {
                add_blocker.validate_compress(plugin_ctx, common, asset_ctx)
            }
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_compress(plugin_ctx, common, asset_ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_compress(plugin_ctx, common, asset_ctx)
            }
            Plugin::Autograph(autograph) => {
                autograph.validate_compress(plugin_ctx, common, asset_ctx)
            }
        }
    }

    /// Route the validation of the decompress action to the appropriate plugin.
    pub(crate) fn validate_decompress(
        plugin: &Plugin,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => {
                royalties.validate_decompress(plugin_ctx, common, asset_ctx)
            }
            Plugin::FreezeDelegate(freeze) => {
                freeze.validate_decompress(plugin_ctx, common, asset_ctx)
            }
            Plugin::BurnDelegate(burn) => burn.validate_decompress(plugin_ctx, common, asset_ctx),
            Plugin::TransferDelegate(transfer) => {
                transfer.validate_decompress(plugin_ctx, common, asset_ctx)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_decompress(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_decompress(plugin_ctx, common, asset_ctx)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_decompress(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer) => {
                permanent_transfer.validate_decompress(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_decompress(plugin_ctx, common, asset_ctx)
            }
            Plugin::Edition(edition) => edition.validate_decompress(plugin_ctx, common, asset_ctx),
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_decompress(plugin_ctx, common, asset_ctx)
            }
            Plugin::AddBlocker(add_blocker) => {
                add_blocker.validate_decompress(plugin_ctx, common, asset_ctx)
            }
            Plugin::ImmutableMetadata(immutable_metadata) => {
                immutable_metadata.validate_decompress(plugin_ctx, common, asset_ctx)
            }
            Plugin::VerifiedCreators(verified_creators) => {
                verified_creators.validate_decompress(plugin_ctx, common, asset_ctx)
            }
            Plugin::Autograph(autograph) => {
                autograph.validate_decompress(plugin_ctx, common, asset_ctx)
            }
        }
    }

    /// Validate the add external plugin adapter lifecycle event.
    pub(crate) fn validate_add_external_plugin_adapter(
        plugin: &Plugin,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => {
                royalties.validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::FreezeDelegate(freeze) => {
                freeze.validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::BurnDelegate(burn) => {
                burn.validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::TransferDelegate(transfer) => {
                transfer.validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                update_delegate.validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze) => {
                permanent_freeze.validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::Attributes(attributes) => {
                attributes.validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer) => permanent_transfer
                .validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx),
            Plugin::PermanentBurnDelegate(permanent_burn) => {
                permanent_burn.validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::Edition(edition) => {
                edition.validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::MasterEdition(master_edition) => {
                master_edition.validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::AddBlocker(add_blocker) => {
                add_blocker.validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::ImmutableMetadata(immutable_metadata) => immutable_metadata
                .validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx),
            Plugin::VerifiedCreators(verified_creators) => verified_creators
                .validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx),
            Plugin::Autograph(autograph) => {
                autograph.validate_add_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
        }
    }

    /// Validate the remove plugin lifecycle event.
    pub(crate) fn validate_remove_external_plugin_adapter(
        plugin: &Plugin,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin {
            Plugin::Royalties(royalties) => {
                royalties.validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::FreezeDelegate(freeze) => {
                freeze.validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::BurnDelegate(burn) => {
                burn.validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::TransferDelegate(transfer) => {
                transfer.validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::UpdateDelegate(update_delegate) => update_delegate
                .validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx),
            Plugin::PermanentFreezeDelegate(permanent_freeze) => permanent_freeze
                .validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx),
            Plugin::Attributes(attributes) => {
                attributes.validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer) => permanent_transfer
                .validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx),
            Plugin::PermanentBurnDelegate(permanent_burn) => permanent_burn
                .validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx),
            Plugin::Edition(edition) => {
                edition.validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::MasterEdition(master_edition) => master_edition
                .validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx),
            Plugin::AddBlocker(add_blocker) => {
                add_blocker.validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
            Plugin::ImmutableMetadata(immutable_metadata) => immutable_metadata
                .validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx),
            Plugin::VerifiedCreators(verified_creators) => verified_creators
                .validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx),
            Plugin::Autograph(autograph) => {
                autograph.validate_remove_external_plugin_adapter(plugin_ctx, common, asset_ctx)
            }
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
pub(crate) struct PluginValidationContext<'b> {
    // /// This list of all the accounts passed into the instruction.
    // pub accounts: &'a [AccountInfo<'a>],
    // /// The asset account.
    // pub asset_info: Option<&'a AccountInfo<'a>>,
    // /// The collection account.
    // pub collection_info: Option<&'a AccountInfo<'a>>,
    /// The authority of the current (self) plugin
    pub self_authority: &'b Authority,
    // /// The authority account info of ix `authority` signer
    // pub authority_info: &'a AccountInfo<'a>,
    /// The authorities types which match the authority signer
    pub resolved_authorities: Option<&'b [Authority]>,
    // /// The new owner account for transfers
    // pub new_owner: Option<&'a AccountInfo<'a>>,
    // /// The new asset authority address.
    // pub new_asset_authority: Option<&'b UpdateAuthority>,
    // /// The new collection authority address.
    // pub new_collection_authority: Option<&'b Pubkey>,
    // /// The plugin being acted upon with new data from the ix if any. This None for create.
    // pub target_plugin: Option<&'b Plugin>,
}

/// The common context for a plugin validation.
pub(crate) struct AssetValidationCommon<'a> {
    // pub(crate) accounts: &'a [AccountInfo<'a>],
    pub(crate) authority_info: &'a AccountInfo<'a>,
    pub(crate) asset_info: &'a AccountInfo<'a>,
    pub(crate) collection_info: Option<&'a AccountInfo<'a>>,
}

/// The required context for a plugin validation.
pub(crate) enum AssetValidationContext<'a> {
    /// Add External Plugin Adapter Context
    AddExternalPluginAdapter {
        new_external_plugin_adapter: ExternalPluginAdapter,
    },
    /// Add Plugin Context
    AddPlugin { new_plugin: Plugin },
    /// Approve Plugin Authority Context
    ApprovePluginAuthority { plugin: Plugin },
    /// Burn Context
    Burn { accounts: &'a [AccountInfo<'a>] },
    /// Compress Context
    Compress,
    /// Create Context
    Create { accounts: &'a [AccountInfo<'a>] },
    /// Decompress Context
    Decompress,
    /// Remove External Plugin Adapter Context
    RemoveExternalPluginAdapter {
        plugin_to_remove: ExternalPluginAdapter,
    },
    /// Remove Plugin Context
    RemovePlugin { plugin_to_remove: Plugin },
    /// Revoke Plugin Authority Context
    RevokePluginAuthority { plugin: Plugin },
    /// Transfer Context
    Transfer {
        new_owner: &'a AccountInfo<'a>,
        accounts: &'a [AccountInfo<'a>],
    },
    // /// Update External Plugin Adapter Context
    UpdateExternalPluginAdapter {
        new_external_plugin_adapter: ExternalPluginAdapterUpdateInfo,
    },
    /// Update Plugin Context
    UpdatePlugin { new_plugin: Plugin },
    /// Update Context
    Update {
        new_update_authority: Option<UpdateAuthority>,
        accounts: &'a [AccountInfo<'a>],
    },
}

/// Plugin validation trait which is implemented by each plugin.
pub(crate) trait PluginValidation {
    /// Validate the add plugin lifecycle action.
    /// This gets called on all existing plugins when a new plugin is added.
    fn validate_add_plugin(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the remove plugin lifecycle action.
    /// This gets called on all existing plugins when the target plugin is removed.
    fn validate_remove_plugin(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the add plugin lifecycle action.
    /// This gets called on all existing plugins when a new external plugin is added.
    fn validate_add_external_plugin_adapter(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the remove plugin lifecycle action.
    /// This gets called on all existing plugins when a new external plugin is removed.
    fn validate_remove_external_plugin_adapter(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the approve plugin authority lifecycle action.
    fn validate_approve_plugin_authority(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the create lifecycle action.
    /// This ONLY gets called to validate the self plugin
    fn validate_create(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the update lifecycle action.
    /// This gets called on all existing plugins when an asset or collection is updated.
    fn validate_update(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the update_plugin lifecycle action.
    /// This gets called on all existing plugins when a plugin is updated.
    fn validate_update_plugin(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the burn lifecycle action.
    /// This gets called on all existing plugins when an asset is burned.
    fn validate_burn(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the transfer lifecycle action.
    /// This gets called on all existing plugins when an asset is transferred.
    fn validate_transfer(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the compress lifecycle action.
    fn validate_compress(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the decompress lifecycle action.
    fn validate_decompress(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the update_plugin lifecycle action.
    fn validate_update_external_plugin_adapter(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }
}

/// This function iterates through all plugin checks passed in and performs the validation
/// by deserializing and calling validate on the plugin.
/// The STRONGEST result is returned.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn validate_plugin_checks<'a, 'b>(
    key: Key,
    // accounts: &'a [AccountInfo<'a>],
    checks: &BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    // authority: &'a AccountInfo<'a>,
    // new_owner: Option<&'a AccountInfo<'a>>,
    // new_asset_authority: Option<&UpdateAuthority>,
    // new_collection_authority: Option<&Pubkey>,
    // new_plugin: Option<&Plugin>,
    // asset: Option<&'a AccountInfo<'a>>,
    // collection: Option<&'a AccountInfo<'a>>,
    common: &'b AssetValidationCommon<'a>,
    ctx: &'b AssetValidationContext<'a>,
    resolved_authorities: &[Authority],
    plugin_validate_fp: fn(
        &Plugin,
        &PluginValidationContext,
        &AssetValidationCommon<'a>,
        &AssetValidationContext<'a>,
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
                Key::CollectionV1 => match load_key(common.asset_info, 0)? {
                    Key::AssetV1 => common
                        .collection_info
                        .ok_or(MplCoreError::InvalidCollection)?,
                    Key::CollectionV1 => common.asset_info,
                    _ => unreachable!(),
                },
                Key::AssetV1 => common.asset_info,
                _ => unreachable!(),
            };

            let validation_ctx = PluginValidationContext {
                //     accounts,
                //     asset_info: asset,
                //     collection_info: collection,
                self_authority: &registry_record.authority,
                //     authority_info: authority,
                resolved_authorities: Some(resolved_authorities),
                //     new_owner,
                //     new_asset_authority,
                //     new_collection_authority,
                //     target_plugin: new_plugin,
            };

            let result = plugin_validate_fp(
                &Plugin::load(account, registry_record.offset)?,
                &validation_ctx,
                common,
                ctx,
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
pub(crate) fn validate_external_plugin_adapter_checks(
    key: Key,
    // accounts: &'a [AccountInfo<'a>],
    external_checks: &BTreeMap<
        ExternalPluginAdapterKey,
        (Key, ExternalCheckResultBits, ExternalRegistryRecord),
    >,
    // authority: &'a AccountInfo<'a>,
    // new_owner: Option<&'a AccountInfo<'a>>,
    // new_asset_authority: Option<&UpdateAuthority>,
    // new_collection_authority: Option<&Pubkey>,
    // new_plugin: Option<&Plugin>,
    // asset: Option<&'a AccountInfo<'a>>,
    // collection: Option<&'a AccountInfo<'a>>,
    common: &AssetValidationCommon,
    ctx: &AssetValidationContext,
    resolved_authorities: &[Authority],
    external_plugin_adapter_validate_fp: fn(
        &ExternalPluginAdapter,
        &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
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
                Key::CollectionV1 => common
                    .collection_info
                    .ok_or(MplCoreError::InvalidCollection)?,
                Key::AssetV1 => common.asset_info,
                _ => unreachable!(),
            };

            let validation_ctx = PluginValidationContext {
                // accounts,
                // asset_info: asset,
                // collection_info: collection,
                self_authority: &external_registry_record.authority,
                // authority_info: authority,
                resolved_authorities: Some(resolved_authorities),
                // new_owner,
                // new_asset_authority,
                // new_collection_authority,
                // target_plugin: new_plugin,
            };

            let result = external_plugin_adapter_validate_fp(
                &ExternalPluginAdapter::load(account, external_registry_record.offset)?,
                &validation_ctx,
                common,
                ctx,
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
