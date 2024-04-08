use borsh::{BorshDeserialize, BorshSerialize};

use super::ExternalPluginSchema;

/// The data store third party plugin contains arbitrary data that can be written to by the
/// authority specified in the `ExternalPluginKey`.  Note this is different then the plugin
/// authority as it cannot add/revoke authority for the plugin.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataStore {
    /// Schema for the data used by the plugin.
    pub schema: ExternalPluginSchema,
}
