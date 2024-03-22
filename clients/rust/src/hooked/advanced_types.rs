use solana_program::pubkey::Pubkey;

use crate::{
    accounts::{BaseAssetV1, BaseCollectionV1, PluginHeaderV1},
    types::{
        Attributes, BurnDelegate, FreezeDelegate, PermanentBurnDelegate, PermanentFreezeDelegate,
        PermanentTransferDelegate, PluginAuthority, Royalties, TransferDelegate, UpdateDelegate,
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
    pub plugin_header: PluginHeaderV1,
}
