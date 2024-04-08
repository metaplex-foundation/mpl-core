use borsh::{BorshDeserialize, BorshSerialize};

use super::ExternalPluginSchema;

/// Advanced lifecycle hook that CPIs into a hooked program specified in the `ExternalPluginKey`.
/// This hook is used for any lifecycle events that were selected in the `ExternalPluginRecord` for
/// the plugin.  Any accounts specified in the extra account list account are added to the CPI.
/// The hooked program will return a validation result and new data to store at the plugin's data
/// offset (which in the account is immediately after this header).
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct AdvancedLifecycleHook {
    /// Schema for the data used by the plugin.
    pub schema: ExternalPluginSchema,
}
