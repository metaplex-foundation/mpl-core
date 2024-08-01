use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};
use strum::EnumCount;

use crate::{
    error::MplCoreError,
    plugins::lifecycle::{approve, reject},
    state::{AssetV1, SolanaAccount},
};

use super::{
    AppData, AppDataInitInfo, AppDataUpdateInfo, Authority, DataSection, DataSectionInitInfo,
    ExternalCheckResult, ExternalRegistryRecord, LifecycleHook, LifecycleHookInitInfo,
    LifecycleHookUpdateInfo, LinkedAppData, LinkedAppDataInitInfo, LinkedAppDataUpdateInfo,
    LinkedLifecycleHook, LinkedLifecycleHookInitInfo, LinkedLifecycleHookUpdateInfo, Oracle,
    OracleInitInfo, OracleUpdateInfo, PluginValidation, PluginValidationContext, ValidationResult,
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
    /// App Data.
    AppData,
    /// Linked Lifecycle Hook.
    LinkedLifecycleHook,
    /// Linked App Data.
    LinkedAppData,
    /// Data Section.
    DataSection,
}

impl From<&ExternalPluginAdapterKey> for ExternalPluginAdapterType {
    fn from(key: &ExternalPluginAdapterKey) -> Self {
        match key {
            ExternalPluginAdapterKey::LifecycleHook(_) => ExternalPluginAdapterType::LifecycleHook,
            ExternalPluginAdapterKey::LinkedLifecycleHook(_) => {
                ExternalPluginAdapterType::LinkedLifecycleHook
            }
            ExternalPluginAdapterKey::Oracle(_) => ExternalPluginAdapterType::Oracle,
            ExternalPluginAdapterKey::AppData(_) => ExternalPluginAdapterType::AppData,
            ExternalPluginAdapterKey::LinkedAppData(_) => ExternalPluginAdapterType::LinkedAppData,
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
            ExternalPluginAdapterInitInfo::AppData(_) => ExternalPluginAdapterType::AppData,
            ExternalPluginAdapterInitInfo::LinkedLifecycleHook(_) => {
                ExternalPluginAdapterType::LinkedLifecycleHook
            }
            ExternalPluginAdapterInitInfo::LinkedAppData(_) => {
                ExternalPluginAdapterType::LinkedAppData
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
    /// Arbitrary data that can be written to by the data `Authority` stored in the attached
    /// struct.  Note this data authority is different then the plugin authority.
    AppData(AppData),
    /// Collection Only: Linked Lifecycle Hook.  The hooked program and extra accounts are specified in the attached
    /// struct.  The hooked program is called at specified lifecycle events and will return a
    /// validation result and new data to store.
    LinkedLifecycleHook(LinkedLifecycleHook),
    /// Collection only: Arbitrary data that can be written to by the data `Authority` stored on any asset in the Collection in the Data Section struct.
    /// Authority is different then the plugin authority.
    LinkedAppData(LinkedAppData),
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
                ExternalPluginAdapter::AppData(app_data),
                ExternalPluginAdapterUpdateInfo::AppData(update_info),
            ) => {
                app_data.update(update_info);
            }
            (
                ExternalPluginAdapter::LinkedLifecycleHook(linked_lifecycle_hook),
                ExternalPluginAdapterUpdateInfo::LinkedLifecycleHook(update_info),
            ) => {
                linked_lifecycle_hook.update(update_info);
            }
            (
                ExternalPluginAdapter::LinkedAppData(linked_app_data),
                ExternalPluginAdapterUpdateInfo::LinkedAppData(update_info),
            ) => {
                linked_app_data.update(update_info);
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
            ExternalPluginAdapterInitInfo::AppData(_) => ExternalCheckResult::none(),
            ExternalPluginAdapterInitInfo::LinkedLifecycleHook(init_info) => {
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
            ExternalPluginAdapterInitInfo::LinkedAppData(_) => ExternalCheckResult::none(),
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
            ExternalPluginAdapter::AppData(app_data) => app_data.validate_create(ctx),
            ExternalPluginAdapter::LinkedLifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_create(ctx)
            }
            ExternalPluginAdapter::LinkedAppData(app_data) => app_data.validate_create(ctx),
            // Here we block the creation of a DataSection plugin because this is only done internally.
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
            ExternalPluginAdapter::AppData(app_data) => app_data.validate_update(ctx),
            ExternalPluginAdapter::LinkedLifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_update(ctx)
            }
            ExternalPluginAdapter::LinkedAppData(app_data) => app_data.validate_update(ctx),
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
            ExternalPluginAdapter::AppData(app_data) => app_data.validate_burn(ctx),
            ExternalPluginAdapter::LinkedLifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_burn(ctx)
            }
            ExternalPluginAdapter::LinkedAppData(app_data) => app_data.validate_burn(ctx),
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
            ExternalPluginAdapter::AppData(app_data) => app_data.validate_transfer(ctx),
            ExternalPluginAdapter::LinkedLifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_transfer(ctx)
            }
            ExternalPluginAdapter::LinkedAppData(app_data) => app_data.validate_transfer(ctx),
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
            ExternalPluginAdapter::AppData(app_data) => {
                app_data.validate_add_external_plugin_adapter(ctx)
            }
            ExternalPluginAdapter::LinkedLifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_add_external_plugin_adapter(ctx)
            }
            ExternalPluginAdapter::LinkedAppData(app_data) => {
                app_data.validate_add_external_plugin_adapter(ctx)
            }
            // Here we block the creation of a DataSection plugin because this is only done internally.
            ExternalPluginAdapter::DataSection(_) => Ok(ValidationResult::Rejected),
        }
    }

    /// Validate the add external plugin adapter lifecycle event.
    pub(crate) fn validate_update_external_plugin_adapter(
        external_plugin_adapter: &ExternalPluginAdapter,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let resolved_authorities = ctx
            .resolved_authorities
            .ok_or(MplCoreError::InvalidAuthority)?;
        let base_result = if resolved_authorities.contains(ctx.self_authority) {
            solana_program::msg!("Base: Approved");
            ValidationResult::Approved
        } else {
            ValidationResult::Pass
        };

        let result = match external_plugin_adapter {
            ExternalPluginAdapter::LifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_update_external_plugin_adapter(ctx)
            }
            ExternalPluginAdapter::Oracle(oracle) => {
                oracle.validate_update_external_plugin_adapter(ctx)
            }
            ExternalPluginAdapter::AppData(app_data) => {
                app_data.validate_update_external_plugin_adapter(ctx)
            }
            ExternalPluginAdapter::LinkedLifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_update_external_plugin_adapter(ctx)
            }
            ExternalPluginAdapter::LinkedAppData(app_data) => {
                app_data.validate_update_external_plugin_adapter(ctx)
            }
            // Here we block the update of a DataSection plugin because this is only done internally.
            ExternalPluginAdapter::DataSection(_) => Ok(ValidationResult::Rejected),
        }?;

        match (&base_result, &result) {
            (ValidationResult::Approved, ValidationResult::Approved) => {
                approve!()
            }
            (ValidationResult::Approved, ValidationResult::Rejected) => {
                reject!()
            }
            (ValidationResult::Rejected, ValidationResult::Approved) => {
                reject!()
            }
            (ValidationResult::Rejected, ValidationResult::Rejected) => {
                reject!()
            }
            (ValidationResult::Pass, _) => Ok(result),
            (ValidationResult::ForceApproved, _) => unreachable!(),
            (_, ValidationResult::Pass) => Ok(base_result),
            (_, ValidationResult::ForceApproved) => unreachable!(),
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
            ExternalPluginAdapterInitInfo::AppData(init_info) => {
                ExternalPluginAdapter::AppData(AppData::from(init_info))
            }
            ExternalPluginAdapterInitInfo::LinkedLifecycleHook(init_info) => {
                ExternalPluginAdapter::LinkedLifecycleHook(LinkedLifecycleHook::from(init_info))
            }
            ExternalPluginAdapterInitInfo::LinkedAppData(init_info) => {
                ExternalPluginAdapter::LinkedAppData(LinkedAppData::from(init_info))
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
    /// App Data.
    AppData(AppDataInitInfo),
    /// Linked Lifecycle Hook.
    LinkedLifecycleHook(LinkedLifecycleHookInitInfo),
    /// Linked App Data.
    LinkedAppData(LinkedAppDataInitInfo),
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
    /// App Data.
    AppData(AppDataUpdateInfo),
    /// Linked Lifecycle Hook.
    LinkedLifecycleHook(LinkedLifecycleHookUpdateInfo),
    /// Linked App Data.
    LinkedAppData(LinkedAppDataUpdateInfo),
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
    /// App Data.
    AppData(Authority),
    /// Linked Lifecycle Hook.
    LinkedLifecycleHook(Pubkey),
    /// Linked App Data.
    LinkedAppData(Authority),
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
    LinkedLifecycleHook(Pubkey),
    /// Linked App Data.
    LinkedAppData(Authority),
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
            ExternalPluginAdapterType::LinkedLifecycleHook => {
                let pubkey =
                    Pubkey::deserialize(&mut &account.data.borrow()[pubkey_or_authority_offset..])?;
                Ok(Self::LinkedLifecycleHook(pubkey))
            }
            ExternalPluginAdapterType::Oracle => {
                let pubkey =
                    Pubkey::deserialize(&mut &account.data.borrow()[pubkey_or_authority_offset..])?;
                Ok(Self::Oracle(pubkey))
            }
            ExternalPluginAdapterType::AppData => {
                let authority = Authority::deserialize(
                    &mut &account.data.borrow()[pubkey_or_authority_offset..],
                )?;
                Ok(Self::AppData(authority))
            }
            ExternalPluginAdapterType::LinkedAppData => {
                let authority = Authority::deserialize(
                    &mut &account.data.borrow()[pubkey_or_authority_offset..],
                )?;
                Ok(Self::LinkedAppData(authority))
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
            ExternalPluginAdapterInitInfo::AppData(init_info) => {
                ExternalPluginAdapterKey::AppData(init_info.data_authority)
            }
            ExternalPluginAdapterInitInfo::LinkedLifecycleHook(init_info) => {
                ExternalPluginAdapterKey::LinkedLifecycleHook(init_info.hooked_program)
            }
            ExternalPluginAdapterInitInfo::LinkedAppData(init_info) => {
                ExternalPluginAdapterKey::LinkedAppData(init_info.data_authority)
            }
            ExternalPluginAdapterInitInfo::DataSection(init_info) => {
                ExternalPluginAdapterKey::DataSection(init_info.parent_key)
            }
        }
    }
}
