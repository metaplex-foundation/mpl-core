use solana_program::pubkey::Pubkey;

use crate::{
    accounts::{BaseAsset, BaseCollection, PluginHeader},
    types::{
        Attributes, Authority, Burn, Freeze, PermanentFreeze, Royalties, Transfer, UpdateDelegate,
    },
};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum AuthorityType {
    None,
    Owner,
    UpdateAuthority,
    Pubkey,
}

impl From<Authority> for AuthorityType {
    fn from(authority: Authority) -> Self {
        match authority {
            Authority::None => AuthorityType::None,
            Authority::Owner => AuthorityType::Owner,
            Authority::UpdateAuthority => AuthorityType::UpdateAuthority,
            Authority::Pubkey { address: _ } => AuthorityType::Pubkey,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct BaseAuthority {
    pub authority_type: AuthorityType,
    pub address: Option<Pubkey>,
}

impl From<Authority> for BaseAuthority {
    fn from(authority: Authority) -> Self {
        match authority {
            Authority::None => BaseAuthority {
                authority_type: AuthorityType::None,
                address: None,
            },
            Authority::Owner => BaseAuthority {
                authority_type: AuthorityType::Owner,
                address: None,
            },
            Authority::UpdateAuthority => BaseAuthority {
                authority_type: AuthorityType::UpdateAuthority,
                address: None,
            },
            Authority::Pubkey { address } => BaseAuthority {
                authority_type: AuthorityType::Pubkey,
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

#[derive(Debug)]
pub struct RoyaltiesPlugin {
    pub base: BasePlugin,
    pub royalties: Royalties,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct FreezePlugin {
    pub base: BasePlugin,
    pub freeze: Freeze,
}

#[derive(Debug)]
pub struct BurnPlugin {
    pub base: BasePlugin,
    pub burn: Burn,
}

#[derive(Debug)]
pub struct TransferPlugin {
    pub base: BasePlugin,
    pub transfer: Transfer,
}

#[derive(Debug)]
pub struct UpdateDelegatePlugin {
    pub base: BasePlugin,
    pub update_delegate: UpdateDelegate,
}

#[derive(Debug)]
pub struct PermanentFreezePlugin {
    pub base: BasePlugin,
    pub permanent_freeze: PermanentFreeze,
}

#[derive(Debug)]
pub struct AttributesPlugin {
    pub base: BasePlugin,
    pub attributes: Attributes,
}

#[derive(Debug, Default)]
pub struct PluginsList {
    pub royalties: Option<RoyaltiesPlugin>,
    pub freeze: Option<FreezePlugin>,
    pub burn: Option<BurnPlugin>,
    pub transfer: Option<TransferPlugin>,
    pub update_delegate: Option<UpdateDelegatePlugin>,
    pub permanent_freeze: Option<PermanentFreezePlugin>,
    pub attributes: Option<AttributesPlugin>,
}

#[derive(Debug)]
pub struct Asset {
    pub base: BaseAsset,
    pub plugin_list: PluginsList,
    pub plugin_header: Option<PluginHeader>,
}

#[derive(Debug)]
pub struct Collection {
    pub base: BaseCollection,
    pub plugin_list: PluginsList,
    pub plugin_header: PluginHeader,
}
