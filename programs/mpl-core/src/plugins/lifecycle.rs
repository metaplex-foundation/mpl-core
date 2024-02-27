use solana_program::program_error::ProgramError;

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        BurnAccounts, CompressAccounts, CreateAccounts, DecompressAccounts, TransferAccounts,
        UpdateAccounts, UpdatePluginAccounts,
    },
    processor::{
        BurnArgs, CompressArgs, CreateArgs, DecompressArgs, TransferArgs, UpdateArgs,
        UpdatePluginArgs,
    },
    state::{Asset, Authority},
};

use super::{Plugin, PluginType};

/// Lifecycle permissions
/// Plugins use this field to indicate their permission to approve or deny
/// a lifecycle action.
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
    pub fn check_create(&self) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match self {
            PluginType::Collection => CheckResult::CanReject,
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny an update action.
    pub fn check_update(&self) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match self {
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny a burn action.
    pub fn check_burn(&self) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match self {
            PluginType::Freeze => CheckResult::CanReject,
            PluginType::Burn => CheckResult::CanApprove,
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny a transfer action.
    pub fn check_transfer(&self) -> CheckResult {
        match self {
            PluginType::Royalties => CheckResult::CanReject,
            PluginType::Freeze => CheckResult::CanReject,
            PluginType::Transfer => CheckResult::CanApprove,
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny a compress action.
    pub fn check_compress(&self) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match self {
            _ => CheckResult::None,
        }
    }

    /// Check if a plugin is permitted to approve or deny a decompress action.
    pub fn check_decompress(&self) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match self {
            _ => CheckResult::None,
        }
    }
}

impl Plugin {
    /// Route the validation of the create action to the appropriate plugin.
    pub fn validate_create(
        &self,
        ctx: &CreateAccounts,
        args: &CreateArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_create(ctx, args, authorities),
            Plugin::Freeze(freeze) => freeze.validate_create(ctx, args, authorities),
            Plugin::Burn(burn) => burn.validate_create(ctx, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_create(ctx, args, authorities),
            Plugin::Collection(collection) => collection.validate_create(ctx, args, authorities),
        }
    }

    /// Route the validation of the update action to the appropriate plugin.
    pub fn validate_update(
        &self,
        ctx: &UpdateAccounts,
        args: &UpdateArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_update(ctx, args, authorities),
            Plugin::Freeze(freeze) => freeze.validate_update(ctx, args, authorities),
            Plugin::Burn(burn) => burn.validate_update(ctx, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_update(ctx, args, authorities),
            Plugin::Collection(collection) => collection.validate_update(ctx, args, authorities),
        }
    }

    /// Route the validation of the update_plugin action to the appropriate plugin.
    /// There is no check for updating a plugin because the plugin itself MUST validate the change.
    pub fn validate_update_plugin(
        &self,
        asset: &Asset,
        ctx: &UpdatePluginAccounts,
        args: &UpdatePluginArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => {
                royalties.validate_update_plugin(asset, ctx, args, authorities)
            }
            Plugin::Freeze(freeze) => freeze.validate_update_plugin(asset, ctx, args, authorities),
            Plugin::Burn(burn) => burn.validate_update_plugin(asset, ctx, args, authorities),
            Plugin::Transfer(transfer) => {
                transfer.validate_update_plugin(asset, ctx, args, authorities)
            }
            Plugin::Collection(collection) => {
                collection.validate_update_plugin(asset, ctx, args, authorities)
            }
        }
    }

    /// Route the validation of the burn action to the appropriate plugin.
    pub fn validate_burn(
        &self,
        ctx: &BurnAccounts,
        args: &BurnArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_burn(ctx, args, authorities),
            Plugin::Freeze(freeze) => freeze.validate_burn(ctx, args, authorities),
            Plugin::Burn(burn) => burn.validate_burn(ctx, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_burn(ctx, args, authorities),
            Plugin::Collection(collection) => collection.validate_burn(ctx, args, authorities),
        }
    }

    /// Route the validation of the transfer action to the appropriate plugin.
    pub fn validate_transfer(
        &self,
        ctx: &TransferAccounts,
        args: &TransferArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_transfer(ctx, args, authorities),
            Plugin::Freeze(freeze) => freeze.validate_transfer(ctx, args, authorities),
            Plugin::Burn(burn) => burn.validate_transfer(ctx, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_transfer(ctx, args, authorities),
            Plugin::Collection(collection) => collection.validate_transfer(ctx, args, authorities),
        }
    }

    /// Route the validation of the compress action to the appropriate plugin.
    pub fn validate_compress(
        &self,
        ctx: &CompressAccounts,
        args: &CompressArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_compress(ctx, args, authorities),
            Plugin::Freeze(freeze) => freeze.validate_compress(ctx, args, authorities),
            Plugin::Burn(burn) => burn.validate_compress(ctx, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_compress(ctx, args, authorities),
            Plugin::Collection(collection) => collection.validate_compress(ctx, args, authorities),
        }
    }

    /// Route the validation of the decompress action to the appropriate plugin.
    pub fn validate_decompress(
        &self,
        ctx: &DecompressAccounts,
        args: &DecompressArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_decompress(ctx, args, authorities),
            Plugin::Freeze(freeze) => freeze.validate_decompress(ctx, args, authorities),
            Plugin::Burn(burn) => burn.validate_decompress(ctx, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_decompress(ctx, args, authorities),
            Plugin::Collection(collection) => {
                collection.validate_decompress(ctx, args, authorities)
            }
        }
    }
}

/// Lifecycle validations
/// Plugins utilize this to indicate whether they approve or reject a lifecycle action.
#[derive(Eq, PartialEq)]
pub enum ValidationResult {
    /// The plugin approves the lifecycle action.
    Approved,
    /// The plugin rejects the lifecycle action.
    Rejected,
    /// The plugin abstains from approving or rejecting the lifecycle action.
    Pass,
}

/// Plugin validation trait which is implemented by each plugin.
pub trait PluginValidation {
    /// Validate the create lifecycle action.
    fn validate_create(
        &self,
        ctx: &CreateAccounts,
        args: &CreateArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError>;

    /// Validate the update lifecycle action.
    fn validate_update(
        &self,
        ctx: &UpdateAccounts,
        args: &UpdateArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError>;

    /// Validate the update_plugin lifecycle action.
    fn validate_update_plugin(
        &self,
        asset: &Asset,
        ctx: &UpdatePluginAccounts,
        _args: &UpdatePluginArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        if (ctx.authority.key == &asset.owner && authorities.contains(&Authority::Owner))
            || (ctx.authority.key == &asset.update_authority
                && authorities.contains(&Authority::UpdateAuthority))
            || authorities.contains(&Authority::Pubkey {
                address: *ctx.authority.key,
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
        ctx: &BurnAccounts,
        args: &BurnArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError>;

    /// Validate the transfer lifecycle action.
    fn validate_transfer(
        &self,
        ctx: &TransferAccounts,
        args: &TransferArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError>;

    /// Validate the compress lifecycle action.
    fn validate_compress(
        &self,
        ctx: &CompressAccounts,
        args: &CompressArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError>;

    /// Validate the decompress lifecycle action.
    fn validate_decompress(
        &self,
        ctx: &DecompressAccounts,
        args: &DecompressArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError>;
}
