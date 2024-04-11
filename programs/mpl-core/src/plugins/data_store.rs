use borsh::{BorshDeserialize, BorshSerialize};

use super::{Authority, ExternalCheckResult, ExternalPluginSchema, HookableLifecycleEvent};

/// The data store third party plugin contains arbitrary data that can be written to by the
/// authority specified in the `ExternalPluginKey`.  Note this is different then the plugin
/// authority as it cannot update/revoke authority for the plugin.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataStore {
    /// Schema for the data used by the plugin.
    pub schema: ExternalPluginSchema,
    /// The offset to the plugin data in the account.
    pub data_offset: usize,
    /// The length of the plugin data.
    pub data_len: usize,
}

/// Data store initialization info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataStoreInitInfo {
    /// Initial authority.
    pub init_authority: Option<Authority>,
    /// The lifecyle events for which the the external plugin is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginSchema>,
    /// External plugin initial data.
    pub data: Option<Vec<u8>>,
}

/// Data store update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataStoreUpdateInfo {
    /// The lifecyle events for which the the external plugin is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginSchema>,
}
