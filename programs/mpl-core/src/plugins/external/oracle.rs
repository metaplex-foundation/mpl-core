use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::MplCoreError;

use crate::plugins::{
    abstain, Authority, ExternalCheckResult, ExternalValidationResult, ExtraAccount,
    HookableLifecycleEvent, PluginValidation, PluginValidationContext, ValidationResult,
};

/// Oracle plugin that allows getting a `ValidationResult` for a lifecycle event from an arbitrary
/// account either specified by or derived from the `base_address`.  This hook is used for any
/// lifecycle events that were selected in the `ExternalRegistryRecord` for the plugin.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct Oracle {
    /// The address of the oracle, or if using the `pda` option, a program ID from which
    /// to derive a PDA.
    pub base_address: Pubkey,
    /// Optional account specification (PDA derived from `base_address` or other available account
    /// specifications).  Note that even when this configuration is used there is still only one
    /// Oracle account specified by the adapter.
    pub base_address_config: Option<ExtraAccount>,
    /// Validation results offset in the Oracle account.  Default is `ValidationResultsOffset::NoOffset`.
    pub results_offset: ValidationResultsOffset,
}

impl Oracle {
    /// Updates the oracle with the new info.
    pub fn update(&mut self, info: &OracleUpdateInfo) {
        if let Some(base_address_config) = &info.base_address_config {
            self.base_address_config = Some(base_address_config.clone());
        }
        if let Some(results_offset) = &info.results_offset {
            self.results_offset = *results_offset;
        }
    }
}

impl PluginValidation for Oracle {
    fn validate_add_external_plugin_adapter(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    fn validate_create(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        self.validate_helper(ctx, HookableLifecycleEvent::Create)
    }

    fn validate_transfer(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        self.validate_helper(ctx, HookableLifecycleEvent::Transfer)
    }

    fn validate_burn(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        self.validate_helper(ctx, HookableLifecycleEvent::Burn)
    }

    fn validate_update(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        self.validate_helper(ctx, HookableLifecycleEvent::Update)
    }
}

impl Oracle {
    fn validate_helper(
        &self,
        ctx: &PluginValidationContext,
        event: HookableLifecycleEvent,
    ) -> Result<ValidationResult, ProgramError> {
        let oracle_account = match &self.base_address_config {
            None => self.base_address,
            Some(extra_account) => extra_account.derive(&self.base_address, ctx)?,
        };

        let oracle_account = ctx
            .accounts
            .iter()
            .find(|account| *account.key == oracle_account)
            .ok_or(MplCoreError::MissingExternalPluginAdapterAccount)?;

        let offset = self.results_offset.to_offset_usize();

        let oracle_data = (*oracle_account.data).borrow();
        let mut oracle_data_slice = oracle_data
            .get(offset..)
            .ok_or(MplCoreError::InvalidOracleAccountData)?;

        if oracle_data_slice.len() < OracleValidation::serialized_size() {
            return Err(MplCoreError::InvalidOracleAccountData.into());
        }

        let validation_result = OracleValidation::deserialize(&mut oracle_data_slice)
            .map_err(|_| MplCoreError::InvalidOracleAccountData)?;

        match validation_result {
            OracleValidation::Uninitialized => Err(MplCoreError::UninitializedOracleAccount.into()),
            OracleValidation::V1 {
                create,
                transfer,
                burn,
                update,
            } => match event {
                HookableLifecycleEvent::Create => Ok(ValidationResult::from(create)),
                HookableLifecycleEvent::Transfer => Ok(ValidationResult::from(transfer)),
                HookableLifecycleEvent::Burn => Ok(ValidationResult::from(burn)),
                HookableLifecycleEvent::Update => Ok(ValidationResult::from(update)),
            },
        }
    }
}

impl From<&OracleInitInfo> for Oracle {
    fn from(init_info: &OracleInitInfo) -> Self {
        Self {
            base_address: init_info.base_address,
            base_address_config: init_info.base_address_config.clone(),
            results_offset: init_info
                .results_offset
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
    /// The lifecyle events for which the the external plugin adapter is active.
    pub lifecycle_checks: Vec<(HookableLifecycleEvent, ExternalCheckResult)>,
    /// Optional account specification (PDA derived from `base_address` or other available account
    /// specifications).  Note that even when this configuration is used there is still only one
    /// Oracle account specified by the adapter.
    pub base_address_config: Option<ExtraAccount>,
    /// Optional offset for validation results struct used in Oracle account.  Default
    /// is `ValidationResultsOffset::NoOffset`.
    pub results_offset: Option<ValidationResultsOffset>,
}

/// Oracle update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct OracleUpdateInfo {
    /// The lifecyle events for which the the external plugin adapter is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// Optional account specification (PDA derived from `base_address` or other available account
    /// specifications).  Note that even when this configuration is used there is still only one
    /// Oracle account specified by the adapter.
    pub base_address_config: Option<ExtraAccount>,
    /// Optional offset for validation results struct used in Oracle account.  Default
    /// is `ValidationResultsOffset::NoOffset`.
    pub results_offset: Option<ValidationResultsOffset>,
}

/// Offset to where the validation results struct is located in an Oracle account.
#[derive(Copy, Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ValidationResultsOffset {
    /// The validation struct is located at the beginning of the account.
    NoOffset,
    /// The Oracle is an Anchor account so the validation struct is located after an 8-byte
    /// account discriminator.
    Anchor,
    /// The validation struct is located at the specified offset within the account.
    Custom(usize),
}

impl ValidationResultsOffset {
    /// Convert the `ValidationResultsOffset` to the correct offset value as a `usize`.
    pub fn to_offset_usize(self) -> usize {
        match self {
            Self::NoOffset => 0,
            Self::Anchor => 8,
            Self::Custom(offset) => offset,
        }
    }
}

/// Validation results struct for an Oracle account.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum OracleValidation {
    /// Uninitialized data.  This is intended to prevent leaving an account zeroed out by mistake.
    Uninitialized,
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

impl OracleValidation {
    /// Borsh- and Anchor-serialized size of the `OracleValidation` struct.
    pub fn serialized_size() -> usize {
        5
    }
}
