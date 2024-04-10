use borsh::{BorshDeserialize, BorshSerialize};

use super::ExternalPluginSchema;

/// The data store third party plugin contains arbitrary data that can be written to by the
/// authority specified in the `ExternalPluginKey`.  Note this is different then the plugin
/// authority as it cannot update/revoke authority for the plugin.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataStore {
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginSchema>,
    /// The offset to the plugin data in the account.
    pub data_offset: usize,
    /// The length of the plugin data.
    pub data_len: usize,
}
