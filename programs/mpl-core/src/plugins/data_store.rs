use borsh::{BorshDeserialize, BorshSerialize};

use super::{Authority, ExternalPluginSchema};

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
    /// Data authority who can update the data store.  Cannot be changed after plugin is
    /// added.
    pub data_authority: Authority,
    /// Initial plugin authority who can update plugin properties.
    pub init_plugin_authority: Option<Authority>,
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginSchema>,
}

/// Data store update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataStoreUpdateInfo {
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginSchema>,
}
