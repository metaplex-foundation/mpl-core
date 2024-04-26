use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use strum::EnumCount;

use crate::error::MplCoreError;

use super::{
    Authority, DataStore, DataStoreInitInfo, DataStoreUpdateInfo, ExternalCheckResult,
    ExternalRegistryRecord, LifecycleHook, LifecycleHookInitInfo, LifecycleHookUpdateInfo, Oracle,
    OracleInitInfo, OracleUpdateInfo, PluginValidation, PluginValidationContext, ValidationResult,
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
    /// Update the plugin from the update info.
    pub fn update(&mut self, update_info: &ExternalPluginUpdateInfo) {
        match (self, update_info) {
            (
                ExternalPlugin::LifecycleHook(lifecycle_hook),
                ExternalPluginUpdateInfo::LifecycleHook(update_info),
            ) => {
                lifecycle_hook.update(update_info);
            }
            (ExternalPlugin::Oracle(oracle), ExternalPluginUpdateInfo::Oracle(update_info)) => {
                oracle.update(update_info);
            }
            (
                ExternalPlugin::DataStore(data_store),
                ExternalPluginUpdateInfo::DataStore(update_info),
            ) => {
                data_store.update(update_info);
            }
            _ => unreachable!(),
        }
    }

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
        external_plugin: &ExternalPlugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match external_plugin {
            ExternalPlugin::LifecycleHook(lifecycle_hook) => lifecycle_hook.validate_create(ctx),
            ExternalPlugin::Oracle(oracle) => oracle.validate_create(ctx),
            ExternalPlugin::DataStore(data_store) => data_store.validate_create(ctx),
        }
    }

    /// Route the validation of the update action to the appropriate plugin.
    pub(crate) fn validate_update(
        external_plugin: &ExternalPlugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match external_plugin {
            ExternalPlugin::LifecycleHook(lifecycle_hook) => lifecycle_hook.validate_update(ctx),
            ExternalPlugin::Oracle(oracle) => oracle.validate_update(ctx),
            ExternalPlugin::DataStore(data_store) => data_store.validate_update(ctx),
        }
    }

    /// Route the validation of the burn action to the appropriate plugin.
    pub(crate) fn validate_burn(
        external_plugin: &ExternalPlugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match external_plugin {
            ExternalPlugin::LifecycleHook(lifecycle_hook) => lifecycle_hook.validate_burn(ctx),
            ExternalPlugin::Oracle(oracle) => oracle.validate_burn(ctx),
            ExternalPlugin::DataStore(data_store) => data_store.validate_burn(ctx),
        }
    }

    /// Route the validation of the transfer action to the appropriate external plugin.
    pub(crate) fn validate_transfer(
        external_plugin: &ExternalPlugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match external_plugin {
            ExternalPlugin::LifecycleHook(lifecycle_hook) => lifecycle_hook.validate_transfer(ctx),
            ExternalPlugin::Oracle(oracle) => oracle.validate_transfer(ctx),
            ExternalPlugin::DataStore(data_store) => data_store.validate_transfer(ctx),
        }
    }

    /// Validate the add external plugin lifecycle event.
    pub(crate) fn validate_add_external_plugin(
        external_plugin: &ExternalPlugin,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match external_plugin {
            ExternalPlugin::LifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_add_external_plugin(ctx)
            }
            ExternalPlugin::Oracle(oracle) => oracle.validate_add_external_plugin(ctx),
            ExternalPlugin::DataStore(data_store) => data_store.validate_add_external_plugin(ctx),
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

impl ExternalPluginKey {
    pub(crate) fn from_record(
        account: &AccountInfo,
        external_registry_record: &ExternalRegistryRecord,
    ) -> Result<Self, ProgramError> {
        let pubkey_or_authority_offset = external_registry_record
            .offset
            .checked_add(1)
            .ok_or(MplCoreError::NumericalOverflow)?;

        match external_registry_record.plugin_type {
            ExternalPluginType::LifecycleHook => {
                let pubkey =
                    Pubkey::deserialize(&mut &account.data.borrow()[pubkey_or_authority_offset..])?;
                Ok(Self::LifecycleHook(pubkey))
            }
            ExternalPluginType::Oracle => {
                let pubkey =
                    Pubkey::deserialize(&mut &account.data.borrow()[pubkey_or_authority_offset..])?;
                Ok(Self::Oracle(pubkey))
            }
            ExternalPluginType::DataStore => {
                let authority = Authority::deserialize(
                    &mut &account.data.borrow()[pubkey_or_authority_offset..],
                )?;
                Ok(Self::DataStore(authority))
            }
        }
    }
}
