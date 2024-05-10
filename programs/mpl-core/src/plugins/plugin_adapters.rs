use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};
use strum::EnumCount;

use crate::{
    error::MplCoreError,
    state::{AssetV1, SolanaAccount},
};

use super::{
    AdapterCheckResult, AdapterRegistryRecord, Authority, DataStore, DataStoreInitInfo,
    DataStoreUpdateInfo, LifecycleHook, LifecycleHookInitInfo, LifecycleHookUpdateInfo, Oracle,
    OracleInitInfo, OracleUpdateInfo, PluginValidation, PluginValidationContext, ValidationResult,
};

/// List of third party plugin types.
#[repr(C)]
#[derive(
    Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq, EnumCount, PartialOrd, Ord,
)]
pub enum PluginAdapterType {
    /// Lifecycle Hook.
    LifecycleHook,
    /// Oracle.
    Oracle,
    /// Data Store.
    DataStore,
}

impl From<&PluginAdapterKey> for PluginAdapterType {
    fn from(key: &PluginAdapterKey) -> Self {
        match key {
            PluginAdapterKey::LifecycleHook(_) => PluginAdapterType::LifecycleHook,
            PluginAdapterKey::Oracle(_) => PluginAdapterType::Oracle,
            PluginAdapterKey::DataStore(_) => PluginAdapterType::DataStore,
        }
    }
}

impl From<&PluginAdapterInitInfo> for PluginAdapterType {
    fn from(init_info: &PluginAdapterInitInfo) -> Self {
        match init_info {
            PluginAdapterInitInfo::LifecycleHook(_) => PluginAdapterType::LifecycleHook,
            PluginAdapterInitInfo::Oracle(_) => PluginAdapterType::Oracle,
            PluginAdapterInitInfo::DataStore(_) => PluginAdapterType::DataStore,
        }
    }
}

/// Definition of the plugin adapter variants, each containing a link to the plugin adapter
/// struct.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum PluginAdapter {
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

impl PluginAdapter {
    /// Update the plugin from the update info.
    pub fn update(&mut self, update_info: &PluginAdapterUpdateInfo) {
        match (self, update_info) {
            (
                PluginAdapter::LifecycleHook(lifecycle_hook),
                PluginAdapterUpdateInfo::LifecycleHook(update_info),
            ) => {
                lifecycle_hook.update(update_info);
            }
            (PluginAdapter::Oracle(oracle), PluginAdapterUpdateInfo::Oracle(update_info)) => {
                oracle.update(update_info);
            }
            (
                PluginAdapter::DataStore(data_store),
                PluginAdapterUpdateInfo::DataStore(update_info),
            ) => {
                data_store.update(update_info);
            }
            _ => unreachable!(),
        }
    }

    /// Check if a plugin is permitted to approve or deny a create action.
    pub fn check_create(plugin: &PluginAdapterInitInfo) -> AdapterCheckResult {
        match plugin {
            PluginAdapterInitInfo::LifecycleHook(init_info) => {
                if let Some(checks) = init_info
                    .lifecycle_checks
                    .iter()
                    .find(|event| event.0 == HookableLifecycleEvent::Create)
                {
                    checks.1
                } else {
                    AdapterCheckResult::none()
                }
            }
            PluginAdapterInitInfo::Oracle(init_info) => {
                if let Some(checks) = init_info
                    .lifecycle_checks
                    .iter()
                    .find(|event| event.0 == HookableLifecycleEvent::Create)
                {
                    checks.1
                } else {
                    AdapterCheckResult::none()
                }
            }
            PluginAdapterInitInfo::DataStore(_) => AdapterCheckResult::none(),
        }
    }

    /// Validate the add plugin adapter lifecycle event.
    pub(crate) fn validate_create(
        plugin_adapter: &PluginAdapter,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin_adapter {
            PluginAdapter::LifecycleHook(lifecycle_hook) => lifecycle_hook.validate_create(ctx),
            PluginAdapter::Oracle(oracle) => oracle.validate_create(ctx),
            PluginAdapter::DataStore(data_store) => data_store.validate_create(ctx),
        }
    }

