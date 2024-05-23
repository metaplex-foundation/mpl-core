use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use super::{
    Authority, ExternalPluginAdapterSchema, PluginValidation, PluginValidationContext,
    ValidationResult,
};

/// The data store third party plugin contains arbitrary data that can be written to by the
/// `data_authority`.  Note this is different then the overall plugin authority stored in the
/// `ExternalRegistryRecord` as it cannot update/revoke authority or change other metadata for the
/// plugin.  The data is stored at the plugin's data offset (which in the account is immediately
/// after this header).
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct SecureDataStore {
    /// Data authority who can update the data store.  Cannot be changed after plugin is
    /// added.
    pub data_authority: Authority,
    /// Schema for the data used by the plugin.
    pub schema: ExternalPluginAdapterSchema,
}

impl SecureDataStore {
    /// Updates the data store with the new info.
    pub fn update(&mut self, info: &SecureDataStoreUpdateInfo) {
        if let Some(schema) = &info.schema {
            self.schema = *schema;
        }
    }
}

impl PluginValidation for SecureDataStore {
    fn validate_add_external_plugin_adapter(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_transfer(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }
}

impl From<&SecureDataStoreInitInfo> for SecureDataStore {
    fn from(init_info: &SecureDataStoreInitInfo) -> Self {
        Self {
            data_authority: init_info.data_authority,
            schema: init_info.schema.unwrap_or_default(),
        }
    }
}

/// Data store initialization info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct SecureDataStoreInitInfo {
    /// Data authority who can update the data store.  This field cannot be
    /// changed after the plugin is added.
    pub data_authority: Authority,
    /// Initial plugin authority who can update plugin properties.
    pub init_plugin_authority: Option<Authority>,
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginAdapterSchema>,
}

/// Data store update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct SecureDataStoreUpdateInfo {
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginAdapterSchema>,
}
