use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;
use std::{cmp::Ordering, io::ErrorKind};

use crate::types::{ExternalCheckResult, ExternalPlugin, ExternalPluginKey, Key, PluginAuthority};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum AuthorityType {
    None,
    Owner,
    UpdateAuthority,
    Address,
}

impl From<PluginAuthority> for AuthorityType {
    fn from(authority: PluginAuthority) -> Self {
        match authority {
            PluginAuthority::None => AuthorityType::None,
            PluginAuthority::Owner => AuthorityType::Owner,
            PluginAuthority::UpdateAuthority => AuthorityType::UpdateAuthority,
            PluginAuthority::Address { address: _ } => AuthorityType::Address,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct BaseAuthority {
    pub authority_type: AuthorityType,
    pub address: Option<Pubkey>,
}

impl From<PluginAuthority> for BaseAuthority {
    fn from(authority: PluginAuthority) -> Self {
        match authority {
            PluginAuthority::None => BaseAuthority {
                authority_type: AuthorityType::None,
                address: None,
            },
            PluginAuthority::Owner => BaseAuthority {
                authority_type: AuthorityType::Owner,
                address: None,
            },
            PluginAuthority::UpdateAuthority => BaseAuthority {
                authority_type: AuthorityType::UpdateAuthority,
                address: None,
            },
            PluginAuthority::Address { address } => BaseAuthority {
                authority_type: AuthorityType::Address,
                address: Some(address),
            },
        }
    }
}

/// Registry record that can be used when the plugin type is not known (i.e. a `PluginType` that
/// is too new for this client to know about).
pub struct RegistryRecordSafe {
    pub plugin_type: u8,
    pub authority: PluginAuthority,
    pub offset: u64,
}

impl RegistryRecordSafe {
    /// Associated function for sorting `RegistryRecordIndexable` by offset.
    pub fn compare_offsets(a: &RegistryRecordSafe, b: &RegistryRecordSafe) -> Ordering {
        a.offset.cmp(&b.offset)
    }
}

/// External Registry record that can be used when the plugin type is not known (i.e. a `ExternalPluginType` that
/// is too new for this client to know about).
pub struct ExternalRegistryRecordSafe {
    pub plugin_type: u8,
    pub authority: PluginAuthority,
    pub lifecycle_checks: Option<Vec<(u8, ExternalCheckResult)>>,
    pub offset: u64,
    pub data_offset: Option<u64>,
    pub data_len: Option<u64>,
}

impl ExternalRegistryRecordSafe {
    /// Associated function for sorting `RegistryRecordIndexable` by offset.
    pub fn compare_offsets(a: &RegistryRecordSafe, b: &RegistryRecordSafe) -> Ordering {
        a.offset.cmp(&b.offset)
    }
}

/// Plugin registry that an account can safely be deserialized into even if some plugins are
/// not known.  Note this skips over external plugins for now, and will be updated when those
/// are defined.
pub struct PluginRegistryV1Safe {
    pub _key: Key,
    pub registry: Vec<RegistryRecordSafe>,
    pub external_registry: Vec<ExternalRegistryRecordSafe>,
}

impl PluginRegistryV1Safe {
    #[inline(always)]
    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
        let mut data: &[u8] = data;
        let key = Key::deserialize(&mut data)?;
        if key != Key::PluginRegistryV1 {
            return Err(ErrorKind::InvalidInput.into());
        }

        let registry_size = u32::deserialize(&mut data)?;

        let mut registry = vec![];
        for _ in 0..registry_size {
            let plugin_type = u8::deserialize(&mut data)?;
            let authority = PluginAuthority::deserialize(&mut data)?;
            let offset = u64::deserialize(&mut data)?;

            registry.push(RegistryRecordSafe {
                plugin_type,
                authority,
                offset,
            });
        }

        let external_registry_size = u32::deserialize(&mut data)?;

        let mut external_registry = vec![];
        for _ in 0..external_registry_size {
            let plugin_type = u8::deserialize(&mut data)?;
            let authority = PluginAuthority::deserialize(&mut data)?;
            let lifecycle_checks =
                Option::<Vec<(u8, ExternalCheckResult)>>::deserialize(&mut data)?;
            let offset = u64::deserialize(&mut data)?;
            let data_offset = Option::<u64>::deserialize(&mut data)?;
            let data_len = Option::<u64>::deserialize(&mut data)?;

            external_registry.push(ExternalRegistryRecordSafe {
                plugin_type,
                authority,
                lifecycle_checks,
                offset,
                data_offset,
                data_len,
            });
        }

        Ok(Self {
            _key: key,
            registry,
            external_registry,
        })
    }
}

impl From<&ExternalPlugin> for ExternalPluginKey {
    fn from(plugin: &ExternalPlugin) -> Self {
        match plugin {
            ExternalPlugin::DataStore(data_store) => {
                ExternalPluginKey::DataStore(data_store.data_authority.clone())
            }
            ExternalPlugin::Oracle(oracle) => ExternalPluginKey::Oracle(oracle.base_address),
            ExternalPlugin::LifecycleHook(lifecycle_hook) => {
                ExternalPluginKey::LifecycleHook(lifecycle_hook.hooked_program)
            }
        }
    }
}