    /// Route the validation of the update action to the appropriate plugin.
    pub(crate) fn validate_update(
        plugin_adapter: &PluginAdapter,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin_adapter {
            PluginAdapter::LifecycleHook(lifecycle_hook) => lifecycle_hook.validate_update(ctx),
            PluginAdapter::Oracle(oracle) => oracle.validate_update(ctx),
            PluginAdapter::DataStore(data_store) => data_store.validate_update(ctx),
        }
    }

    /// Route the validation of the burn action to the appropriate plugin.
    pub(crate) fn validate_burn(
        plugin_adapter: &PluginAdapter,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin_adapter {
            PluginAdapter::LifecycleHook(lifecycle_hook) => lifecycle_hook.validate_burn(ctx),
            PluginAdapter::Oracle(oracle) => oracle.validate_burn(ctx),
            PluginAdapter::DataStore(data_store) => data_store.validate_burn(ctx),
        }
    }

    /// Route the validation of the transfer action to the appropriate plugin adapter.
    pub(crate) fn validate_transfer(
        plugin_adapter: &PluginAdapter,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin_adapter {
            PluginAdapter::LifecycleHook(lifecycle_hook) => lifecycle_hook.validate_transfer(ctx),
            PluginAdapter::Oracle(oracle) => oracle.validate_transfer(ctx),
            PluginAdapter::DataStore(data_store) => data_store.validate_transfer(ctx),
        }
    }

