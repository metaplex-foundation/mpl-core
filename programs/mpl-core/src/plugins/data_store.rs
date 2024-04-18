use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use super::{
    Authority, ExternalPluginSchema, PluginValidation, PluginValidationContext, ValidationResult,
};

/// The data store third party plugin contains arbitrary data that can be written to by the
/// `data_authority`.  Note this is different then the overall plugin authority stored in the
/// `ExternalPluginRecord` as it cannot update/revoke authority or change other metadata for the
/// plugin.  The data is stored at the plugin's data offset (which in the account is immediately
/// after this header).
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataStore {
    /// Data authority who can update the data store.  Cannot be changed after plugin is
    /// added.
    pub data_authority: Authority,
    /// Schema for the data used by the plugin.
    pub schema: ExternalPluginSchema,
    /// The offset to the plugin data in the account.
    pub data_offset: usize,
    /// The length of the plugin data.
    pub data_len: usize,
}

impl From<&DataStoreInitInfo> for DataStore {
    fn from(init_info: &DataStoreInitInfo) -> Self {
        Self {
            data_authority: init_info.data_authority,
            schema: init_info.schema.unwrap_or_default(),
            data_offset: 0,
            data_len: 0,
        }
    }
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

impl PluginValidation for DataStoreInitInfo {
    fn validate_add_external_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Approved)
    }
}

/// Data store update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataStoreUpdateInfo {
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginSchema>,
}
