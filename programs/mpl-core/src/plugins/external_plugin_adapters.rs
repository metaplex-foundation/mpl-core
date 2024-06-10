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
    AssetLinkedLifecycleHook, AssetLinkedLifecycleHookInitInfo, AssetLinkedSecureDataStore,
    AssetLinkedSecureDataStoreInitInfo, AssetLinkedSecureDataStoreUpdateInfo, Authority,
    DataSection, DataSectionInitInfo, ExternalCheckResult, ExternalRegistryRecord, LifecycleHook,
    LifecycleHookInitInfo, LifecycleHookUpdateInfo, Oracle, OracleInitInfo, OracleUpdateInfo,
    PluginValidation, PluginValidationContext, SecureDataStore, SecureDataStoreInitInfo,
    SecureDataStoreUpdateInfo, ValidationResult,
};

/// List of third party plugin types.
#[repr(C)]
#[derive(
    Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq, EnumCount, PartialOrd, Ord,
)]
pub enum ExternalPluginAdapterType {
    /// Lifecycle Hook.
    LifecycleHook,
    /// Oracle.
    Oracle,
    /// Secure Store.
    SecureDataStore,
    /// Asset Linked Lifecycle Hook.
    AssetLinkedLifecycleHook,
    /// Asset Linked Secure Store.
    AssetLinkedSecureDataStore,
    /// Data Section.
    DataSection,
}

impl From<&ExternalPluginAdapterKey> for ExternalPluginAdapterType {
    fn from(key: &ExternalPluginAdapterKey) -> Self {
        match key {
            ExternalPluginAdapterKey::LifecycleHook(_) => ExternalPluginAdapterType::LifecycleHook,
            ExternalPluginAdapterKey::AssetLinkedLifecycleHook(_) => {
                ExternalPluginAdapterType::AssetLinkedLifecycleHook
            }
            ExternalPluginAdapterKey::Oracle(_) => ExternalPluginAdapterType::Oracle,
            ExternalPluginAdapterKey::SecureDataStore(_) => {
                ExternalPluginAdapterType::SecureDataStore
            }
            ExternalPluginAdapterKey::AssetLinkedSecureDataStore(_) => {
                ExternalPluginAdapterType::AssetLinkedSecureDataStore
            }
            ExternalPluginAdapterKey::DataSection(_) => ExternalPluginAdapterType::DataSection,
        }
    }
}

impl From<&ExternalPluginAdapterInitInfo> for ExternalPluginAdapterType {
    fn from(init_info: &ExternalPluginAdapterInitInfo) -> Self {
        match init_info {
            ExternalPluginAdapterInitInfo::LifecycleHook(_) => {
                ExternalPluginAdapterType::LifecycleHook
            }
            ExternalPluginAdapterInitInfo::Oracle(_) => ExternalPluginAdapterType::Oracle,
            ExternalPluginAdapterInitInfo::SecureDataStore(_) => {
                ExternalPluginAdapterType::SecureDataStore
            }
            ExternalPluginAdapterInitInfo::AssetLinkedSecureDataStore(_) => {
                ExternalPluginAdapterType::AssetLinkedSecureDataStore
            }
            ExternalPluginAdapterInitInfo::DataSection(_) => ExternalPluginAdapterType::DataSection,
        }
    }
}

/// Definition of the external plugin adapter variants, each containing a link to the external plugin adapter
/// struct.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ExternalPluginAdapter {
    /// Lifecycle Hook.  The hooked program and extra accounts are specified in the attached
    /// struct.  The hooked program is called at specified lifecycle events and will return a
    /// validation result and new data to store.
    LifecycleHook(LifecycleHook),
    /// Oracle.  Get a `ValidationResult` result from an account either specified by or derived
    /// from a `Pubkey` stored in the attached struct.
    Oracle(Oracle),
    /// Collection Only: Asset-Linked Lifecycle Hook.  The hooked program and extra accounts are specified in the attached
    /// struct.  The hooked program is called at specified lifecycle events and will return a
    /// validation result and new data to store.
    AssetLinkedLifecycleHook(AssetLinkedLifecycleHook),
    /// Arbitrary data that can be written to by the data `Authority` stored in the attached
    /// struct.  Note this data authority is different then the plugin authority.
    SecureDataStore(SecureDataStore),
    /// Collection only: Arbitrary data that can be written to by the data `Authority` stored on any asset in the Collection in the Data Section struct.
    /// Authority is different then the plugin authority.
    AssetLinkedSecureDataStore(AssetLinkedSecureDataStore),
    /// Data Section.  This is a special plugin that is used to contain the data of other external
    /// plugins.
    DataSection(DataSection),
}

