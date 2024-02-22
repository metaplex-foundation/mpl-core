use solana_program::program_error::ProgramError;

use crate::{
    error::MplAssetError,
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

// Lifecycle permissions
pub enum CheckResult {
    CanApprove,
    CanReject,
    None,
}

impl PluginType {
    pub fn check_create(&self) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match self {
            _ => CheckResult::None,
        }
    }

    pub fn check_update(&self) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match self {
            PluginType::Freeze => CheckResult::CanApprove,
            _ => CheckResult::None,
        }
    }

    pub fn check_burn(&self) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match self {
            PluginType::Freeze => CheckResult::CanReject,
            PluginType::Burn => CheckResult::CanApprove,
            _ => CheckResult::None,
        }
    }

    pub fn check_transfer(&self) -> CheckResult {
        match self {
            PluginType::Royalties => CheckResult::CanReject,
            PluginType::Freeze => CheckResult::CanReject,
            _ => CheckResult::None,
        }
    }

    pub fn check_compress(&self) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match self {
            _ => CheckResult::None,
        }
    }

    pub fn check_decompress(&self) -> CheckResult {
        #[allow(clippy::match_single_binding)]
        match self {
            _ => CheckResult::None,
        }
    }
}

impl Plugin {
    pub fn validate_create(
        &self,
        ctx: &CreateAccounts,
        args: &CreateArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplAssetError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_create(ctx, args, authorities),
            Plugin::Freeze(freeze) => freeze.validate_create(ctx, args, authorities),
            Plugin::Burn(burn) => burn.validate_create(ctx, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_create(ctx, args, authorities),
            Plugin::Collection(collection) => collection.validate_create(ctx, args, authorities),
        }
    }

    pub fn validate_update(
        &self,
        ctx: &UpdateAccounts,
        args: &UpdateArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplAssetError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_update(ctx, args, authorities),
            Plugin::Freeze(freeze) => freeze.validate_update(ctx, args, authorities),
            Plugin::Burn(burn) => burn.validate_update(ctx, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_update(ctx, args, authorities),
            Plugin::Collection(collection) => collection.validate_update(ctx, args, authorities),
        }
    }

    // There is no check for updating a plugin because the plugin itself MUST validate the change.
    pub fn validate_update_plugin(
        &self,
        asset: &Asset,
        ctx: &UpdatePluginAccounts,
        args: &UpdatePluginArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplAssetError::InvalidPlugin.into()),
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

    pub fn validate_burn(
        &self,
        ctx: &BurnAccounts,
        args: &BurnArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplAssetError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_burn(ctx, args, authorities),
            Plugin::Freeze(freeze) => freeze.validate_burn(ctx, args, authorities),
            Plugin::Burn(burn) => burn.validate_burn(ctx, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_burn(ctx, args, authorities),
            Plugin::Collection(collection) => collection.validate_burn(ctx, args, authorities),
        }
    }

    pub fn validate_transfer(
        &self,
        ctx: &TransferAccounts,
        args: &TransferArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplAssetError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_transfer(ctx, args, authorities),
            Plugin::Freeze(freeze) => freeze.validate_transfer(ctx, args, authorities),
            Plugin::Burn(burn) => burn.validate_transfer(ctx, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_transfer(ctx, args, authorities),
            Plugin::Collection(collection) => collection.validate_transfer(ctx, args, authorities),
        }
    }

    pub fn validate_compress(
        &self,
        ctx: &CompressAccounts,
        args: &CompressArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplAssetError::InvalidPlugin.into()),
            Plugin::Royalties(royalties) => royalties.validate_compress(ctx, args, authorities),
            Plugin::Freeze(freeze) => freeze.validate_compress(ctx, args, authorities),
            Plugin::Burn(burn) => burn.validate_compress(ctx, args, authorities),
            Plugin::Transfer(transfer) => transfer.validate_compress(ctx, args, authorities),
            Plugin::Collection(collection) => collection.validate_compress(ctx, args, authorities),
        }
    }

    pub fn validate_decompress(
        &self,
        ctx: &DecompressAccounts,
        args: &DecompressArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplAssetError::InvalidPlugin.into()),
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

// Lifecycle validations
#[derive(Eq, PartialEq)]
pub enum ValidationResult {
    Approved,
    Rejected,
    Pass,
}

pub trait PluginValidation {
    fn validate_create(
        &self,
        ctx: &CreateAccounts,
        args: &CreateArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError>;
    fn validate_update(
        &self,
        ctx: &UpdateAccounts,
        args: &UpdateArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError>;
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
    fn validate_burn(
        &self,
        ctx: &BurnAccounts,
        args: &BurnArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError>;
    fn validate_transfer(
        &self,
        ctx: &TransferAccounts,
        args: &TransferArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError>;
    fn validate_compress(
        &self,
        ctx: &CompressAccounts,
        args: &CompressArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError>;
    fn validate_decompress(
        &self,
        ctx: &DecompressAccounts,
        args: &DecompressArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, ProgramError>;
}
