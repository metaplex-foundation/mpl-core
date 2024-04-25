use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use super::{
    Authority, ExternalCheckResult, ExternalValidationResult, ExtraAccount, HookableLifecycleEvent,
    PluginValidation, PluginValidationContext, ValidationResult,
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
    /// Validation results offset in the Oracle account.  Default is `ValidationResultsOffset::NoOffset`.
    pub results_offset: ValidationResultsOffset,
}

impl PluginValidation for Oracle {
    fn validate_add_external_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_transfer(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }
}

impl From<&OracleInitInfo> for Oracle {
    fn from(init_info: &OracleInitInfo) -> Self {
        Self {
            base_address: init_info.base_address,
            pda: init_info.pda.clone(),
            results_offset: init_info
                .results_offset
                .clone()
                .unwrap_or(ValidationResultsOffset::NoOffset),
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
    /// Optional offset for validation results struct used in Oracle account.  Default
    /// is `ValidationResultsOffset::NoOffset`.
    pub results_offset: Option<ValidationResultsOffset>,
}

/// Oracle update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct OracleUpdateInfo {
    /// The lifecyle events for which the the external plugin is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// Optional PDA (derived from Pubkey attached to `ExternalPluginKey`).
    pub pda: Option<ExtraAccount>,
    /// Optional offset for validation results struct used in Oracle account.  Default
    /// is `ValidationResultsOffset::NoOffset`.
    pub results_offset: Option<ValidationResultsOffset>,
}

/// Offset to where the validation results struct is located in an Oracle account.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ValidationResultsOffset {
    /// The validation struct is located at the beginning of the account.
    NoOffset,
    /// The Oracle is an Anchor account so the validation struct is located after an 8-byte
    /// account discriminator.
    Anchor,
    /// The validation struct is located at the specified offset within the account.
    Custom(usize),
}

/// Validation results struct for an Oracle account.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum OracleValidation {
    /// Version 1 of the format.
    V1 {
        /// Validation for the the create lifecycle action.
        create: ExternalValidationResult,
        /// Validation for the transfer lifecycle action.
        transfer: ExternalValidationResult,
        /// Validation for the burn lifecycle action.
        burn: ExternalValidationResult,
        /// Validation for the update lifecycle action.
        update: ExternalValidationResult,
    },
}