impl ExternalPluginAdapter {
    /// Update the plugin from the update info.
    pub fn update(&mut self, update_info: &ExternalPluginAdapterUpdateInfo) {
        match (self, update_info) {
            (
                ExternalPluginAdapter::LifecycleHook(lifecycle_hook),
                ExternalPluginAdapterUpdateInfo::LifecycleHook(update_info),
            ) => {
                lifecycle_hook.update(update_info);
            }
            (
                ExternalPluginAdapter::Oracle(oracle),
                ExternalPluginAdapterUpdateInfo::Oracle(update_info),
            ) => {
                oracle.update(update_info);
            }
            (
                ExternalPluginAdapter::SecureDataStore(secure_data_store),
                ExternalPluginAdapterUpdateInfo::SecureDataStore(update_info),
            ) => {
                secure_data_store.update(update_info);
            }
            _ => unreachable!(),
        }
    }

    /// Check if a plugin is permitted to approve or deny a create action.
    pub fn check_create(plugin: &ExternalPluginAdapterInitInfo) -> ExternalCheckResult {
        match plugin {
            ExternalPluginAdapterInitInfo::LifecycleHook(init_info) => {
                if let Some(checks) = init_info
                    .lifecycle_checks
                    .iter()
                    .find(|event| event.0 == HookableLifecycleEvent::Create)
                {
                    checks.1
                } else {
                    ExternalCheckResult::none()
                }
            }
            ExternalPluginAdapterInitInfo::Oracle(init_info) => {
                if let Some(checks) = init_info
                    .lifecycle_checks
                    .iter()
                    .find(|event| event.0 == HookableLifecycleEvent::Create)
                {
                    checks.1
                } else {
                    ExternalCheckResult::none()
                }
            }
            ExternalPluginAdapterInitInfo::SecureDataStore(_) => ExternalCheckResult::none(),
            ExternalPluginAdapterInitInfo::AssetLinkedSecureDataStore(_) => {
                ExternalCheckResult::none()
            }
            ExternalPluginAdapterInitInfo::DataSection(_) => ExternalCheckResult::none(),
        }
    }

