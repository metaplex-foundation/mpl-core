use solana_program::program_error::ProgramError;

use super::{Plugin, PluginType};

// Lifecycle permissions
pub enum CheckResult {
    CanApprove,
    CanReject,
    None,
}

pub trait CheckLifecyclePermission {
    fn check_create(plugin_type: PluginType) -> Result<CheckResult, ProgramError> {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => Ok(CheckResult::None),
        }
    }

    fn check_update(plugin_type: PluginType) -> Result<CheckResult, ProgramError> {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => Ok(CheckResult::None),
        }
    }

    fn check_burn(plugin_type: PluginType) -> Result<CheckResult, ProgramError> {
        match plugin_type {
            PluginType::Delegate => {
                // If the delegate plugin authority is the signer, then approve the burn.
                Ok(CheckResult::CanApprove)
                // Otherwise pass.
            }
            _ => Ok(CheckResult::None),
        }
    }

    fn check_transfer(plugin_type: PluginType) -> Result<CheckResult, ProgramError> {
        match plugin_type {
            PluginType::Delegate => {
                // If the delegate plugin authority is the signer, then approve the burn.
                Ok(CheckResult::CanApprove)
                // Otherwise pass.
            }
            PluginType::Royalties => {
                // If the auth rules passes, pass.
                Ok(CheckResult::CanReject)
                // Otherwise fail.
            }
            _ => Ok(CheckResult::None),
        }
    }

    fn check_compress(plugin_type: PluginType) -> Result<CheckResult, ProgramError> {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => Ok(CheckResult::None),
        }
    }

    fn check_decompress(plugin_type: PluginType) -> Result<CheckResult, ProgramError> {
        #[allow(clippy::match_single_binding)]
        match plugin_type {
            _ => Ok(CheckResult::None),
        }
    }
}

// Lifecycle validations
pub enum ValidationResult {
    Approved,
    Rejected,
    Pass,
}

pub trait ValidateLifecycle {
    fn validate_create(plugin: &Plugin) -> Result<ValidationResult, ProgramError>;
    fn validate_update(plugin: &Plugin) -> Result<ValidationResult, ProgramError>;
    fn validate_burn(plugin: &Plugin) -> Result<ValidationResult, ProgramError>;
    fn validate_transfer(plugin: &Plugin) -> Result<ValidationResult, ProgramError>;
    fn validate_compress(plugin: &Plugin) -> Result<ValidationResult, ProgramError>;
    fn validate_decompress(plugin: &Plugin) -> Result<ValidationResult, ProgramError>;
}
