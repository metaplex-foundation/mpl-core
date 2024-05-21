use borsh::{BorshDeserialize, BorshSerialize};

use super::{LinkedDataKey, PluginValidation};

/// The data section plugin is a third party plugin that is _always_ managed by another plugin.
/// Currently these are used for the `SecureDataStore`, `AssetLinkedSecureDataStore`, and `LifecycleHook` plugins.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataSection {
    /// The key to the plugin that manages this data section.
    pub manager_key: LinkedDataKey,
}

impl PluginValidation for DataSection {}

impl From<&DataSectionInitInfo> for DataSection {
    fn from(init_info: &DataSectionInitInfo) -> Self {
        Self {
            manager_key: init_info.manager_key,
        }
    }
}

/// Data store initialization info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataSectionInitInfo {
    /// The key to the plugin that manages this data section.
    pub manager_key: LinkedDataKey,
}

/// Data store update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct DataSectionUpdateInfo {}