    /// Validate the add external plugin adapter lifecycle event.
    pub(crate) fn validate_create(
        external_plugin_adapter: &ExternalPluginAdapter,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        solana_program::msg!("ExternalPluginAdapter::validate_create");
        match external_plugin_adapter {
            ExternalPluginAdapter::LifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_create(ctx)
            }
            ExternalPluginAdapter::Oracle(oracle) => oracle.validate_create(ctx),
            ExternalPluginAdapter::SecureDataStore(data_store) => data_store.validate_create(ctx),
            ExternalPluginAdapter::AssetLinkedSecureDataStore(data_store) => {
                data_store.validate_create(ctx)
            }
            // This should be unreachable because no corresponding it cannot be added by a user.
            ExternalPluginAdapter::DataSection(_) => Ok(ValidationResult::Rejected),
        }
    }

    /// Route the validation of the update action to the appropriate plugin.
    pub(crate) fn validate_update(
        external_plugin_adapter: &ExternalPluginAdapter,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match external_plugin_adapter {
            ExternalPluginAdapter::LifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_update(ctx)
            }
            ExternalPluginAdapter::Oracle(oracle) => oracle.validate_update(ctx),
            ExternalPluginAdapter::SecureDataStore(data_store) => data_store.validate_update(ctx),
            ExternalPluginAdapter::AssetLinkedSecureDataStore(data_store) => {
                data_store.validate_update(ctx)
            }
            ExternalPluginAdapter::DataSection(_) => Ok(ValidationResult::Pass),
        }
    }

    /// Route the validation of the burn action to the appropriate plugin.
    pub(crate) fn validate_burn(
        external_plugin_adapter: &ExternalPluginAdapter,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match external_plugin_adapter {
            ExternalPluginAdapter::LifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_burn(ctx)
            }
            ExternalPluginAdapter::Oracle(oracle) => oracle.validate_burn(ctx),
            ExternalPluginAdapter::SecureDataStore(data_store) => data_store.validate_burn(ctx),
            ExternalPluginAdapter::AssetLinkedSecureDataStore(data_store) => {
                data_store.validate_burn(ctx)
            }
            ExternalPluginAdapter::DataSection(_) => Ok(ValidationResult::Pass),
        }
    }

    /// Route the validation of the transfer action to the appropriate external plugin adapter.
    pub(crate) fn validate_transfer(
        external_plugin_adapter: &ExternalPluginAdapter,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match external_plugin_adapter {
            ExternalPluginAdapter::LifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_transfer(ctx)
            }
            ExternalPluginAdapter::Oracle(oracle) => oracle.validate_transfer(ctx),
            ExternalPluginAdapter::SecureDataStore(data_store) => data_store.validate_transfer(ctx),
            ExternalPluginAdapter::AssetLinkedSecureDataStore(data_store) => {
                data_store.validate_transfer(ctx)
            }
            ExternalPluginAdapter::DataSection(_) => Ok(ValidationResult::Pass),
        }
    }

    /// Validate the add external plugin adapter lifecycle event.
    pub(crate) fn validate_add_external_plugin_adapter(
        external_plugin_adapter: &ExternalPluginAdapter,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match external_plugin_adapter {
            ExternalPluginAdapter::LifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_add_external_plugin_adapter(ctx)
            }
            ExternalPluginAdapter::Oracle(oracle) => {
                oracle.validate_add_external_plugin_adapter(ctx)
            }
            ExternalPluginAdapter::SecureDataStore(data_store) => {
                data_store.validate_add_external_plugin_adapter(ctx)
            }
            ExternalPluginAdapter::AssetLinkedSecureDataStore(data_store) => {
                data_store.validate_add_external_plugin_adapter(ctx)
            }
            ExternalPluginAdapter::DataSection(_) => Ok(ValidationResult::Pass),
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

impl From<&ExternalPluginAdapterInitInfo> for ExternalPluginAdapter {
    fn from(init_info: &ExternalPluginAdapterInitInfo) -> Self {
        match init_info {
            ExternalPluginAdapterInitInfo::LifecycleHook(init_info) => {
                ExternalPluginAdapter::LifecycleHook(LifecycleHook::from(init_info))
            }
            ExternalPluginAdapterInitInfo::Oracle(init_info) => {
                ExternalPluginAdapter::Oracle(Oracle::from(init_info))
            }
            ExternalPluginAdapterInitInfo::SecureDataStore(init_info) => {
                ExternalPluginAdapter::SecureDataStore(SecureDataStore::from(init_info))
            }
            ExternalPluginAdapterInitInfo::AssetLinkedSecureDataStore(init_info) => {
                ExternalPluginAdapter::AssetLinkedSecureDataStore(AssetLinkedSecureDataStore::from(
                    init_info,
                ))
            }
            ExternalPluginAdapterInitInfo::DataSection(init_info) => {
                ExternalPluginAdapter::DataSection(DataSection::from(init_info))
            }
        }
    }
}

#[repr(C)]
#[derive(Eq, PartialEq, Clone, BorshSerialize, BorshDeserialize, Debug, PartialOrd, Ord, Hash)]
/// An enum listing all the lifecyle events available for external plugin adapter hooks.  Note that some
/// lifecycle events such as adding and removing plugins will be checked by default as they are
/// inherently part of the external plugin adapter system.
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

/// Type used to specify extra accounts for external plugin adapters.
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
pub enum ExternalPluginAdapterSchema {
    /// Raw binary data.
    #[default]
    Binary,
    /// JSON.
    Json,
    /// MessagePack serialized data.
    MsgPack,
}

/// Information needed to initialize an external plugin adapter.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ExternalPluginAdapterInitInfo {
    /// Lifecycle Hook.
    LifecycleHook(LifecycleHookInitInfo),
    /// Oracle.
    Oracle(OracleInitInfo),
    /// Secure Store.
    SecureDataStore(SecureDataStoreInitInfo),
    /// Asset-Linked Lifecycle Hook.
    AssetLinkedLifecycleHook(AssetLinkedLifecycleHookInitInfo),
    /// Asset-Linked Secure Store.
    AssetLinkedSecureDataStore(AssetLinkedSecureDataStoreInitInfo),
    /// Data Section.
    DataSection(DataSectionInitInfo),
}

