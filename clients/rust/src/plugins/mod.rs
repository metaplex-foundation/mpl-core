pub mod utils;

pub use utils::*;

use crate::{
    types::{
        Attributes, BurnDelegate, DataStore, Edition, FreezeDelegate, LifecycleHook, MasterEdition,
        Oracle, PermanentBurnDelegate, PermanentFreezeDelegate, PermanentTransferDelegate,
        Royalties, TransferDelegate, UpdateDelegate,
    },
    BaseAuthority,
};

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

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MasterEditionPlugin {
    pub base: BasePlugin,
    pub master_edition: MasterEdition,
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
    pub master_edition: Option<MasterEditionPlugin>,
}

#[derive(Debug, Default)]
pub struct ExternalPluginsList {
    pub lifecycle_hooks: Vec<LifecycleHook>,
    pub oracles: Vec<Oracle>,
    pub data_stores: Vec<DataStore>,
}
