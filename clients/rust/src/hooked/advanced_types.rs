use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;
use std::{cmp::Ordering, io::ErrorKind};

use crate::{
    accounts::{BaseAssetV1, BaseCollectionV1, PluginHeaderV1},
    types::{
        Attributes, BurnDelegate, Edition, FreezeDelegate, Key, PermanentBurnDelegate,
        PermanentFreezeDelegate, PermanentTransferDelegate, PluginAuthority, Royalties,
        TransferDelegate, UpdateDelegate,
    },
};

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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct BasePlugin {
    pub authority: BaseAuthority,
    pub offset: Option<u64>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RoyaltiesPlugin {
    pub base: BasePlugin,
    pub royalties: Royalties,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct FreezeDelegatePlugin {
    pub base: BasePlugin,
    pub freeze_delegate: FreezeDelegate,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct BurnDelegatePlugin {
    pub base: BasePlugin,
    pub burn_delegate: BurnDelegate,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TransferDelegatePlugin {
    pub base: BasePlugin,
    pub transfer_delegate: TransferDelegate,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UpdateDelegatePlugin {
    pub base: BasePlugin,
    pub update_delegate: UpdateDelegate,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PermanentFreezeDelegatePlugin {
    pub base: BasePlugin,
    pub permanent_freeze_delegate: PermanentFreezeDelegate,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AttributesPlugin {
    pub base: BasePlugin,
    pub attributes: Attributes,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PermanentTransferDelegatePlugin {
    pub base: BasePlugin,
    pub permanent_transfer_delegate: PermanentTransferDelegate,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PermanentBurnDelegatePlugin {
    pub base: BasePlugin,
    pub permanent_burn_delegate: PermanentBurnDelegate,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct EditionPlugin {
    pub base: BasePlugin,
    pub edition: Edition,
}

#[derive(Debug, Default)]
pub struct PluginsList {
    pub royalties: Option<RoyaltiesPlugin>,
    pub freeze_delegate: Option<FreezeDelegatePlugin>,
    pub burn_delegate: Option<BurnDelegatePlugin>,
    pub transfer_delegate: Option<TransferDelegatePlugin>,
    pub update_delegate: Option<UpdateDelegatePlugin>,
    pub permanent_freeze_delegate: Option<PermanentFreezeDelegatePlugin>,
    pub attributes: Option<AttributesPlugin>,
    pub permanent_transfer_delegate: Option<PermanentTransferDelegatePlugin>,
    pub permanent_burn_delegate: Option<PermanentBurnDelegatePlugin>,
    pub edition: Option<EditionPlugin>,
}

#[derive(Debug)]
pub struct Asset {
    pub base: BaseAssetV1,
    pub plugin_list: PluginsList,
    pub plugin_header: Option<PluginHeaderV1>,
}

#[derive(Debug)]
pub struct Collection {
    pub base: BaseCollectionV1,
    pub plugin_list: PluginsList,
    pub plugin_header: Option<PluginHeaderV1>,
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

/// Plugin registry that an account can safely be deserialized into even if some plugins are
/// not known.  Note this skips over external plugins for now, and will be updated when those
/// are defined.
pub struct PluginRegistryV1Safe {
    pub _key: Key,
    pub registry: Vec<RegistryRecordSafe>,
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

        Ok(Self {
            _key: key,
            registry,
        })
    }
}
