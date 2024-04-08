use borsh::{BorshDeserialize, BorshSerialize};

use super::{ExternalPluginSchema, ExtraAccount};

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
    pub schema: Option<ExternalPluginSchema>,
}
