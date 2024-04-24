use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use strum::EnumCount;

use crate::error::MplCoreError;

use super::{
    Authority, DataStore, DataStoreInitInfo, DataStoreUpdateInfo, ExternalCheckResult,
    LifecycleHook, LifecycleHookInitInfo, LifecycleHookUpdateInfo, Oracle, OracleInitInfo,
    OracleUpdateInfo, PluginValidation, PluginValidationContext, ValidationResult,
};

/// List of third party plugin types.
#[repr(C)]
#[derive(
    Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq, EnumCount, PartialOrd, Ord,
)]
pub enum ExternalPluginType {
    /// Lifecycle Hook.
    LifecycleHook,
    /// Oracle.
    Oracle,
    /// Data Store.
    DataStore,
}

impl From<&ExternalPluginKey> for ExternalPluginType {
    fn from(key: &ExternalPluginKey) -> Self {
        match key {
            ExternalPluginKey::LifecycleHook(_) => ExternalPluginType::LifecycleHook,
            ExternalPluginKey::Oracle(_) => ExternalPluginType::Oracle,
            ExternalPluginKey::DataStore(_) => ExternalPluginType::DataStore,
        }
    }
}

impl From<&ExternalPluginInitInfo> for ExternalPluginType {
    fn from(init_info: &ExternalPluginInitInfo) -> Self {
        match init_info {
            ExternalPluginInitInfo::LifecycleHook(_) => ExternalPluginType::LifecycleHook,
            ExternalPluginInitInfo::Oracle(_) => ExternalPluginType::Oracle,
            ExternalPluginInitInfo::DataStore(_) => ExternalPluginType::DataStore,
        }
    }
}

/// Definition of the external plugin variants, each containing a link to the external plugin
/// struct.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ExternalPlugin {
    /// Lifecycle Hook.  The hooked program and extra accounts are specified in the attached
    /// struct.  The hooked program is called at specified lifecycle events and will return a
    /// validation result and new data to store.
    LifecycleHook(LifecycleHook),
    /// Oracle.  Get a `ValidationResult` result from an account either specified by or derived
    /// from a `Pubkey` stored in the attached struct.
    Oracle(Oracle),
    /// Arbitrary data that can be written to by the data `Authority` stored in the attached
    /// struct.  Note this data authority is different then the plugin authority.
    DataStore(DataStore),
}

impl ExternalPlugin {
    /// Check if a plugin is permitted to approve or deny a create action.
    pub fn check_create(plugin: &ExternalPluginInitInfo) -> ExternalCheckResult {
        match plugin {
            ExternalPluginInitInfo::LifecycleHook(init_info) => {
                if let Some(lifecycle_checks) = &init_info.lifecycle_checks {
                    if let Some(checks) = lifecycle_checks
                        .iter()
                        .find(|event| event.0 == HookableLifecycleEvent::Create)
                    {
                        checks.1
                    } else {
                        ExternalCheckResult::none()
                    }
                } else {
                    ExternalCheckResult::none()
                }
            }
            ExternalPluginInitInfo::Oracle(init_info) => {
                if let Some(lifecycle_checks) = &init_info.lifecycle_checks {
                    if let Some(checks) = lifecycle_checks
                        .iter()
                        .find(|event| event.0 == HookableLifecycleEvent::Create)
                    {
                        checks.1
                    } else {
                        ExternalCheckResult::none()
                    }
                } else {
                    ExternalCheckResult::none()
                }
            }
            ExternalPluginInitInfo::DataStore(_) => ExternalCheckResult::none(),
        }
    }

    /// Validate the add external plugin lifecycle event.
    pub(crate) fn validate_create(
        init_info: &ExternalPluginInitInfo,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match init_info {
            ExternalPluginInitInfo::LifecycleHook(init_info) => init_info.validate_create(ctx),
            ExternalPluginInitInfo::Oracle(init_info) => init_info.validate_create(ctx),
            ExternalPluginInitInfo::DataStore(init_info) => init_info.validate_create(ctx),
        }
    }

    /// Validate the add external plugin lifecycle event.
    pub(crate) fn validate_add_external_plugin(
        init_info: &ExternalPluginInitInfo,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match init_info {
            ExternalPluginInitInfo::LifecycleHook(init_info) => {
                init_info.validate_add_external_plugin(ctx)
            }
            ExternalPluginInitInfo::Oracle(init_info) => {
                init_info.validate_add_external_plugin(ctx)
            }
            ExternalPluginInitInfo::DataStore(init_info) => {
                init_info.validate_add_external_plugin(ctx)
            }
        }
    }

    /// Load and deserialize a plugin from an offset in the account.
    pub fn load(account: &AccountInfo, offset: usize) -> Result<Self, ProgramError> {
        let mut bytes: &[u8] = &(*account.data).borrow()[offset..];
        Self::deserialize(&mut bytes).map_err(|error| {
            msg!("Error: {}", error);
            MplCoreError::DeserializationError.into()
        })
    }