    /// Validate the add plugin adapter lifecycle event.
    pub(crate) fn validate_add_plugin_adapter(
        plugin_adapter: &PluginAdapter,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match plugin_adapter {
            PluginAdapter::LifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_add_plugin_adapter(ctx)
            }
            PluginAdapter::Oracle(oracle) => oracle.validate_add_plugin_adapter(ctx),
            PluginAdapter::DataStore(data_store) => data_store.validate_add_plugin_adapter(ctx),
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

impl From<&PluginAdapterInitInfo> for PluginAdapter {
    fn from(init_info: &PluginAdapterInitInfo) -> Self {
        match init_info {
            PluginAdapterInitInfo::LifecycleHook(init_info) => {
                PluginAdapter::LifecycleHook(LifecycleHook::from(init_info))
            }
            PluginAdapterInitInfo::Oracle(init_info) => {
                PluginAdapter::Oracle(Oracle::from(init_info))
            }
            PluginAdapterInitInfo::DataStore(init_info) => {
                PluginAdapter::DataStore(DataStore::from(init_info))
            }
        }
    }
}

#[repr(C)]
#[derive(Eq, PartialEq, Clone, BorshSerialize, BorshDeserialize, Debug, PartialOrd, Ord, Hash)]
/// An enum listing all the lifecyle events available for plugin adapter hooks.  Note that some
/// lifecycle events such as adding and removing plugins will be checked by default as they are
/// inherently part of the plugin adapter system.
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

/// Prefix used with some of the `ExtraAccounts` that are PDAs.
pub const MPL_CORE_PREFIX: &str = "mpl-core";

/// Type used to specify extra accounts for plugin adapters.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ExtraAccount {
    /// Program-based PDA with seeds \["mpl-core"\]
    PreconfiguredProgram {
        /// Account is a signer
        is_signer: bool,
        /// Account is writable.
        is_writable: bool,
    },
    /// Collection-based PDA with seeds \["mpl-core", collection_pubkey\]
    PreconfiguredCollection {
        /// Account is a signer
        is_signer: bool,
        /// Account is writable.
        is_writable: bool,
    },
    /// Owner-based PDA with seeds \["mpl-core", owner_pubkey\]
    PreconfiguredOwner {
        /// Account is a signer
        is_signer: bool,
        /// Account is writable.
        is_writable: bool,
    },
    /// Recipient-based PDA with seeds \["mpl-core", recipient_pubkey\]
    /// If the lifecycle event has no recipient the derivation will fail.
    PreconfiguredRecipient {
        /// Account is a signer
        is_signer: bool,
        /// Account is writable.
        is_writable: bool,
    },
    /// Asset-based PDA with seeds \["mpl-core", asset_pubkey\]
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
        /// Program ID if not the base address/program ID for the external plugin.
        custom_program_id: Option<Pubkey>,
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

impl ExtraAccount {
    pub(crate) fn derive(
        &self,
        program_id: &Pubkey,
        ctx: &PluginValidationContext,
    ) -> Result<Pubkey, ProgramError> {
        match self {
            ExtraAccount::PreconfiguredProgram { .. } => {
                let seeds = &[MPL_CORE_PREFIX.as_bytes()];
                let (pubkey, _bump) = Pubkey::find_program_address(seeds, program_id);
                Ok(pubkey)
            }
            ExtraAccount::PreconfiguredCollection { .. } => {
                let collection = ctx
                    .collection_info
                    .ok_or(MplCoreError::MissingCollection)?
                    .key;
                let seeds = &[MPL_CORE_PREFIX.as_bytes(), collection.as_ref()];
                let (pubkey, _bump) = Pubkey::find_program_address(seeds, program_id);
                Ok(pubkey)
            }
            ExtraAccount::PreconfiguredOwner { .. } => {
                let asset_info = ctx.asset_info.ok_or(MplCoreError::MissingAsset)?;
                let owner = AssetV1::load(asset_info, 0)?.owner;
                let seeds = &[MPL_CORE_PREFIX.as_bytes(), owner.as_ref()];
                let (pubkey, _bump) = Pubkey::find_program_address(seeds, program_id);
                Ok(pubkey)
            }
            ExtraAccount::PreconfiguredRecipient { .. } => {
                let recipient = ctx.new_owner.ok_or(MplCoreError::MissingNewOwner)?.key;
                let seeds = &[MPL_CORE_PREFIX.as_bytes(), recipient.as_ref()];
                let (pubkey, _bump) = Pubkey::find_program_address(seeds, program_id);
                Ok(pubkey)
            }
            ExtraAccount::PreconfiguredAsset { .. } => {
                let asset = ctx.asset_info.ok_or(MplCoreError::MissingAsset)?.key;
                let seeds = &[MPL_CORE_PREFIX.as_bytes(), asset.as_ref()];
                let (pubkey, _bump) = Pubkey::find_program_address(seeds, program_id);
                Ok(pubkey)
            }
            ExtraAccount::CustomPda {
                seeds,
                custom_program_id,
                ..
            } => {
                let seeds = transform_seeds(seeds, ctx)?;

                // Convert the Vec of Vec into Vec of u8 slices.
                let vec_of_slices: Vec<&[u8]> = seeds.iter().map(Vec::as_slice).collect();

                let (pubkey, _bump) = Pubkey::find_program_address(
                    &vec_of_slices,
                    custom_program_id.as_ref().unwrap_or(program_id),
                );
                Ok(pubkey)
            }
            ExtraAccount::Address { address, .. } => Ok(*address),
        }
    }
}

// Transform seeds from their tokens into actual seeds based on passed-in context values.
fn transform_seeds(
    seeds: &Vec<Seed>,
    ctx: &PluginValidationContext,
) -> Result<Vec<Vec<u8>>, ProgramError> {
    let mut transformed_seeds = Vec::<Vec<u8>>::new();

    for seed in seeds {
        match seed {
            Seed::Collection => {
                let collection = ctx
                    .collection_info
                    .ok_or(MplCoreError::MissingCollection)?
                    .key
                    .as_ref()
                    .to_vec();
                transformed_seeds.push(collection);
            }
            Seed::Owner => {
                let asset_info = ctx.asset_info.ok_or(MplCoreError::MissingAsset)?;
                let owner = AssetV1::load(asset_info, 0)?.owner.as_ref().to_vec();
                transformed_seeds.push(owner);
            }
            Seed::Recipient => {
                let recipient = ctx
                    .new_owner
                    .ok_or(MplCoreError::MissingNewOwner)?
                    .key
                    .as_ref()
                    .to_vec();
                transformed_seeds.push(recipient);
            }
            Seed::Asset => {
                let asset = ctx
                    .asset_info
                    .ok_or(MplCoreError::MissingAsset)?
                    .key
                    .as_ref()
                    .to_vec();
                transformed_seeds.push(asset);
            }
            Seed::Address(pubkey) => {
                transformed_seeds.push(pubkey.as_ref().to_vec());
            }
            Seed::Bytes(val) => {
                transformed_seeds.push(val.clone());
            }
        }
    }

    Ok(transformed_seeds)
}

/// Seeds to be used for extra account custom PDA derivations.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum Seed {
    /// Insert the collection `Pubkey`.  If the asset has no collection the lifecycle action will
    /// fail.
    Collection,
    /// Insert the owner `Pubkey`.
    Owner,
    /// Insert the recipient `Pubkey`.  If the lifecycle event has no recipient the action will fail.
    Recipient,
    /// Insert the asset `Pubkey`.
    Asset,
    /// Insert the specified `Pubkey`.
    Address(Pubkey),
    /// Insert the specified bytes.
    Bytes(Vec<u8>),
}

/// Schema used for third party plugin data.
#[repr(C)]
#[derive(Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq, Default)]
pub enum PluginAdapterSchema {
    /// Raw binary data.
    #[default]
    Binary,
    /// JSON.
    Json,
    /// MessagePack serialized data.
    MsgPack,
}

/// Information needed to initialize a plugin adapter.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum PluginAdapterInitInfo {
    /// Lifecycle Hook.
    LifecycleHook(LifecycleHookInitInfo),
    /// Oracle.
    Oracle(OracleInitInfo),
    /// Data Store.
    DataStore(DataStoreInitInfo),
}

