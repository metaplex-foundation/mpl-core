use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use super::{
    Authority, ExternalCheckResult, ExtraAccount, HookableLifecycleEvent, PluginValidation,
    PluginValidationContext, ValidationResult,
};

/// Oracle plugin that allows getting a `ValidationResult` for a lifecycle event from an arbitrary
/// account either specified by or derived from the `base_address`.  This hook is used for any
/// lifecycle events that were selected in the `ExternalPluginRecord` for the plugin.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct Oracle {
    /// The address of the oracle, or if using the `pda` option, a program ID from which
    /// to derive a PDA.
    pub base_address: Pubkey,
    /// Optional PDA (derived from Pubkey attached to `ExternalPluginKey`).
    pub pda: Option<ExtraAccount>,
}

impl From<&OracleInitInfo> for Oracle {
    fn from(init_info: &OracleInitInfo) -> Self {
        Self {
            base_address: init_info.base_address,
            pda: init_info.pda.clone(),
        }
    }
}

/// Oracle initialization info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct OracleInitInfo {
    /// The address of the oracle, or if using the `pda` option, a program ID from which
    /// to derive a PDA.
    pub base_address: Pubkey,
    /// Initial plugin authority.
    pub init_plugin_authority: Option<Authority>,
    /// The lifecyle events for which the the external plugin is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// Optional PDA (derived from Pubkey attached to `ExternalPluginKey`).
    pub pda: Option<ExtraAccount>,
}

impl PluginValidation for OracleInitInfo {
    fn validate_add_external_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Approved)
    }
}

/// Oracle update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct OracleUpdateInfo {
    /// The lifecyle events for which the the external plugin is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// Optional PDA (derived from Pubkey attached to `ExternalPluginKey`).
    pub pda: Option<ExtraAccount>,
}
