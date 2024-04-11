use borsh::{BorshDeserialize, BorshSerialize};

use super::{
    Authority, ExternalCheckResult, ExternalPluginSchema, ExtraAccount, HookableLifecycleEvent,
};

/// Lifecycle hook that CPIs into a hooked program specified in the `ExternalPluginKey`.  This hook
/// is used for any lifecycle events that were selected in the `ExternalPluginRecord` for the
/// plugin.  If any extra accounts are present in the `extra_accounts` optional Vec, then these
/// accounts are added to the CPI call in the order in which they are in the Vec.  Any PDAs in the
/// Vec are derived using the hooked program.  The hooked program will return a validation result
/// and new data to store at the plugin's data offset (which in the account is immediately after
/// this header).
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct LifecycleHook {
    /// The extra accounts to use for the lifecycle hook.
    pub extra_accounts: Option<Vec<ExtraAccount>>,
    /// Schema for the data used by the plugin.
    pub schema: ExternalPluginSchema,
    /// The offset to the plugin data in the account.
    pub data_offset: usize,
    /// The length of the plugin data.
    pub data_len: usize,
}

/// Lifecycle hook initialization info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct LifecycleHookInitInfo {
    /// Initial authority.
    pub init_authority: Option<Authority>,
    /// The lifecyle events for which the the external plugin is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// The extra accounts to use for the lifecycle hook.
    pub extra_accounts: Option<Vec<ExtraAccount>>,
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginSchema>,
    /// External plugin initial data.
    pub data: Option<Vec<u8>>,
}

/// Lifecycle hook update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct LifecycleHookUpdateInfo {
    /// The lifecyle events for which the the external plugin is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// The extra accounts to use for the lifecycle hook.
    pub extra_accounts: Option<Vec<ExtraAccount>>,
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginSchema>,
}