    /// Save and serialize a plugin to an offset in the account.
    pub fn save(&self, account: &AccountInfo, offset: usize) -> ProgramResult {
        borsh::to_writer(&mut account.data.borrow_mut()[offset..], self).map_err(|error| {
            msg!("Error: {}", error);
            MplCoreError::SerializationError.into()
        })
    }
}

impl From<&ExternalPluginInitInfo> for ExternalPlugin {
    fn from(init_info: &ExternalPluginInitInfo) -> Self {
        match init_info {
            ExternalPluginInitInfo::LifecycleHook(init_info) => {
                ExternalPlugin::LifecycleHook(LifecycleHook::from(init_info))
            }
            ExternalPluginInitInfo::Oracle(init_info) => {
                ExternalPlugin::Oracle(Oracle::from(init_info))
            }
            ExternalPluginInitInfo::DataStore(init_info) => {
                ExternalPlugin::DataStore(DataStore::from(init_info))
            }
        }
    }
}

#[repr(C)]
#[derive(Eq, PartialEq, Clone, BorshSerialize, BorshDeserialize, Debug, PartialOrd, Ord, Hash)]
/// An enum listing all the lifecyle events available for external plugin hooks.  Note that some
/// lifecycle events such as adding and removing plugins will be checked by default as they are
/// inherently part of the external plugin system.
pub enum HookableLifecycleEvent {
    /// Add a plugin.
    Create,
    /// Transfer an Asset.
    Transfer,
    /// Burn an Asset or a Collection.
    Burn,
    /// Update an Asset or a Collection.
    Update,
}
/// Type used to specify extra accounts for external plugins.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ExtraAccount {
    /// Program-based PDA with seeds ["mpl-core"]
    PreconfiguredProgram {
        /// Account is a signer
        is_signer: bool,
        /// Account is writable.
        is_writable: bool,
    },
    /// Collection-based PDA with seeds ["mpl-core", <collection Pubkey>]
    PreconfiguredCollection {
        /// Account is a signer
        is_signer: bool,
        /// Account is writable.
        is_writable: bool,
    },
    /// Owner-based PDA with seeds ["mpl-core", <owner Pubkey>]
    PreconfiguredOwner {
        /// Account is a signer
        is_signer: bool,
        /// Account is writable.
        is_writable: bool,
    },
    /// Recipient-based PDA with seeds ["mpl-core", <recipient Pubkey>]
    /// If the lifecycle event has no recipient the derivation will fail.
    PreconfiguredRecipient {
        /// Account is a signer
        is_signer: bool,
        /// Account is writable.
        is_writable: bool,
    },
    /// Asset-based PDA with seeds ["mpl-core", <asset Pubkey>]
    PreconfiguredAsset {
        /// Account is a signer
        is_signer: bool,
        /// Account is writable.
        is_writable: bool,
    },
    /// PDA based on user-specified seeds.
    CustomPda {
        /// Seeds used to derive the PDA.
        seeds: Vec<Seed>,
        /// Account is a signer
        is_signer: bool,
        /// Account is writable.
        is_writable: bool,
    },
    /// Directly-specified address.
    Address {
        /// Address.
        address: Pubkey,
        /// Account is a signer
        is_signer: bool,
        /// Account is writable.
        is_writable: bool,
    },
}

/// Seeds to be used for extra account custom PDA derivations.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum Seed {
    /// Insert the program ID.
    Program,
    /// Insert the collection Pubkey.  If the asset has no collection the lifecycle action will
    /// fail.
    Collection,
    /// Insert the owner Pubkey.
    Owner,
    /// Insert the recipient Pubkey.  If the lifecycle event has no recipient the action will fail.
    Recipient,
    /// Insert the asset Pubkey.
    Asset,
    /// Insert the specified bytes.
    Bytes(Vec<u8>),
}

/// Schema used for third party plugin data.
#[repr(C)]
#[derive(Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq, Default)]
pub enum ExternalPluginSchema {
    /// Raw binary data.
    #[default]
    Binary,
    /// JSON.
    Json,
    /// MessagePack serialized data.
    MsgPack,
}

/// Information needed to initialize an external plugin.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ExternalPluginInitInfo {
    /// Lifecycle Hook.
    LifecycleHook(LifecycleHookInitInfo),
    /// Oracle.
    Oracle(OracleInitInfo),
    /// Data Store.
    DataStore(DataStoreInitInfo),
}

/// Information needed to update an external plugin.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ExternalPluginUpdateInfo {
    /// Lifecycle Hook.
    LifecycleHook(LifecycleHookUpdateInfo),
    /// Oracle.
    Oracle(OracleUpdateInfo),
    /// Data Store.
    DataStore(DataStoreUpdateInfo),
}

/// Key used to uniquely specify an external plugin after it is created.
#[repr(C)]
#[derive(
    Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq, EnumCount, PartialOrd, Ord,
)]
pub enum ExternalPluginKey {
    /// Lifecycle Hook.
    LifecycleHook(Pubkey),
    /// Oracle.
    Oracle(Pubkey),
    /// Data Store.
    DataStore(Authority),
}
