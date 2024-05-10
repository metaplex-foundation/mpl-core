#[cfg(feature = "anchor")]
use anchor_lang::prelude::AnchorSerialize;
#[cfg(not(feature = "anchor"))]
use borsh::BorshSerialize;

use crate::{
    accounts::{BaseAssetV1, PluginHeaderV1},
    registry_records_to_plugin_adapter_list, registry_records_to_plugin_list, Asset,
    PluginRegistryV1Safe,
};

impl Asset {
    pub fn deserialize(data: &[u8]) -> Result<Self, std::io::Error> {
        let base = BaseAssetV1::from_bytes(data)?;
        let base_data = base.try_to_vec()?;
        let (plugin_header, plugin_list, plugin_adapter_list) = if base_data.len() != data.len() {
            let plugin_header = PluginHeaderV1::from_bytes(&data[base_data.len()..])?;
            let plugin_registry = PluginRegistryV1Safe::from_bytes(
                &data[plugin_header.plugin_registry_offset as usize..],
            )?;

            let plugin_list = registry_records_to_plugin_list(&plugin_registry.registry, data)?;
            let plugin_adapter_list =
                registry_records_to_plugin_adapter_list(&plugin_registry.adapter_registry, data)?;

            (
                Some(plugin_header),
                Some(plugin_list),
                Some(plugin_adapter_list),
            )
        } else {
            (None, None, None)
        };

        Ok(Self {
            base,
            plugin_list: plugin_list.unwrap_or_default(),
            plugin_adapter_list: plugin_adapter_list.unwrap_or_default(),
            plugin_header,
        })
    }

    #[inline(always)]
    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
        Self::deserialize(data)
    }
}

impl<'a> TryFrom<&solana_program::account_info::AccountInfo<'a>> for Asset {
    type Error = std::io::Error;

    fn try_from(
        account_info: &solana_program::account_info::AccountInfo<'a>,
    ) -> Result<Self, Self::Error> {
        let data: &[u8] = &(*account_info.data).borrow();
        Self::deserialize(data)
    }
}