/// Information needed to update a plugin adapter.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum PluginAdapterUpdateInfo {
    /// Lifecycle Hook.
    LifecycleHook(LifecycleHookUpdateInfo),
    /// Oracle.
    Oracle(OracleUpdateInfo),
    /// Data Store.
    DataStore(DataStoreUpdateInfo),
}

/// Key used to uniquely specify a plugin adapter after it is created.
#[repr(C)]
#[derive(
    Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq, EnumCount, PartialOrd, Ord,
)]
pub enum PluginAdapterKey {
    /// Lifecycle Hook.
    LifecycleHook(Pubkey),
    /// Oracle.
    Oracle(Pubkey),
    /// Data Store.
    DataStore(Authority),
}

impl PluginAdapterKey {
    pub(crate) fn from_record(
        account: &AccountInfo,
        adapter_registry_record: &AdapterRegistryRecord,
    ) -> Result<Self, ProgramError> {
        let pubkey_or_authority_offset = adapter_registry_record
            .offset
            .checked_add(1)
            .ok_or(MplCoreError::NumericalOverflow)?;

        match adapter_registry_record.plugin_type {
            PluginAdapterType::LifecycleHook => {
                let pubkey =
                    Pubkey::deserialize(&mut &account.data.borrow()[pubkey_or_authority_offset..])?;
                Ok(Self::LifecycleHook(pubkey))
            }
            PluginAdapterType::Oracle => {
                let pubkey =
                    Pubkey::deserialize(&mut &account.data.borrow()[pubkey_or_authority_offset..])?;
                Ok(Self::Oracle(pubkey))
            }
            PluginAdapterType::DataStore => {
                let authority = Authority::deserialize(
                    &mut &account.data.borrow()[pubkey_or_authority_offset..],
                )?;
                Ok(Self::DataStore(authority))
            }
        }
    }
}

impl From<&PluginAdapterInitInfo> for PluginAdapterKey {
    fn from(init_info: &PluginAdapterInitInfo) -> Self {
        match init_info {
            PluginAdapterInitInfo::LifecycleHook(init_info) => {
                PluginAdapterKey::LifecycleHook(init_info.hooked_program)
            }
            PluginAdapterInitInfo::Oracle(init_info) => {
                PluginAdapterKey::Oracle(init_info.base_address)
            }
            PluginAdapterInitInfo::DataStore(init_info) => {
                PluginAdapterKey::DataStore(init_info.data_authority)
            }
        }
    }
}
