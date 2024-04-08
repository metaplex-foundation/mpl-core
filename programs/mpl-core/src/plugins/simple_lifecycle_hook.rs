use borsh::{BorshDeserialize, BorshSerialize};

use super::{ExternalPluginSchema, PreconfiguredPda};

/// Simple lifecycle hook that CPIs into a hooked program specified in the `ExternalPluginKey`.
/// This hook is used for any lifecycle events that were selected in the `ExternalPluginRecord` for
/// the plugin.  If any preconfigured PDAs are present in the `pdas` Vec, then these PDAs are
/// derived using the hooked program and added to the CPI call.  The hooked program will return a
/// validation result and new data to store at the plugin's data offset (which in the account is
/// immediately after this header).
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct SimpleLifecycleHook {
    /// The preconfigured PDAs to use for the lifecycle hook.
    pub pdas: Vec<PreconfiguredPda>,
    /// Schema for the data used by the plugin.
    pub schema: ExternalPluginSchema,
}
