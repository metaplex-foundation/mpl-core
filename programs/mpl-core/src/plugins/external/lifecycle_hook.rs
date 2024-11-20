use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::plugins::{
    abstain, Authority, ExternalCheckResult, ExternalPluginAdapterSchema, ExtraAccount,
    HookableLifecycleEvent, PluginValidation, PluginValidationContext, ValidationResult,
};

/// Lifecycle hook that CPIs into the `hooked_program`.  This hook is used for any lifecycle events
/// that were selected in the `ExternalRegistryRecord` for the plugin.  If any extra accounts are
/// present in the `extra_accounts` optional `Vec`, then these accounts are added to the CPI call
/// in the order in which they are in the Vec.  Any PDAs in the `Vec` are derived using the hooked
/// program.  The hooked program will return a validation result and new data to store at the
/// plugin's data offset (which in the account is immediately after this header).
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct LifecycleHook {
    /// The `Pubkey` for the hooked program.
    pub hooked_program: Pubkey, // 32
    /// The extra accounts to use for the lifecycle hook.
    pub extra_accounts: Option<Vec<ExtraAccount>>,
    /// The authority of who can update the Lifecycle Hook data. This can be for the purposes
    /// of initialization of data, or schema migration. This field cannot be changed after
    /// the plugin is added.
    pub data_authority: Option<Authority>,
    /// Schema for the data used by the plugin.
    pub schema: ExternalPluginAdapterSchema, // 1
}

impl LifecycleHook {
    /// Updates the lifecycle hook with the new info.
    pub fn update(&mut self, info: &LifecycleHookUpdateInfo) {
        if let Some(extra_accounts) = &info.extra_accounts {
            self.extra_accounts = Some(extra_accounts.clone());
        }
        if let Some(schema) = &info.schema {
            self.schema = *schema;
        }
    }
}

impl PluginValidation for LifecycleHook {
    fn validate_add_external_plugin_adapter(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    fn validate_transfer(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }
}

impl From<&LifecycleHookInitInfo> for LifecycleHook {
    fn from(init_info: &LifecycleHookInitInfo) -> Self {
        Self {
            hooked_program: init_info.hooked_program,
            extra_accounts: init_info.extra_accounts.clone(),
            data_authority: init_info.data_authority,
            schema: init_info.schema.unwrap_or_default(),
        }
    }
}

/// Lifecycle hook initialization info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct LifecycleHookInitInfo {
    /// The `Pubkey` for the hooked program.
    pub hooked_program: Pubkey,
    /// Initial plugin authority.
    pub init_plugin_authority: Option<Authority>,
    /// The lifecyle events for which the the external plugin adapter is active.
    pub lifecycle_checks: Vec<(HookableLifecycleEvent, ExternalCheckResult)>,
    /// The extra accounts to use for the lifecycle hook.
    pub extra_accounts: Option<Vec<ExtraAccount>>,
    /// The authority of who can update the Lifecycle Hook data. This can be for the purposes
    /// of initialization of data, or schema migration. This field cannot be changed after
    /// the plugin is added.
    pub data_authority: Option<Authority>,
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginAdapterSchema>,
}

/// Lifecycle hook update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct LifecycleHookUpdateInfo {
    /// The lifecyle events for which the the external plugin adapter is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// The extra accounts to use for the lifecycle hook.
    pub extra_accounts: Option<Vec<ExtraAccount>>,
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginAdapterSchema>,
}
