use borsh::{BorshDeserialize, BorshSerialize};
use modular_bitfield::{bitfield, specifiers::B29};
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use std::collections::BTreeMap;

use crate::{
    error::MplCoreError,
    plugins::{
        ExternalPluginAdapter, ExternalPluginAdapterKey, ExternalRegistryRecord, Plugin,
        PluginType, RegistryRecord,
    },
    state::{Authority, DataBlob, Key, UpdateAuthority},
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

impl DataBlob for ExternalCheckResult {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

impl ExternalCheckResult {
    const BASE_LEN: usize = 4; // u32 flags

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
            PluginType::BubblegumV2 => CheckResult::CanReject,
            PluginType::PermanentFreezeExecute => CheckResult::CanReject,
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
            PluginType::BubblegumV2 => CheckResult::CanReject,
            PluginType::PermanentFreezeExecute => CheckResult::CanReject,
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
            PluginType::Groups => CheckResult::CanReject,
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

    /// Check permissions for the execute lifecycle event.
    pub fn check_execute(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            PluginType::FreezeExecute => CheckResult::CanReject,
            PluginType::PermanentFreezeExecute => CheckResult::CanReject,
            _ => CheckResult::None,
        }
    }

    /// Check permissions for the add external plugin adapter lifecycle event.
    pub fn check_add_external_plugin_adapter(plugin_type: &PluginType) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            PluginType::BubblegumV2 => CheckResult::CanReject,
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
        plugin.inner().validate_add_plugin(ctx)
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

        plugin.inner().validate_remove_plugin(ctx)
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

        plugin.inner().validate_approve_plugin_authority(ctx)
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
            solana_program::msg!("{}:{}:Base:Approved", std::file!(), std::line!());
            ValidationResult::Approved
        } else {
            ValidationResult::Pass
        };

        let result = plugin.inner().validate_revoke_plugin_authority(ctx)?;

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
        plugin.inner().validate_create(ctx)
    }

    /// Route the validation of the update action to the appropriate plugin.
    pub(crate) fn validate_update(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        plugin.inner().validate_update(ctx)
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
        // If the authority is right for the asset performing the validation.
        let base_result = if resolved_authorities.contains(ctx.self_authority)
        // And the target plugin is also the one being updated (i.e. self).
        && ctx.target_plugin.is_some()
        && PluginType::from(ctx.target_plugin.unwrap()) == PluginType::from(plugin)
        {
            solana_program::msg!("{}:{}:Base:Approved", std::file!(), std::line!());
            ValidationResult::Approved
        } else {
            ValidationResult::Pass
        };

        let result = plugin.inner().validate_update_plugin(ctx)?;

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
        plugin.inner().validate_burn(ctx)
    }

    /// Route the validation of the transfer action to the appropriate plugin.
    pub(crate) fn validate_transfer(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        plugin.inner().validate_transfer(ctx)
    }

    /// Route the validation of the compress action to the appropriate plugin.
    pub(crate) fn validate_compress(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        plugin.inner().validate_compress(ctx)
    }

    /// Route the validation of the decompress action to the appropriate plugin.
    pub(crate) fn validate_decompress(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        plugin.inner().validate_decompress(ctx)
    }

    /// Validate the execute lifecycle event.
    pub(crate) fn validate_execute(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        plugin.inner().validate_execute(ctx)
    }

    /// Validate the add external plugin adapter lifecycle event.
    pub(crate) fn validate_add_external_plugin_adapter(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        plugin.inner().validate_add_external_plugin_adapter(ctx)
    }

    /// Validate the remove plugin lifecycle event.
    pub(crate) fn validate_remove_external_plugin_adapter(
        plugin: &Plugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        plugin.inner().validate_remove_external_plugin_adapter(ctx)
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

#[allow(dead_code)]
/// The required context for a plugin validation.
pub(crate) struct PluginValidationContext<'a, 'b> {
    /// This list of all the accounts passed into the instruction.
    pub accounts: &'a [AccountInfo<'a>],
    /// The asset account.
    pub asset_info: Option<&'a AccountInfo<'a>>,
    /// The collection account.
    pub collection_info: Option<&'a AccountInfo<'a>>,
    /// The key of the account the current (self) plugin lives on, i.e. whether
    /// it is an asset plugin (`Key::AssetV1`) or a collection plugin
    /// (`Key::CollectionV1`). This lets plugins distinguish a plugin that lives
    /// on the lifecycle target from one inherited from a parent collection.
    pub self_key: Key,
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
    /// The authority of the target plugin.
    pub target_plugin_authority: Option<&'b Authority>,
    /// The plugin being acted upon with new data from the ix if any. This None for create.
    pub target_external_plugin: Option<&'b ExternalPluginAdapter>,
    /// The authority of the target plugin.
    pub target_external_plugin_authority: Option<&'b Authority>,
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

    /// Validate the execute lifecycle action.
    fn validate_execute(
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
    accounts: &'a [AccountInfo<'a>],
    checks: &BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    authority: &'a AccountInfo<'a>,
    new_owner: Option<&'a AccountInfo<'a>>,
    new_asset_authority: Option<&UpdateAuthority>,
    new_collection_authority: Option<&Pubkey>,
    new_plugin: Option<&Plugin>,
    new_plugin_authority: Option<&Authority>,
    new_external_plugin: Option<&ExternalPluginAdapter>,
    new_external_plugin_authority: Option<&Authority>,
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
        if matches!(
            check_result,
            CheckResult::CanApprove | CheckResult::CanReject
        ) {
            let account = match check_key {
                Key::CollectionV1 => collection.ok_or(MplCoreError::InvalidCollection)?,
                Key::AssetV1 => asset.ok_or(MplCoreError::InvalidAsset)?,
                _ => unreachable!(),
            };

            let validation_ctx = PluginValidationContext {
                accounts,
                asset_info: asset,
                collection_info: collection,
                self_key: *check_key,
                self_authority: &registry_record.authority,
                authority_info: authority,
                resolved_authorities: Some(resolved_authorities),
                new_owner,
                new_asset_authority,
                new_collection_authority,
                target_plugin: new_plugin,
                target_plugin_authority: new_plugin_authority,
                target_external_plugin: new_external_plugin,
                target_external_plugin_authority: new_external_plugin_authority,
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
    new_plugin_authority: Option<&Authority>,
    new_external_plugin: Option<&ExternalPluginAdapter>,
    new_external_plugin_authority: Option<&Authority>,
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
        if check_result.can_listen() || check_result.can_approve() || check_result.can_reject() {
            let account = match check_key {
                Key::CollectionV1 => collection.ok_or(MplCoreError::InvalidCollection)?,
                Key::AssetV1 => asset.ok_or(MplCoreError::InvalidAsset)?,
                _ => unreachable!(),
            };

            let validation_ctx = PluginValidationContext {
                accounts,
                asset_info: asset,
                collection_info: collection,
                self_key: *check_key,
                self_authority: &external_registry_record.authority,
                authority_info: authority,
                resolved_authorities: Some(resolved_authorities),
                new_owner,
                new_asset_authority,
                new_collection_authority,
                target_plugin: new_plugin,
                target_plugin_authority: new_plugin_authority,
                target_external_plugin: new_external_plugin,
                target_external_plugin_authority: new_external_plugin_authority,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_external_check_result_size() {
        let fixture = ExternalCheckResult { flags: 0 };
        let serialized = borsh::to_vec(&fixture).unwrap();
        assert_eq!(
            serialized.len(),
            fixture.len(),
            "Serialized {:?} should match size returned by len()",
            fixture
        );
    }
}

#[cfg(test)]
mod validation_tests {
    use {
        super::*,
        crate::plugins::{
            test_ctx::{default_ctx, FakeAccount},
            AddBlocker, Attributes, Autograph, BubblegumV2, BurnDelegate, Edition, FreezeDelegate,
            FreezeExecute, Groups, ImmutableMetadata, MasterEdition, PermanentBurnDelegate,
            PermanentFreezeDelegate, PermanentFreezeExecute, PermanentTransferDelegate, Royalties,
            RuleSet, TransferDelegate, UpdateDelegate, VerifiedCreators,
        },
        strum::IntoEnumIterator,
    };

    /// One value of every `Plugin` variant, in `PluginType` order.
    fn every_plugin() -> Vec<Plugin> {
        vec![
            Plugin::Royalties(Royalties {
                basis_points: 0,
                creators: vec![],
                rule_set: RuleSet::None,
            }),
            Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
            Plugin::BurnDelegate(BurnDelegate {}),
            Plugin::TransferDelegate(TransferDelegate {}),
            Plugin::UpdateDelegate(UpdateDelegate {
                additional_delegates: vec![],
            }),
            Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: false }),
            Plugin::Attributes(Attributes {
                attribute_list: vec![],
            }),
            Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
            Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
            Plugin::Edition(Edition { number: 0 }),
            Plugin::MasterEdition(MasterEdition {
                max_supply: None,
                name: None,
                uri: None,
            }),
            Plugin::AddBlocker(AddBlocker {}),
            Plugin::ImmutableMetadata(ImmutableMetadata {}),
            Plugin::VerifiedCreators(VerifiedCreators { signatures: vec![] }),
            Plugin::Autograph(Autograph { signatures: vec![] }),
            Plugin::BubblegumV2(BubblegumV2 {}),
            Plugin::FreezeExecute(FreezeExecute { frozen: false }),
            Plugin::PermanentFreezeExecute(PermanentFreezeExecute { frozen: false }),
            Plugin::Groups(Groups { groups: vec![] }),
        ]
    }

    fn core_err<T>(result: Result<T, ProgramError>) -> MplCoreError {
        match result {
            Err(ProgramError::Custom(code)) => {
                num_traits::FromPrimitive::from_u32(code).expect("an MplCoreError code")
            }
            Err(other) => panic!("expected a custom program error, got {other:?}"),
            Ok(_) => panic!("expected an error"),
        }
    }

    // -----------------------------------------------------------------------
    // External check result bitfield
    // -----------------------------------------------------------------------

    #[test]
    fn external_check_result_bits_round_trip() {
        for flags in 0u32..8 {
            let result = ExternalCheckResult { flags };
            let bits = ExternalCheckResultBits::from(result);
            assert_eq!(bits.can_listen(), flags & 0x1 != 0);
            assert_eq!(bits.can_approve(), flags & 0x2 != 0);
            assert_eq!(bits.can_reject(), flags & 0x4 != 0);
            assert_eq!(
                ExternalCheckResult::from(bits),
                result,
                "flags {flags} should round trip"
            );
        }

        // The unused 29 bits are preserved in both directions.
        let wide = ExternalCheckResult { flags: 0xFFFF_FFFF };
        let bits = ExternalCheckResultBits::from(wide);
        assert_eq!(bits.empty_bits(), (1 << 29) - 1);
        assert_eq!(ExternalCheckResult::from(bits), wide);

        // The builder setters (the program never calls them).
        let built = ExternalCheckResultBits::new()
            .with_can_listen(true)
            .with_can_approve(true)
            .with_can_reject(true)
            .with_empty_bits(0);
        assert_eq!(
            ExternalCheckResult::from(built),
            ExternalCheckResult { flags: 0x7 }
        );

        // The mutating setters, including the checked ones the program never
        // calls.
        let mut bits = ExternalCheckResultBits::new();
        bits.set_can_listen(true);
        bits.set_can_approve(true);
        bits.set_can_reject(true);
        assert_eq!(
            ExternalCheckResult::from(bits),
            ExternalCheckResult { flags: 0x7 }
        );
        bits.set_can_listen_checked(false).unwrap();
        bits.set_can_approve_checked(false).unwrap();
        bits.set_can_reject_checked(false).unwrap();
        bits.set_empty_bits_checked(1).unwrap();
        assert_eq!(
            ExternalCheckResult::from(bits),
            ExternalCheckResult { flags: 0x8 }
        );
        assert!(
            bits.set_empty_bits_checked(1 << 29).is_err(),
            "a value wider than 29 bits must be refused"
        );

        assert_eq!(
            ExternalCheckResult::none(),
            ExternalCheckResult { flags: 0 }
        );
        assert_eq!(
            ExternalCheckResult::can_reject_only(),
            ExternalCheckResult { flags: 0x4 }
        );
    }

    /// `CompressV1` / `DecompressV1` return `NotAvailable` before validation,
    /// so these routers and the trait defaults behind them are dead on-chain.
    #[test]
    fn compress_and_decompress_routers_and_trait_defaults() {
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let authority = Authority::UpdateAuthority;
        let ctx = default_ctx(&[], &signer_info, &authority);
        let target = inert();

        for plugin in every_plugin() {
            assert_eq!(
                Plugin::validate_compress(&plugin, &ctx).unwrap(),
                ValidationResult::Pass,
                "{:?}::validate_compress",
                PluginType::from(&plugin)
            );
            assert_eq!(
                Plugin::validate_decompress(&plugin, &ctx).unwrap(),
                ValidationResult::Pass,
                "{:?}::validate_decompress",
                PluginType::from(&plugin)
            );
        }

        // The `PluginValidation` default bodies, reached through a plugin that
        // overrides none of them.
        let attributes = inert();
        let inner = attributes.inner();
        let mut ctx_with_target = default_ctx(&[], &signer_info, &authority);
        ctx_with_target.target_plugin = Some(&target);
        for result in [
            inner.validate_add_plugin(&ctx_with_target),
            inner.validate_remove_plugin(&ctx_with_target),
            inner.validate_approve_plugin_authority(&ctx_with_target),
            inner.validate_revoke_plugin_authority(&ctx_with_target),
            inner.validate_create(&ctx),
            inner.validate_update(&ctx),
            inner.validate_update_plugin(&ctx_with_target),
            inner.validate_burn(&ctx),
            inner.validate_transfer(&ctx),
            inner.validate_compress(&ctx),
            inner.validate_decompress(&ctx),
            inner.validate_execute(&ctx),
            inner.validate_add_external_plugin_adapter(&ctx),
            inner.validate_remove_external_plugin_adapter(&ctx),
            inner.validate_update_external_plugin_adapter(&ctx),
        ] {
            assert_eq!(
                result.unwrap(),
                ValidationResult::Pass,
                "every PluginValidation default body abstains"
            );
        }

        // The remaining `Plugin::validate_*` routers.
        assert_eq!(
            Plugin::validate_execute(&attributes, &ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            Plugin::validate_add_external_plugin_adapter(&attributes, &ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            Plugin::validate_remove_external_plugin_adapter(&attributes, &ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            Plugin::validate_create(&attributes, &ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            Plugin::validate_update(&attributes, &ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            Plugin::validate_burn(&attributes, &ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            Plugin::validate_add_plugin(&attributes, &ctx_with_target).unwrap(),
            ValidationResult::Pass
        );
    }

    #[test]
    fn external_validation_result_maps_onto_validation_result() {
        assert_eq!(
            ValidationResult::from(ExternalValidationResult::Approved),
            ValidationResult::Approved
        );
        assert_eq!(
            ValidationResult::from(ExternalValidationResult::Rejected),
            ValidationResult::Rejected
        );
        assert_eq!(
            ValidationResult::from(ExternalValidationResult::Pass),
            ValidationResult::Pass
        );
    }

    // -----------------------------------------------------------------------
    // Check tables
    // -----------------------------------------------------------------------

    /// Asserts that `table` returns `CanApprove` exactly for `can_approve`,
    /// `CanReject` exactly for `can_reject`, and `default` for everything else.
    fn assert_table(
        name: &str,
        table: fn(&PluginType) -> CheckResult,
        can_approve: &[PluginType],
        can_reject: &[PluginType],
        default: CheckResult,
    ) {
        for plugin_type in PluginType::iter() {
            let expected = if can_approve.contains(&plugin_type) {
                CheckResult::CanApprove
            } else if can_reject.contains(&plugin_type) {
                CheckResult::CanReject
            } else {
                default
            };
            assert_eq!(
                table(&plugin_type),
                expected,
                "{name}({plugin_type:?}) should be {expected:?}"
            );
        }
    }

    #[test]
    fn plugin_type_check_tables_match_the_documented_matrix() {
        assert_table(
            "check_add_plugin",
            PluginType::check_add_plugin,
            &[PluginType::UpdateDelegate],
            &[
                PluginType::AddBlocker,
                PluginType::Royalties,
                PluginType::PermanentFreezeDelegate,
                PluginType::PermanentTransferDelegate,
                PluginType::PermanentBurnDelegate,
                PluginType::Edition,
                PluginType::Autograph,
                PluginType::VerifiedCreators,
                PluginType::BubblegumV2,
                PluginType::PermanentFreezeExecute,
            ],
            CheckResult::None,
        );

        // Every other type defaults to `CanReject` here, because a plugin with
        // `Authority::None` must be able to refuse its own removal.
        assert_table(
            "check_remove_plugin",
            PluginType::check_remove_plugin,
            &[PluginType::UpdateDelegate],
            &[],
            CheckResult::CanReject,
        );

        for (name, table) in [
            (
                "check_update_plugin",
                PluginType::check_update_plugin as fn(&PluginType) -> CheckResult,
            ),
            (
                "check_approve_plugin_authority",
                PluginType::check_approve_plugin_authority,
            ),
            (
                "check_revoke_plugin_authority",
                PluginType::check_revoke_plugin_authority,
            ),
        ] {
            assert_table(name, table, &[], &[], CheckResult::CanApprove);
        }

        assert_table(
            "check_create",
            PluginType::check_create,
            &[PluginType::UpdateDelegate],
            &[
                PluginType::Royalties,
                PluginType::Autograph,
                PluginType::VerifiedCreators,
            ],
            CheckResult::None,
        );
        assert_table(
            "check_update",
            PluginType::check_update,
            &[PluginType::UpdateDelegate],
            &[PluginType::ImmutableMetadata],
            CheckResult::None,
        );
        assert_table(
            "check_burn",
            PluginType::check_burn,
            &[PluginType::BurnDelegate, PluginType::PermanentBurnDelegate],
            &[
                PluginType::FreezeDelegate,
                PluginType::PermanentFreezeDelegate,
                PluginType::Groups,
            ],
            CheckResult::None,
        );
        assert_table(
            "check_transfer",
            PluginType::check_transfer,
            &[
                PluginType::TransferDelegate,
                PluginType::PermanentTransferDelegate,
            ],
            &[
                PluginType::Royalties,
                PluginType::FreezeDelegate,
                PluginType::PermanentFreezeDelegate,
            ],
            CheckResult::None,
        );
        assert_table(
            "check_execute",
            PluginType::check_execute,
            &[],
            &[
                PluginType::FreezeExecute,
                PluginType::PermanentFreezeExecute,
            ],
            CheckResult::None,
        );
        assert_table(
            "check_add_external_plugin_adapter",
            PluginType::check_add_external_plugin_adapter,
            &[],
            &[PluginType::BubblegumV2],
            CheckResult::None,
        );

        // `CompressV1` / `DecompressV1` return `NotAvailable` before reaching
        // validation, and the remove/update adapter events consult no plugin.
        for (name, table) in [
            (
                "check_compress",
                PluginType::check_compress as fn(&PluginType) -> CheckResult,
            ),
            ("check_decompress", PluginType::check_decompress),
            (
                "check_remove_external_plugin_adapter",
                PluginType::check_remove_external_plugin_adapter,
            ),
            (
                "check_update_external_plugin_adapter",
                PluginType::check_update_external_plugin_adapter,
            ),
        ] {
            assert_table(name, table, &[], &[], CheckResult::None);
        }
    }

    // -----------------------------------------------------------------------
    // Dispatch tables
    // -----------------------------------------------------------------------

    #[test]
    fn plugin_dispatch_tables_cover_every_variant() {
        let plugins = every_plugin();
        assert_eq!(
            plugins.len(),
            PluginType::iter().count(),
            "every_plugin() must hold one value per PluginType"
        );

        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let self_authority = Authority::None;

        for (plugin, plugin_type) in plugins.iter().zip(PluginType::iter()) {
            assert_eq!(
                PluginType::from(plugin),
                plugin_type,
                "From<&Plugin> for PluginType is out of order at {plugin_type:?}"
            );
            assert_eq!(
                plugin.manager(),
                plugin_type.manager(),
                "Plugin::manager must agree with PluginType::manager"
            );

            // `inner()` routes to the plugin's own trait object; every plugin
            // either abstains or rejects a transfer from a default context.
            let ctx = default_ctx(&[], &signer_info, &self_authority);
            let result = plugin.inner().validate_transfer(&ctx);
            match plugin {
                // Royalties needs a new owner.
                Plugin::Royalties(_) => {
                    assert_eq!(core_err(result), MplCoreError::MissingNewOwner);
                }
                _ => {
                    assert!(
                        matches!(
                            result,
                            Ok(ValidationResult::Pass) | Ok(ValidationResult::Rejected)
                        ),
                        "{plugin_type:?}::validate_transfer returned {result:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn plugin_save_and_load_round_trip_and_report_errors() {
        let plugin = inert();
        let bytes = borsh::to_vec(&plugin).unwrap();

        let mut account = FakeAccount::with_data(vec![0u8; bytes.len() + 4]);
        let info = account.info();
        plugin.save(&info, 4).unwrap();
        assert_eq!(Plugin::load(&info, 4).unwrap(), plugin);

        // A corrupted discriminator is a clean `DeserializationError`.
        info.data.borrow_mut()[4] = 0xFE;
        assert_eq!(
            Plugin::load(&info, 4).unwrap_err(),
            MplCoreError::DeserializationError.into()
        );

        // Writing into a buffer that is too short is a `SerializationError`;
        // the program always reallocates first, so this arm is defensive.
        let mut small = FakeAccount::with_data(vec![0u8; 1]);
        let small_info = small.info();
        assert_eq!(
            Plugin::save(&plugin, &small_info, 0).unwrap_err(),
            MplCoreError::SerializationError.into()
        );
    }

    #[test]
    fn plugin_type_manager_assigns_owner_only_to_owner_managed_plugins() {
        let owner_managed = [
            PluginType::FreezeDelegate,
            PluginType::BurnDelegate,
            PluginType::TransferDelegate,
            PluginType::Autograph,
            PluginType::FreezeExecute,
        ];
        for plugin_type in PluginType::iter() {
            let expected = if owner_managed.contains(&plugin_type) {
                Authority::Owner
            } else if plugin_type == PluginType::BubblegumV2 {
                Authority::Address {
                    address: mpl_bubblegum::ID,
                }
            } else {
                Authority::UpdateAuthority
            };
            assert_eq!(
                plugin_type.manager(),
                expected,
                "{plugin_type:?}::manager()"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Wrappers and the result-combination matrix
    // -----------------------------------------------------------------------

    /// A plugin that has no `validate_*` overrides at all, so the wrapper's
    /// base result is the only thing in play.
    fn inert() -> Plugin {
        Plugin::Attributes(Attributes {
            attribute_list: vec![],
        })
    }

    #[test]
    fn validate_remove_plugin_refuses_an_authority_none_self_target() {
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let none = Authority::None;
        let target = inert();

        let mut ctx = default_ctx(&[], &signer_info, &none);
        ctx.target_plugin = Some(&target);
        assert_eq!(
            Plugin::validate_remove_plugin(&inert(), &ctx).unwrap(),
            ValidationResult::Rejected
        );

        // A different plugin type with `Authority::None` does not refuse the
        // removal of somebody else.
        let other = Plugin::Edition(Edition { number: 1 });
        assert_eq!(
            Plugin::validate_remove_plugin(&other, &ctx).unwrap(),
            ValidationResult::Pass
        );
    }

    #[test]
    fn validate_approve_plugin_authority_refuses_to_redelegate() {
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let target = inert();

        // The target's authority is already delegated away from its manager.
        let delegated = Authority::Address {
            address: Pubkey::new_unique(),
        };
        let mut ctx = default_ctx(&[], &signer_info, &delegated);
        ctx.target_plugin = Some(&target);
        assert_eq!(
            core_err(Plugin::validate_approve_plugin_authority(&inert(), &ctx)),
            MplCoreError::CannotRedelegate
        );

        // Still at its manager: the plugin's own validator runs (and abstains).
        let manager = Authority::UpdateAuthority;
        let mut ctx = default_ctx(&[], &signer_info, &manager);
        ctx.target_plugin = Some(&target);
        assert_eq!(
            Plugin::validate_approve_plugin_authority(&inert(), &ctx).unwrap(),
            ValidationResult::Pass
        );

        // No target at all: unreachable from the processors.
        let ctx = default_ctx(&[], &signer_info, &manager);
        assert_eq!(
            core_err(Plugin::validate_approve_plugin_authority(&inert(), &ctx)),
            MplCoreError::InvalidPlugin
        );
    }

    #[test]
    fn validate_revoke_plugin_authority_base_result() {
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let target = inert();
        let manager = Authority::UpdateAuthority;
        let resolved = [Authority::UpdateAuthority];

        // Self target, signer resolves to the record authority: approved.
        let mut ctx = default_ctx(&[], &signer_info, &manager);
        ctx.target_plugin = Some(&target);
        ctx.resolved_authorities = Some(&resolved);
        assert_eq!(
            Plugin::validate_revoke_plugin_authority(&inert(), &ctx).unwrap(),
            ValidationResult::Approved
        );

        // The signer does not resolve to it: pass.
        let unrelated = [Authority::Owner];
        ctx.resolved_authorities = Some(&unrelated);
        assert_eq!(
            Plugin::validate_revoke_plugin_authority(&inert(), &ctx).unwrap(),
            ValidationResult::Pass
        );

        // `Authority::None` cannot be revoked.
        let none = Authority::None;
        let mut ctx = default_ctx(&[], &signer_info, &none);
        ctx.target_plugin = Some(&target);
        ctx.resolved_authorities = Some(&resolved);
        assert_eq!(
            Plugin::validate_revoke_plugin_authority(&inert(), &ctx).unwrap(),
            ValidationResult::Rejected
        );

        // No target: unreachable from the processors.
        let ctx = default_ctx(&[], &signer_info, &manager);
        assert_eq!(
            core_err(Plugin::validate_revoke_plugin_authority(&inert(), &ctx)),
            MplCoreError::InvalidPlugin
        );
    }

    #[test]
    fn validate_update_plugin_combines_the_base_and_inner_results() {
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let manager = Authority::UpdateAuthority;

        // No resolved authorities: unreachable from the processors.
        let ctx = default_ctx(&[], &signer_info, &manager);
        assert_eq!(
            core_err(Plugin::validate_update_plugin(&inert(), &ctx)),
            MplCoreError::InvalidAuthority
        );

        // (base Approved, inner Pass): the base wins.
        let target = inert();
        let resolved = [Authority::UpdateAuthority];
        let mut ctx = default_ctx(&[], &signer_info, &manager);
        ctx.resolved_authorities = Some(&resolved);
        ctx.target_plugin = Some(&target);
        assert_eq!(
            Plugin::validate_update_plugin(&inert(), &ctx).unwrap(),
            ValidationResult::Approved
        );

        // (base Pass, inner Pass): the inner result is returned.
        let unrelated = [Authority::Owner];
        ctx.resolved_authorities = Some(&unrelated);
        assert_eq!(
            Plugin::validate_update_plugin(&inert(), &ctx).unwrap(),
            ValidationResult::Pass
        );

        // (base Approved, inner Approved): approved. `Autograph` approves a
        // self-consistent update from its own authority.
        let autograph = Plugin::Autograph(Autograph { signatures: vec![] });
        let owner = Authority::Owner;
        let owner_resolved = [Authority::Owner];
        let mut ctx = default_ctx(&[], &signer_info, &owner);
        ctx.resolved_authorities = Some(&owner_resolved);
        ctx.target_plugin = Some(&autograph);
        assert_eq!(
            Plugin::validate_update_plugin(&autograph, &ctx).unwrap(),
            ValidationResult::Approved
        );

        // The remaining arms of the match — anything involving `Rejected`,
        // `ForceApproved`, or an inner `ForceApproved` — cannot be produced by
        // any plugin in the crate: no `validate_update_plugin` implementation
        // returns `Rejected` (they return an error instead) or
        // `ForceApproved`, and the base result is only ever `Approved` or
        // `Pass`. Roadmap section 11 records the matrix as partly unreachable
        // on-chain for exactly this reason.
    }

    // -----------------------------------------------------------------------
    // Stateless reject / abstain arms of the small plugins
    // -----------------------------------------------------------------------

    #[test]
    fn add_blocker_only_lets_owner_managed_plugins_and_itself_through() {
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let authority = Authority::UpdateAuthority;
        let blocker = AddBlocker {};

        for (target, expected) in [
            (
                Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                ValidationResult::Pass,
            ),
            (Plugin::AddBlocker(AddBlocker {}), ValidationResult::Pass),
            (
                Plugin::Attributes(Attributes {
                    attribute_list: vec![],
                }),
                ValidationResult::Rejected,
            ),
        ] {
            let mut ctx = default_ctx(&[], &signer_info, &authority);
            ctx.target_plugin = Some(&target);
            assert_eq!(
                blocker.validate_add_plugin(&ctx).unwrap(),
                expected,
                "AddBlocker on {:?}",
                PluginType::from(&target)
            );
        }

        // No target: rejects (unreachable from the processors).
        let ctx = default_ctx(&[], &signer_info, &authority);
        assert_eq!(
            blocker.validate_add_plugin(&ctx).unwrap(),
            ValidationResult::Rejected
        );
    }

    #[test]
    fn immutable_metadata_always_rejects_an_update() {
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let authority = Authority::None;
        let ctx = default_ctx(&[], &signer_info, &authority);
        assert_eq!(
            ImmutableMetadata {}.validate_update(&ctx).unwrap(),
            ValidationResult::Rejected
        );
    }

    #[test]
    fn permanent_and_edition_plugins_refuse_to_be_added_or_removed() {
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let authority = Authority::UpdateAuthority;
        let unrelated = Plugin::Attributes(Attributes {
            attribute_list: vec![],
        });

        // (plugin under test, its own Plugin value, blocks its own removal)
        let cases: Vec<(Box<dyn PluginValidation>, Plugin, bool)> = vec![
            (
                Box::new(PermanentBurnDelegate {}),
                Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
                false,
            ),
            (
                Box::new(PermanentTransferDelegate {}),
                Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
                false,
            ),
            (
                Box::new(PermanentFreezeDelegate { frozen: false }),
                Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: false }),
                false,
            ),
            (
                Box::new(PermanentFreezeExecute { frozen: false }),
                Plugin::PermanentFreezeExecute(PermanentFreezeExecute { frozen: false }),
                false,
            ),
            (
                Box::new(Edition { number: 1 }),
                Plugin::Edition(Edition { number: 1 }),
                true,
            ),
            (
                Box::new(BubblegumV2 {}),
                Plugin::BubblegumV2(BubblegumV2 {}),
                true,
            ),
        ];

        for (plugin, own, blocks_removal) in cases {
            let name = PluginType::from(&own);

            let mut ctx = default_ctx(&[], &signer_info, &authority);
            ctx.target_plugin = Some(&own);
            assert_eq!(
                plugin.validate_add_plugin(&ctx).unwrap(),
                ValidationResult::Rejected,
                "{name:?} must refuse to be added after creation"
            );
            assert_eq!(
                plugin.validate_remove_plugin(&ctx).unwrap(),
                if blocks_removal {
                    ValidationResult::Rejected
                } else {
                    ValidationResult::Pass
                },
                "{name:?} removal of itself while unfrozen"
            );

            let mut ctx = default_ctx(&[], &signer_info, &authority);
            ctx.target_plugin = Some(&unrelated);
            assert_eq!(
                plugin.validate_add_plugin(&ctx).unwrap(),
                ValidationResult::Pass,
                "{name:?} must not block unrelated adds"
            );

            // No target at all (unreachable from the processors).
            let ctx = default_ctx(&[], &signer_info, &authority);
            assert_eq!(
                plugin.validate_add_plugin(&ctx).unwrap(),
                ValidationResult::Pass,
                "{name:?} with no target abstains"
            );
        }
    }

    #[test]
    fn bubblegum_v2_allow_list_and_adapter_arms() {
        let mut signer = FakeAccount::wallet();
        let mut asset = FakeAccount::wallet();
        let signer_info = signer.info();
        let asset_info = asset.info();
        let authority = Authority::Address {
            address: mpl_bubblegum::ID,
        };

        let allow_listed = Plugin::Attributes(Attributes {
            attribute_list: vec![],
        });
        let not_allow_listed = Plugin::ImmutableMetadata(ImmutableMetadata {});

        // On the collection itself (`asset_info` is `None`).
        for (target, expected) in [
            (&allow_listed, ValidationResult::Pass),
            (&not_allow_listed, ValidationResult::Rejected),
        ] {
            let mut ctx = default_ctx(&[], &signer_info, &authority);
            ctx.target_plugin = Some(target);
            assert_eq!(BubblegumV2 {}.validate_add_plugin(&ctx).unwrap(), expected);
        }

        // On a member asset nothing is restricted.
        let mut ctx = default_ctx(&[], &signer_info, &authority);
        ctx.asset_info = Some(&asset_info);
        ctx.target_plugin = Some(&not_allow_listed);
        assert_eq!(
            BubblegumV2 {}.validate_add_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );

        // External adapters: refused on the collection, allowed on an asset.
        let ctx = default_ctx(&[], &signer_info, &authority);
        assert_eq!(
            BubblegumV2 {}
                .validate_add_external_plugin_adapter(&ctx)
                .unwrap(),
            ValidationResult::Rejected
        );
        let mut ctx = default_ctx(&[], &signer_info, &authority);
        ctx.asset_info = Some(&asset_info);
        assert_eq!(
            BubblegumV2 {}
                .validate_add_external_plugin_adapter(&ctx)
                .unwrap(),
            ValidationResult::Pass
        );
    }

    #[test]
    fn freeze_plugins_gate_on_the_frozen_flag() {
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let owner = Authority::Owner;
        let resolved = [Authority::Owner];
        let unrelated_target = Plugin::Attributes(Attributes {
            attribute_list: vec![],
        });

        for frozen in [true, false] {
            let expected = if frozen {
                ValidationResult::Rejected
            } else {
                ValidationResult::Pass
            };

            // FreezeDelegate blocks burn and transfer while frozen.
            let freeze = FreezeDelegate { frozen };
            let ctx = default_ctx(&[], &signer_info, &owner);
            assert_eq!(freeze.validate_burn(&ctx).unwrap(), expected);
            assert_eq!(freeze.validate_transfer(&ctx).unwrap(), expected);

            // ... and the removal of *any* plugin while frozen.
            let mut ctx = default_ctx(&[], &signer_info, &owner);
            ctx.target_plugin = Some(&unrelated_target);
            assert_eq!(freeze.validate_remove_plugin(&ctx).unwrap(), expected);

            // FreezeExecute blocks execute while frozen, but only its own
            // removal.
            let freeze_execute = FreezeExecute { frozen };
            let ctx = default_ctx(&[], &signer_info, &owner);
            assert_eq!(freeze_execute.validate_execute(&ctx).unwrap(), expected);
            let mut ctx = default_ctx(&[], &signer_info, &owner);
            ctx.target_plugin = Some(&unrelated_target);
            assert_eq!(
                freeze_execute.validate_remove_plugin(&ctx).unwrap(),
                ValidationResult::Pass,
                "FreezeExecute only blocks its own removal"
            );
            let own = Plugin::FreezeExecute(FreezeExecute { frozen });
            let mut ctx = default_ctx(&[], &signer_info, &owner);
            ctx.target_plugin = Some(&own);
            assert_eq!(
                freeze_execute.validate_remove_plugin(&ctx).unwrap(),
                expected
            );

            // PermanentFreezeExecute has the same shape on execute.
            let permanent = PermanentFreezeExecute { frozen };
            let ctx = default_ctx(&[], &signer_info, &owner);
            assert_eq!(permanent.validate_execute(&ctx).unwrap(), expected);

            // Approve and revoke: both freeze plugins refuse while the *target*
            // is frozen; unfrozen, revoke approves for the record authority.
            for (target, approve, revoke) in [
                (
                    Plugin::FreezeDelegate(FreezeDelegate { frozen }),
                    expected.clone(),
                    if frozen {
                        ValidationResult::Rejected
                    } else {
                        ValidationResult::Approved
                    },
                ),
                (
                    unrelated_target.clone(),
                    ValidationResult::Pass,
                    ValidationResult::Pass,
                ),
            ] {
                let mut ctx = default_ctx(&[], &signer_info, &owner);
                ctx.resolved_authorities = Some(&resolved);
                ctx.target_plugin = Some(&target);
                assert_eq!(
                    freeze.validate_approve_plugin_authority(&ctx).unwrap(),
                    approve
                );
                assert_eq!(
                    freeze.validate_revoke_plugin_authority(&ctx).unwrap(),
                    revoke
                );
            }

            // An unrelated target leaves both execute-freeze plugins abstaining
            // on every authority and removal callback.
            let mut ctx = default_ctx(&[], &signer_info, &owner);
            ctx.resolved_authorities = Some(&resolved);
            ctx.target_plugin = Some(&unrelated_target);
            assert_eq!(
                freeze_execute
                    .validate_approve_plugin_authority(&ctx)
                    .unwrap(),
                ValidationResult::Pass
            );
            assert_eq!(
                freeze_execute
                    .validate_revoke_plugin_authority(&ctx)
                    .unwrap(),
                ValidationResult::Pass
            );
            assert_eq!(
                permanent.validate_remove_plugin(&ctx).unwrap(),
                ValidationResult::Pass
            );
            let permanent_target =
                Plugin::PermanentFreezeExecute(PermanentFreezeExecute { frozen });
            ctx.target_plugin = Some(&permanent_target);
            assert_eq!(permanent.validate_remove_plugin(&ctx).unwrap(), expected);
            assert_eq!(
                permanent.validate_add_plugin(&ctx).unwrap(),
                ValidationResult::Rejected
            );

            let freeze_execute_target = Plugin::FreezeExecute(FreezeExecute { frozen });
            let mut ctx = default_ctx(&[], &signer_info, &owner);
            ctx.resolved_authorities = Some(&resolved);
            ctx.target_plugin = Some(&freeze_execute_target);
            assert_eq!(
                freeze_execute
                    .validate_approve_plugin_authority(&ctx)
                    .unwrap(),
                expected
            );
            assert_eq!(
                freeze_execute
                    .validate_revoke_plugin_authority(&ctx)
                    .unwrap(),
                if frozen {
                    ValidationResult::Rejected
                } else {
                    ValidationResult::Approved
                }
            );

            // PermanentFreezeDelegate blocks burn, transfer and every removal.
            let permanent_freeze = PermanentFreezeDelegate { frozen };
            let ctx = default_ctx(&[], &signer_info, &owner);
            assert_eq!(permanent_freeze.validate_burn(&ctx).unwrap(), expected);
            assert_eq!(permanent_freeze.validate_transfer(&ctx).unwrap(), expected);
            let mut ctx = default_ctx(&[], &signer_info, &owner);
            ctx.target_plugin = Some(&unrelated_target);
            assert_eq!(
                permanent_freeze.validate_remove_plugin(&ctx).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn delegate_plugins_approve_only_their_own_record_authority() {
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let owner = Authority::Owner;

        for (resolved, expected) in [
            (
                vec![Authority::Owner],
                (ValidationResult::Approved, ValidationResult::ForceApproved),
            ),
            (
                vec![Authority::UpdateAuthority],
                (ValidationResult::Pass, ValidationResult::Pass),
            ),
        ] {
            let mut ctx = default_ctx(&[], &signer_info, &owner);
            ctx.resolved_authorities = Some(&resolved);

            assert_eq!(
                BurnDelegate {}.validate_burn(&ctx).unwrap(),
                expected.0,
                "BurnDelegate with {resolved:?}"
            );
            assert_eq!(
                TransferDelegate {}.validate_transfer(&ctx).unwrap(),
                expected.0,
                "TransferDelegate with {resolved:?}"
            );
            assert_eq!(
                PermanentBurnDelegate {}.validate_burn(&ctx).unwrap(),
                expected.1,
                "PermanentBurnDelegate with {resolved:?}"
            );
            assert_eq!(
                PermanentTransferDelegate {}
                    .validate_transfer(&ctx)
                    .unwrap(),
                expected.1,
                "PermanentTransferDelegate with {resolved:?}"
            );
        }

        // With no resolved authorities at all every delegate abstains.
        let ctx = default_ctx(&[], &signer_info, &owner);
        assert_eq!(
            BurnDelegate {}.validate_burn(&ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            TransferDelegate {}.validate_transfer(&ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            PermanentBurnDelegate {}.validate_burn(&ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            PermanentTransferDelegate {}
                .validate_transfer(&ctx)
                .unwrap(),
            ValidationResult::Pass
        );
    }
}
