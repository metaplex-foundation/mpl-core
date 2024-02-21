use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, slot_history::Check};

use super::{Plugin, PluginType};

pub enum CheckResult {
    CanApprove,
    CanReject,
    None,
}

pub enum ValidationResult {
    Approved,
    Rejected,
    Pass,
}

// Lifecycle validations
pub fn check_create(plugin_type: PluginType) -> Result<CheckResult, ProgramError> {
    match plugin_type {
        _ => Ok(CheckResult::No),
    }
}

pub fn check_update(plugin_type: PluginType) -> Result<CheckResult, ProgramError> {
    match plugin_type {
        _ => Ok(CheckResult::No),
    }
}

pub fn check_burn(plugin_type: PluginType) -> Result<CheckResult, ProgramError> {
    match plugin_type {
        PluginType::Delegate => {
            // If the delegate plugin authority is the signer, then approve the burn.
            Ok(CheckResult::Yes)
            // Otherwise pass.
        }
        _ => Ok(CheckResult::No),
    }
}

pub fn check_transfer(plugin_type: PluginType) -> Result<CheckResult, ProgramError> {
    match plugin_type {
        PluginType::Delegate => {
            // If the delegate plugin authority is the signer, then approve the burn.
            Ok(CheckResult::Yes)
            // Otherwise pass.
        }
        PluginType::Royalties => {
            // If the auth rules passes, pass.
            Ok(CheckResult::Yes)
            // Otherwise fail.
        }
        _ => Ok(CheckResult::No),
    }
}

pub fn check_compress(plugin_type: PluginType) -> Result<CheckResult, ProgramError> {
    match plugin_type {
        _ => Ok(CheckResult::No),
    }
}

pub fn check_decompress(plugin_type: PluginType) -> Result<CheckResult, ProgramError> {
    match plugin_type {
        _ => Ok(CheckResult::No),
    }
}

// Lifecycle hooks
pub fn on_create(plugin: Plugin) -> Result<ValidationResult, ProgramError> {
    match plugin {
        _ => Ok(ValidationResult::Pass),
    }
}
pub fn on_update(plugin: Plugin) -> Result<ValidationResult, ProgramError> {
    match plugin {
        _ => Ok(ValidationResult::Pass),
    }
}
pub fn on_burn(plugin: Plugin) -> Result<ValidationResult, ProgramError> {
    match plugin {
        Plugin::Delegate(_) => {
            // If the delegate plugin authority is the signer, then approve the burn.
            Ok(ValidationResult::Pass)
            // Otherwise fail.
        }
        _ => Ok(ValidationResult::Pass),
    }
}
pub fn on_transfer(plugin: Plugin) -> Result<ValidationResult, ProgramError> {
    match plugin {
        Plugin::Delegate(_) => {
            // If the delegate plugin authority is the signer, then approve the Transfer.
            Ok(ValidationResult::Pass)
            // Otherwise fail.
        }
        _ => Ok(ValidationResult::Pass),
    }
}
pub fn on_compress(plugin: Plugin) -> Result<ValidationResult, ProgramError> {
    match plugin {
        _ => Ok(ValidationResult::Pass),
    }
}
pub fn on_decompress(plugin: Plugin) -> Result<ValidationResult, ProgramError> {
    match plugin {
        _ => Ok(ValidationResult::Pass),
    }
}
