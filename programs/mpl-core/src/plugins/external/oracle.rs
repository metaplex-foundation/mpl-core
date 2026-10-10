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
                HookableLifecycleEvent::Execute => Ok(ValidationResult::Pass),
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

#[cfg(test)]
mod validation_tests {
    use {
        super::*,
        crate::plugins::{
            test_ctx::{default_ctx, FakeAccount},
            ExtraAccount,
        },
        solana_program::account_info::AccountInfo,
    };

    fn core_err<T>(result: Result<T, ProgramError>) -> crate::error::MplCoreError {
        match result {
            Err(ProgramError::Custom(code)) => {
                num_traits::FromPrimitive::from_u32(code).expect("an MplCoreError code")
            }
            Err(other) => panic!("expected a custom program error, got {other:?}"),
            Ok(_) => panic!("expected an error"),
        }
    }

    fn validation(result: &ExternalValidationResult) -> OracleValidation {
        OracleValidation::V1 {
            create: result.clone(),
            transfer: result.clone(),
            burn: result.clone(),
            update: result.clone(),
        }
    }

    /// `offset` zero bytes followed by the serialized validation, as the
    /// on-chain fixture builds them.
    fn oracle_bytes(validation: &OracleValidation, offset: usize) -> Vec<u8> {
        let mut data = vec![0u8; offset];
        data.extend(borsh::to_vec(validation).unwrap());
        data
    }

    #[test]
    fn to_offset_usize_and_serialized_size() {
        assert_eq!(ValidationResultsOffset::NoOffset.to_offset_usize(), 0);
        assert_eq!(ValidationResultsOffset::Anchor.to_offset_usize(), 8);
        assert_eq!(ValidationResultsOffset::Custom(42).to_offset_usize(), 42);
        assert_eq!(OracleValidation::serialized_size(), 5);
        assert_eq!(
            borsh::to_vec(&validation(&ExternalValidationResult::Pass))
                .unwrap()
                .len(),
            OracleValidation::serialized_size()
        );
    }

    #[test]
    fn oracle_update_and_init_conversions() {
        let base = Pubkey::new_unique();
        let init = OracleInitInfo {
            base_address: base,
            init_plugin_authority: None,
            lifecycle_checks: vec![],
            base_address_config: None,
            results_offset: None,
        };
        // A missing `results_offset` defaults to `NoOffset`.
        assert_eq!(
            Oracle::from(&init),
            Oracle {
                base_address: base,
                base_address_config: None,
                results_offset: ValidationResultsOffset::NoOffset,
            }
        );

        let config = ExtraAccount::PreconfiguredAsset {
            is_signer: false,
            is_writable: false,
        };
        let init = OracleInitInfo {
            results_offset: Some(ValidationResultsOffset::Anchor),
            base_address_config: Some(config.clone()),
            ..init
        };
        assert_eq!(
            Oracle::from(&init),
            Oracle {
                base_address: base,
                base_address_config: Some(config.clone()),
                results_offset: ValidationResultsOffset::Anchor,
            }
        );

        // `update` only writes the fields that are `Some`.
        let mut oracle = Oracle {
            base_address: base,
            base_address_config: None,
            results_offset: ValidationResultsOffset::NoOffset,
        };
        oracle.update(&OracleUpdateInfo {
            lifecycle_checks: None,
            base_address_config: None,
            results_offset: None,
        });
        assert_eq!(oracle.base_address_config, None);
        assert_eq!(oracle.results_offset, ValidationResultsOffset::NoOffset);

        oracle.update(&OracleUpdateInfo {
            lifecycle_checks: None,
            base_address_config: Some(config.clone()),
            results_offset: Some(ValidationResultsOffset::Custom(9)),
        });
        assert_eq!(oracle.base_address_config, Some(config));
        assert_eq!(oracle.results_offset, ValidationResultsOffset::Custom(9));
    }

    #[test]
    fn validate_helper_returns_the_arm_for_each_event() {
        let base = Pubkey::new_unique();
        let self_authority = Authority::UpdateAuthority;

        for (result, expected) in [
            (
                ExternalValidationResult::Approved,
                ValidationResult::Approved,
            ),
            (
                ExternalValidationResult::Rejected,
                ValidationResult::Rejected,
            ),
            (ExternalValidationResult::Pass, ValidationResult::Pass),
        ] {
            let mut signer = FakeAccount::wallet();
            let mut account = FakeAccount::with_data(oracle_bytes(&validation(&result), 0));
            account.key = base;
            let signer_info = signer.info();
            let info = account.info();
            let accounts: Vec<AccountInfo> = vec![info];
            let ctx = default_ctx(&accounts, &signer_info, &self_authority);
            let oracle = Oracle {
                base_address: base,
                base_address_config: None,
                results_offset: ValidationResultsOffset::NoOffset,
            };

            assert_eq!(oracle.validate_create(&ctx).unwrap(), expected);
            assert_eq!(oracle.validate_transfer(&ctx).unwrap(), expected);
            assert_eq!(oracle.validate_burn(&ctx).unwrap(), expected);
            assert_eq!(oracle.validate_update(&ctx).unwrap(), expected);

            // Roadmap section 11, finding 3: the `Execute` arm of the helper
            // always passes, and nothing routes Oracle's execute through it.
            assert_eq!(
                oracle
                    .validate_helper(&ctx, HookableLifecycleEvent::Execute)
                    .unwrap(),
                ValidationResult::Pass
            );
        }
    }

