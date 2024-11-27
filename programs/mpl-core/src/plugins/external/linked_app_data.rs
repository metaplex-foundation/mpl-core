use borsh::{BorshDeserialize, BorshSerialize};

use crate::plugins::{
    Authority, ExternalPluginAdapterSchema, PluginValidation, PluginValidationContext,
    ValidationResult,
};

/// The app data third party plugin contains arbitrary data that can be written to by the
/// `data_authority`.  Note this is different then the overall plugin authority stored in the
/// `ExternalRegistryRecord` as it cannot update/revoke authority or change other metadata for the
/// plugin.  The data is stored at the plugin's data offset (which in the account is immediately
/// after this header).
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct LinkedAppData {
    /// Data authority who can update the app data.  Cannot be changed after plugin is
    /// added.
    pub data_authority: Authority,
    /// Schema for the data used by the plugin.
    pub schema: ExternalPluginAdapterSchema,
}

impl LinkedAppData {
    /// Updates the app data with the new info.
    pub fn update(&mut self, info: &LinkedAppDataUpdateInfo) {
        if let Some(schema) = &info.schema {
            self.schema = *schema;
        }
    }
}

impl PluginValidation for LinkedAppData {
    fn validate_create(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, solana_program::program_error::ProgramError> {
        solana_program::msg!("LinkedAppData::validate_create");
        if ctx.asset_info.is_some() {
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}

impl From<&LinkedAppDataInitInfo> for LinkedAppData {
    fn from(init_info: &LinkedAppDataInitInfo) -> Self {
        Self {
            data_authority: init_info.data_authority,
            schema: init_info.schema.unwrap_or_default(),
        }
    }
}

/// App data initialization info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct LinkedAppDataInitInfo {
    /// Data authority who can update the app data.  This field cannot be
    /// changed after the plugin is added.
    pub data_authority: Authority,
    /// Initial plugin authority who can update plugin properties.
    pub init_plugin_authority: Option<Authority>,
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginAdapterSchema>,
}

/// App data update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct LinkedAppDataUpdateInfo {
    /// Schema for the data used by the plugin.
    pub schema: Option<ExternalPluginAdapterSchema>,
}