/// Information needed to update an external plugin adapter.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ExternalPluginAdapterUpdateInfo {
    /// Lifecycle Hook.
    LifecycleHook(LifecycleHookUpdateInfo),
    /// Oracle.
    Oracle(OracleUpdateInfo),
    /// Secure Store.
    SecureDataStore(SecureDataStoreUpdateInfo),
    /// Asset-Linked Secure Store.
    AssetLinkedSecureDataStore(AssetLinkedSecureDataStoreUpdateInfo),
}

/// Key used to uniquely specify an external plugin adapter after it is created.
#[repr(C)]
#[derive(
    Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq, EnumCount, PartialOrd, Ord,
)]
pub enum ExternalPluginAdapterKey {
    /// Lifecycle Hook.
    LifecycleHook(Pubkey),
    /// Oracle.
    Oracle(Pubkey),
    /// Secure Store.
    SecureDataStore(Authority),
    /// Asset-Linked Lifecycle Hook.
    AssetLinkedLifecycleHook(Pubkey),
    /// Asset-Linked Secure Store.
    AssetLinkedSecureDataStore(Authority),
    /// Data Section.
    DataSection(LinkedDataKey),
}

/// Key to point to the plugin that manages this data section.
#[repr(C)]
#[derive(
    Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq, EnumCount, PartialOrd, Ord,
)]
pub enum LinkedDataKey {
    /// Lifecycle Hook.
    AssetLinkedLifecycleHook(Pubkey),
    /// Asset-Linked Secure Store.
    AssetLinkedSecureDataStore(Authority),
}

impl ExternalPluginAdapterKey {
    pub(crate) fn from_record(
        account: &AccountInfo,
        external_registry_record: &ExternalRegistryRecord,
    ) -> Result<Self, ProgramError> {
        let pubkey_or_authority_offset = external_registry_record
            .offset
            .checked_add(1)
            .ok_or(MplCoreError::NumericalOverflow)?;

        match external_registry_record.plugin_type {
            ExternalPluginAdapterType::LifecycleHook => {
                let pubkey =
                    Pubkey::deserialize(&mut &account.data.borrow()[pubkey_or_authority_offset..])?;
                Ok(Self::LifecycleHook(pubkey))
            }
            ExternalPluginAdapterType::AssetLinkedLifecycleHook => {
                let pubkey =
                    Pubkey::deserialize(&mut &account.data.borrow()[pubkey_or_authority_offset..])?;
                Ok(Self::AssetLinkedLifecycleHook(pubkey))
            }
            ExternalPluginAdapterType::Oracle => {
                let pubkey =
                    Pubkey::deserialize(&mut &account.data.borrow()[pubkey_or_authority_offset..])?;
                Ok(Self::Oracle(pubkey))
            }
            ExternalPluginAdapterType::SecureDataStore => {
                let authority = Authority::deserialize(
                    &mut &account.data.borrow()[pubkey_or_authority_offset..],
                )?;
                Ok(Self::SecureDataStore(authority))
            }
            ExternalPluginAdapterType::AssetLinkedSecureDataStore => {
                let authority = Authority::deserialize(
                    &mut &account.data.borrow()[pubkey_or_authority_offset..],
                )?;
                Ok(Self::AssetLinkedSecureDataStore(authority))
            }
            ExternalPluginAdapterType::DataSection => {
                let linked_data_key = LinkedDataKey::deserialize(
                    &mut &account.data.borrow()[pubkey_or_authority_offset..],
                )?;
                Ok(Self::DataSection(linked_data_key))
            }
        }
    }
}

impl From<&ExternalPluginAdapterInitInfo> for ExternalPluginAdapterKey {
    fn from(init_info: &ExternalPluginAdapterInitInfo) -> Self {
        match init_info {
            ExternalPluginAdapterInitInfo::LifecycleHook(init_info) => {
                ExternalPluginAdapterKey::LifecycleHook(init_info.hooked_program)
            }
            ExternalPluginAdapterInitInfo::Oracle(init_info) => {
                ExternalPluginAdapterKey::Oracle(init_info.base_address)
            }
            ExternalPluginAdapterInitInfo::SecureDataStore(init_info) => {
                ExternalPluginAdapterKey::SecureDataStore(init_info.data_authority)
            }
            ExternalPluginAdapterInitInfo::AssetLinkedSecureDataStore(init_info) => {
                ExternalPluginAdapterKey::AssetLinkedSecureDataStore(init_info.data_authority)
            }
            ExternalPluginAdapterInitInfo::DataSection(init_info) => {
                ExternalPluginAdapterKey::DataSection(init_info.parent_key)
            }
        }
    }
}