    #[test]
    fn validate_helper_error_arms() {
        let base = Pubkey::new_unique();
        let self_authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();

        let oracle = |results_offset| Oracle {
            base_address: base,
            base_address_config: None,
            results_offset,
        };

        // The account is not among `ctx.accounts` at all.
        let ctx = default_ctx(&[], &signer_info, &self_authority);
        assert_eq!(
            core_err(oracle(ValidationResultsOffset::NoOffset).validate_transfer(&ctx)),
            crate::error::MplCoreError::MissingExternalPluginAdapterAccount
        );

        // (account bytes, stored offset, expected error)
        let cases = [
            (vec![1u8, 0, 0], ValidationResultsOffset::NoOffset),
            (vec![1u8, 2, 2, 2, 2], ValidationResultsOffset::Custom(64)),
        ];
        for (bytes, offset) in cases {
            let mut loop_signer = FakeAccount::wallet();
            let mut account = FakeAccount::with_data(bytes);
            account.key = base;
            let loop_signer_info = loop_signer.info();
            let info = account.info();
            let accounts: Vec<AccountInfo> = vec![info];
            let ctx = default_ctx(&accounts, &loop_signer_info, &self_authority);
            assert_eq!(
                core_err(oracle(offset).validate_transfer(&ctx)),
                crate::error::MplCoreError::InvalidOracleAccountData
            );
        }

        // A discriminant that is neither `Uninitialized` nor `V1`.
        let mut account = FakeAccount::with_data(vec![2u8, 0, 0, 0, 0]);
        account.key = base;
        let info = account.info();
        let accounts: Vec<AccountInfo> = vec![info];
        let ctx = default_ctx(&accounts, &signer_info, &self_authority);
        assert_eq!(
            core_err(oracle(ValidationResultsOffset::NoOffset).validate_transfer(&ctx)),
            crate::error::MplCoreError::InvalidOracleAccountData
        );

        // An all-zero account deserializes as `Uninitialized`.
        let mut account = FakeAccount::with_data(vec![0u8; 16]);
        account.key = base;
        let info = account.info();
        let accounts: Vec<AccountInfo> = vec![info];
        let ctx = default_ctx(&accounts, &signer_info, &self_authority);
        assert_eq!(
            core_err(oracle(ValidationResultsOffset::NoOffset).validate_transfer(&ctx)),
            crate::error::MplCoreError::UninitializedOracleAccount
        );

        // An `Anchor` offset skips the 8-byte discriminator.
        let mut account = FakeAccount::with_data(oracle_bytes(
            &validation(&ExternalValidationResult::Rejected),
            8,
        ));
        account.key = base;
        let info = account.info();
        let accounts: Vec<AccountInfo> = vec![info];
        let ctx = default_ctx(&accounts, &signer_info, &self_authority);
        assert_eq!(
            oracle(ValidationResultsOffset::Anchor)
                .validate_transfer(&ctx)
                .unwrap(),
            ValidationResult::Rejected
        );
    }

    #[test]
    fn validate_helper_derives_the_account_from_the_config() {
        let base = Pubkey::new_unique();
        let self_authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();

        let derived =
            Pubkey::find_program_address(&[crate::plugins::MPL_CORE_PREFIX.as_bytes()], &base).0;
        let mut account = FakeAccount::with_data(oracle_bytes(
            &validation(&ExternalValidationResult::Rejected),
            0,
        ));
        account.key = derived;
        let info = account.info();
        let accounts: Vec<AccountInfo> = vec![info];
        let ctx = default_ctx(&accounts, &signer_info, &self_authority);

        let oracle = Oracle {
            base_address: base,
            base_address_config: Some(ExtraAccount::PreconfiguredProgram {
                is_signer: false,
                is_writable: false,
            }),
            results_offset: ValidationResultsOffset::NoOffset,
        };
        assert_eq!(
            oracle.validate_transfer(&ctx).unwrap(),
            ValidationResult::Rejected
        );
    }

    #[test]
    fn validate_add_external_plugin_adapter_abstains() {
        let self_authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let ctx = default_ctx(&[], &signer_info, &self_authority);
        let oracle = Oracle {
            base_address: Pubkey::new_unique(),
            base_address_config: None,
            results_offset: ValidationResultsOffset::NoOffset,
        };
        assert_eq!(
            oracle.validate_add_external_plugin_adapter(&ctx).unwrap(),
            ValidationResult::Pass
        );
    }
}
