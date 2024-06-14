use borsh::{BorshDeserialize, BorshSerialize};

use super::{ExternalPluginAdapterSchema, LinkedDataKey, PluginValidation};

/// The data section plugin is a third party plugin that is _always_ managed by another plugin.
/// Currently these are used for the `LinkedAppData`, and `LinkedLifecycleHook` plugins.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataSection {
    /// The key to the plugin that manages this data section.
    pub parent_key: LinkedDataKey,
    /// Schema for the data used by the plugin.
    pub schema: ExternalPluginAdapterSchema,
}

impl PluginValidation for DataSection {}

impl From<&DataSectionInitInfo> for DataSection {
    fn from(init_info: &DataSectionInitInfo) -> Self {
        Self {
            parent_key: init_info.parent_key,
            schema: init_info.schema,
        }
    }
}

/// App data initialization info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataSectionInitInfo {
    /// The key to the plugin that manages this data section.
    pub parent_key: LinkedDataKey,
    /// Schema for the data used by the plugin.
    pub schema: ExternalPluginAdapterSchema,
}

/// App data update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataSectionUpdateInfo {}
