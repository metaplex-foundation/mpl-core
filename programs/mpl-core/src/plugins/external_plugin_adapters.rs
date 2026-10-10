use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};
use strum::{EnumCount, EnumIter};

use crate::{
    error::MplCoreError,
    plugins::{approve, reject},
    state::{AssetV1, DataBlob, SolanaAccount},
};

use super::{
    AgentIdentity, AgentIdentityInitInfo, AgentIdentityUpdateInfo, AppData, AppDataInitInfo,
    AppDataUpdateInfo, Authority, DataSection, DataSectionInitInfo, ExternalCheckResult,
    ExternalRegistryRecord, LifecycleHook, LifecycleHookInitInfo, LifecycleHookUpdateInfo,
    LinkedAppData, LinkedAppDataInitInfo, LinkedAppDataUpdateInfo, LinkedLifecycleHook,
    LinkedLifecycleHookInitInfo, LinkedLifecycleHookUpdateInfo, Oracle, OracleInitInfo,
    OracleUpdateInfo, PluginValidation, PluginValidationContext, ValidationResult,
};

/// List of third party plugin types.
#[repr(C)]
#[derive(
    Clone,
    Copy,
    Debug,
    BorshSerialize,
    BorshDeserialize,
    Eq,
    PartialEq,
    EnumCount,
    PartialOrd,
    Ord,
    EnumIter,
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
    /// Agent Identity.
    AgentIdentity,
}

impl ExternalPluginAdapterType {
    /// A u8 enum.
    const BASE_LEN: usize = 1;
}

impl DataBlob for ExternalPluginAdapterType {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
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
            ExternalPluginAdapterKey::AgentIdentity => ExternalPluginAdapterType::AgentIdentity,
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
            ExternalPluginAdapterInitInfo::AgentIdentity(_) => {
                ExternalPluginAdapterType::AgentIdentity
            }
        }
    }
}

impl From<&ExternalPluginAdapter> for ExternalPluginAdapterType {
    fn from(external_plugin_adapter: &ExternalPluginAdapter) -> Self {
        match external_plugin_adapter {
            ExternalPluginAdapter::LifecycleHook(_) => ExternalPluginAdapterType::LifecycleHook,
            ExternalPluginAdapter::Oracle(_) => ExternalPluginAdapterType::Oracle,
            ExternalPluginAdapter::AppData(_) => ExternalPluginAdapterType::AppData,
            ExternalPluginAdapter::LinkedLifecycleHook(_) => {
                ExternalPluginAdapterType::LinkedLifecycleHook
            }
            ExternalPluginAdapter::LinkedAppData(_) => ExternalPluginAdapterType::LinkedAppData,
            ExternalPluginAdapter::DataSection(_) => ExternalPluginAdapterType::DataSection,
            ExternalPluginAdapter::AgentIdentity(_) => ExternalPluginAdapterType::AgentIdentity,
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
    /// Asset only: Agent Identity plugin that links to an ERC-8004 spec registration file via a URI.
    AgentIdentity(AgentIdentity),
}

impl ExternalPluginAdapter {
    /// Update the plugin from the update info.
    pub fn update(&mut self, update_info: &ExternalPluginAdapterUpdateInfo) -> ProgramResult {
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
            (
                ExternalPluginAdapter::AgentIdentity(agent_identity),
                ExternalPluginAdapterUpdateInfo::AgentIdentity(update_info),
            ) => {
                agent_identity.update(update_info);
            }
            _ => return Err(MplCoreError::InvalidPlugin.into()),
        }

        Ok(())
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
            ExternalPluginAdapterInitInfo::AgentIdentity(init_info) => {
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
            ExternalPluginAdapter::AgentIdentity(agent_identity) => {
                agent_identity.validate_create(ctx)
            }
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
            ExternalPluginAdapter::AgentIdentity(agent_identity) => {
                agent_identity.validate_update(ctx)
            }
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
            ExternalPluginAdapter::AgentIdentity(agent_identity) => {
                agent_identity.validate_burn(ctx)
            }
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
            ExternalPluginAdapter::AgentIdentity(agent_identity) => {
                agent_identity.validate_transfer(ctx)
            }
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
            ExternalPluginAdapter::AgentIdentity(agent_identity) => {
                agent_identity.validate_add_external_plugin_adapter(ctx)
            }
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
        // If the authority is right for the asset performing the validation.
        let base_result = if resolved_authorities.contains(ctx.self_authority)
        // And the target plugin is also the one being updated (i.e. self).
        && ctx.target_external_plugin.is_some()
        && ExternalPluginAdapterType::from(ctx.target_external_plugin.unwrap()) == ExternalPluginAdapterType::from(external_plugin_adapter)
        {
            solana_program::msg!("{}:{}:Base:Approved", std::file!(), std::line!());
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
            ExternalPluginAdapter::AgentIdentity(agent_identity) => {
                agent_identity.validate_update_external_plugin_adapter(ctx)
            }
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

    /// Check if a plugin is permitted to approve or deny an execute action.
    pub fn check_execute(plugin: &ExternalPluginAdapterInitInfo) -> ExternalCheckResult {
        match plugin {
            ExternalPluginAdapterInitInfo::LifecycleHook(init_info) => {
                if let Some(checks) = init_info
                    .lifecycle_checks
                    .iter()
                    .find(|event| event.0 == HookableLifecycleEvent::Execute)
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
                    .find(|event| event.0 == HookableLifecycleEvent::Execute)
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
                    .find(|event| event.0 == HookableLifecycleEvent::Execute)
                {
                    checks.1
                } else {
                    ExternalCheckResult::none()
                }
            }
            ExternalPluginAdapterInitInfo::LinkedAppData(_) => ExternalCheckResult::none(),
            ExternalPluginAdapterInitInfo::DataSection(_) => ExternalCheckResult::none(),
            ExternalPluginAdapterInitInfo::AgentIdentity(init_info) => {
                if let Some(checks) = init_info
                    .lifecycle_checks
                    .iter()
                    .find(|event| event.0 == HookableLifecycleEvent::Execute)
                {
                    checks.1
                } else {
                    ExternalCheckResult::none()
                }
            }
        }
    }

    /// Route the validation of the execute action to the appropriate external plugin adapter.
    pub(crate) fn validate_execute(
        external_plugin_adapter: &ExternalPluginAdapter,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match external_plugin_adapter {
            ExternalPluginAdapter::LifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_execute(ctx)
            }
            ExternalPluginAdapter::Oracle(oracle) => oracle.validate_execute(ctx),
            ExternalPluginAdapter::AppData(app_data) => app_data.validate_execute(ctx),
            ExternalPluginAdapter::LinkedLifecycleHook(lifecycle_hook) => {
                lifecycle_hook.validate_execute(ctx)
            }
            ExternalPluginAdapter::LinkedAppData(app_data) => app_data.validate_execute(ctx),
            ExternalPluginAdapter::DataSection(_) => Ok(ValidationResult::Pass),
            ExternalPluginAdapter::AgentIdentity(agent_identity) => {
                agent_identity.validate_execute(ctx)
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
            ExternalPluginAdapterInitInfo::AgentIdentity(init_info) => {
                ExternalPluginAdapter::AgentIdentity(AgentIdentity::from(init_info))
            }
        }
    }
}

#[repr(C)]
#[derive(
    Eq, PartialEq, Clone, BorshSerialize, BorshDeserialize, Debug, PartialOrd, Ord, Hash, EnumIter,
)]
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
    /// Execute an instruction on behalf of the Asset.
    Execute,
}

impl HookableLifecycleEvent {
    /// A u8 enum.
    const BASE_LEN: usize = 1;
}

impl DataBlob for HookableLifecycleEvent {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
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
    /// Agent Identity.
    AgentIdentity(AgentIdentityInitInfo),
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
    /// Agent Identity.
    AgentIdentity(AgentIdentityUpdateInfo),
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
    /// Agent Identity.  Only one per asset so no discriminator needed.
    AgentIdentity,
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
            ExternalPluginAdapterType::AgentIdentity => Ok(Self::AgentIdentity),
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
            ExternalPluginAdapterInitInfo::AgentIdentity(_) => {
                ExternalPluginAdapterKey::AgentIdentity
            }
        }
    }
}

/// Test DataBlob sizing
#[cfg(test)]
mod test {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_external_plugin_adapter_type_size() {
        for fixture in ExternalPluginAdapterType::iter() {
            let serialized = borsh::to_vec(&fixture).unwrap();
            assert_eq!(
                serialized.len(),
                fixture.len(),
                "Serialized {:?} should match size returned by len()",
                fixture
            );
        }
    }

    #[test]
    fn test_hookable_lifecycle_event_size() {
        for fixture in HookableLifecycleEvent::iter() {
            let serialized = borsh::to_vec(&fixture).unwrap();
            assert_eq!(
                serialized.len(),
                fixture.len(),
                "Serialized {:?} should match size returned by len()",
                fixture
            );
        }
    }

    #[test]
    fn test_external_plugin_adapter_update_rejects_mismatched_variant() {
        let mut plugin = ExternalPluginAdapter::AppData(AppData {
            data_authority: Authority::UpdateAuthority,
            schema: ExternalPluginAdapterSchema::Binary,
        });
        let update_info = ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
            lifecycle_checks: None,
            base_address_config: None,
            results_offset: None,
        });

        let error = plugin.update(&update_info).unwrap_err();

        assert_eq!(error, MplCoreError::InvalidPlugin.into());
    }

    #[test]
    fn test_external_plugin_adapter_update_applies_matching_variant() {
        let mut plugin = ExternalPluginAdapter::AppData(AppData {
            data_authority: Authority::UpdateAuthority,
            schema: ExternalPluginAdapterSchema::Binary,
        });
        let update_info = ExternalPluginAdapterUpdateInfo::AppData(AppDataUpdateInfo {
            schema: Some(ExternalPluginAdapterSchema::Json),
        });

        plugin.update(&update_info).unwrap();

        assert_eq!(
            plugin,
            ExternalPluginAdapter::AppData(AppData {
                data_authority: Authority::UpdateAuthority,
                schema: ExternalPluginAdapterSchema::Json,
            })
        );
    }
}

#[cfg(test)]
mod validation_tests {
    use {
        super::*,
        crate::{
            plugins::{
                test_ctx::{default_ctx, FakeAccount},
                AgentIdentity, AgentIdentityInitInfo, AgentIdentityUpdateInfo, AppData,
                AppDataInitInfo, AppDataUpdateInfo, DataSection, DataSectionInitInfo,
                LifecycleHook, LifecycleHookInitInfo, LifecycleHookUpdateInfo, LinkedAppData,
                LinkedAppDataInitInfo, LinkedAppDataUpdateInfo, LinkedLifecycleHook,
                LinkedLifecycleHookInitInfo, Oracle, OracleInitInfo, OracleUpdateInfo,
                ValidationResultsOffset,
            },
            state::{AssetV1, Key, UpdateAuthority},
        },
    };

    const HOOK: Pubkey = Pubkey::new_from_array([7u8; 32]);
    const ORACLE: Pubkey = Pubkey::new_from_array([8u8; 32]);

    fn data_authority() -> Authority {
        Authority::Address {
            address: Pubkey::new_from_array([9u8; 32]),
        }
    }

    fn create_check(flags: u32) -> Vec<(HookableLifecycleEvent, ExternalCheckResult)> {
        vec![(
            HookableLifecycleEvent::Create,
            ExternalCheckResult { flags },
        )]
    }

    /// One init info per variant, paired with the adapter and key it should
    /// convert into.
    #[allow(clippy::type_complexity)]
    fn conversion_cases() -> Vec<(
        ExternalPluginAdapterInitInfo,
        ExternalPluginAdapterType,
        ExternalPluginAdapter,
        ExternalPluginAdapterKey,
    )> {
        vec![
            (
                ExternalPluginAdapterInitInfo::LifecycleHook(LifecycleHookInitInfo {
                    hooked_program: HOOK,
                    init_plugin_authority: None,
                    lifecycle_checks: create_check(0x4),
                    extra_accounts: None,
                    data_authority: Some(data_authority()),
                    schema: None,
                }),
                ExternalPluginAdapterType::LifecycleHook,
                ExternalPluginAdapter::LifecycleHook(LifecycleHook {
                    hooked_program: HOOK,
                    extra_accounts: None,
                    data_authority: Some(data_authority()),
                    schema: ExternalPluginAdapterSchema::Binary,
                }),
                ExternalPluginAdapterKey::LifecycleHook(HOOK),
            ),
            (
                ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
                    base_address: ORACLE,
                    init_plugin_authority: None,
                    lifecycle_checks: create_check(0x4),
                    base_address_config: None,
                    results_offset: None,
                }),
                ExternalPluginAdapterType::Oracle,
                ExternalPluginAdapter::Oracle(Oracle {
                    base_address: ORACLE,
                    base_address_config: None,
                    results_offset: ValidationResultsOffset::NoOffset,
                }),
                ExternalPluginAdapterKey::Oracle(ORACLE),
            ),
            (
                ExternalPluginAdapterInitInfo::AppData(AppDataInitInfo {
                    data_authority: data_authority(),
                    init_plugin_authority: None,
                    schema: None,
                }),
                ExternalPluginAdapterType::AppData,
                ExternalPluginAdapter::AppData(AppData {
                    data_authority: data_authority(),
                    schema: ExternalPluginAdapterSchema::Binary,
                }),
                ExternalPluginAdapterKey::AppData(data_authority()),
            ),
            (
                ExternalPluginAdapterInitInfo::LinkedLifecycleHook(LinkedLifecycleHookInitInfo {
                    hooked_program: HOOK,
                    init_plugin_authority: None,
                    lifecycle_checks: create_check(0x4),
                    extra_accounts: None,
                    data_authority: Some(data_authority()),
                    schema: None,
                }),
                ExternalPluginAdapterType::LinkedLifecycleHook,
                ExternalPluginAdapter::LinkedLifecycleHook(LinkedLifecycleHook {
                    hooked_program: HOOK,
                    extra_accounts: None,
                    data_authority: Some(data_authority()),
                    schema: ExternalPluginAdapterSchema::Binary,
                }),
                ExternalPluginAdapterKey::LinkedLifecycleHook(HOOK),
            ),
            (
                ExternalPluginAdapterInitInfo::LinkedAppData(LinkedAppDataInitInfo {
                    data_authority: data_authority(),
                    init_plugin_authority: None,
                    schema: None,
                }),
                ExternalPluginAdapterType::LinkedAppData,
                ExternalPluginAdapter::LinkedAppData(LinkedAppData {
                    data_authority: data_authority(),
                    schema: ExternalPluginAdapterSchema::Binary,
                }),
                ExternalPluginAdapterKey::LinkedAppData(data_authority()),
            ),
            (
                ExternalPluginAdapterInitInfo::DataSection(DataSectionInitInfo {
                    parent_key: LinkedDataKey::LinkedAppData(data_authority()),
                    schema: ExternalPluginAdapterSchema::Json,
                }),
                ExternalPluginAdapterType::DataSection,
                ExternalPluginAdapter::DataSection(DataSection {
                    parent_key: LinkedDataKey::LinkedAppData(data_authority()),
                    schema: ExternalPluginAdapterSchema::Json,
                }),
                ExternalPluginAdapterKey::DataSection(LinkedDataKey::LinkedAppData(
                    data_authority(),
                )),
            ),
            (
                ExternalPluginAdapterInitInfo::AgentIdentity(AgentIdentityInitInfo {
                    uri: "https://example.com/agent.json".to_string(),
                    init_plugin_authority: None,
                    lifecycle_checks: create_check(0x4),
                }),
                ExternalPluginAdapterType::AgentIdentity,
                ExternalPluginAdapter::AgentIdentity(AgentIdentity {
                    uri: "https://example.com/agent.json".to_string(),
                }),
                ExternalPluginAdapterKey::AgentIdentity,
            ),
        ]
    }

    #[test]
    fn init_info_conversions_cover_every_variant() {
        for (init_info, expected_type, expected_adapter, expected_key) in conversion_cases() {
            assert_eq!(
                ExternalPluginAdapterType::from(&init_info),
                expected_type,
                "{init_info:?} type"
            );
            assert_eq!(
                ExternalPluginAdapter::from(&init_info),
                expected_adapter,
                "{init_info:?} adapter"
            );
            assert_eq!(
                ExternalPluginAdapterKey::from(&init_info),
                expected_key,
                "{init_info:?} key"
            );
            // The adapter and its key agree on the type.
            assert_eq!(
                ExternalPluginAdapterType::from(&expected_adapter),
                expected_type
            );
            assert_eq!(
                ExternalPluginAdapterType::from(&expected_key),
                expected_type
            );
        }
    }

    #[test]
    fn check_create_and_check_execute_read_the_declared_checks() {
        // Hookable adapters return the declared flags for the event, and
        // `none()` when the event is absent; the data-only adapters always
        // return `none()`.
        for (init_info, expected_create) in [
            (
                ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
                    base_address: ORACLE,
                    init_plugin_authority: None,
                    lifecycle_checks: create_check(0x4),
                    base_address_config: None,
                    results_offset: None,
                }),
                ExternalCheckResult { flags: 0x4 },
            ),
            (
                ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
                    base_address: ORACLE,
                    init_plugin_authority: None,
                    lifecycle_checks: vec![(
                        HookableLifecycleEvent::Burn,
                        ExternalCheckResult { flags: 0x4 },
                    )],
                    base_address_config: None,
                    results_offset: None,
                }),
                ExternalCheckResult::none(),
            ),
            (
                ExternalPluginAdapterInitInfo::AppData(AppDataInitInfo {
                    data_authority: data_authority(),
                    init_plugin_authority: None,
                    schema: None,
                }),
                ExternalCheckResult::none(),
            ),
            (
                ExternalPluginAdapterInitInfo::LinkedAppData(LinkedAppDataInitInfo {
                    data_authority: data_authority(),
                    init_plugin_authority: None,
                    schema: None,
                }),
                ExternalCheckResult::none(),
            ),
            (
                ExternalPluginAdapterInitInfo::DataSection(DataSectionInitInfo {
                    parent_key: LinkedDataKey::LinkedAppData(data_authority()),
                    schema: ExternalPluginAdapterSchema::Binary,
                }),
                ExternalCheckResult::none(),
            ),
        ] {
            assert_eq!(
                ExternalPluginAdapter::check_create(&init_info),
                expected_create,
                "check_create({init_info:?})"
            );
        }

        // `check_execute` has no callers in the program (roadmap section 11,
        // finding 7); it reads the `Execute` entry the same way.
        let execute_hook = ExternalPluginAdapterInitInfo::LifecycleHook(LifecycleHookInitInfo {
            hooked_program: HOOK,
            init_plugin_authority: None,
            lifecycle_checks: vec![(
                HookableLifecycleEvent::Execute,
                ExternalCheckResult { flags: 0x1 },
            )],
            extra_accounts: None,
            data_authority: None,
            schema: None,
        });
        assert_eq!(
            ExternalPluginAdapter::check_execute(&execute_hook),
            ExternalCheckResult { flags: 0x1 }
        );
        assert_eq!(
            ExternalPluginAdapter::check_execute(&ExternalPluginAdapterInitInfo::AppData(
                AppDataInitInfo {
                    data_authority: data_authority(),
                    init_plugin_authority: None,
                    schema: None,
                }
            )),
            ExternalCheckResult::none()
        );
    }

    #[test]
    fn update_requires_matching_adapter_and_update_info_variants() {
        let mut oracle = ExternalPluginAdapter::Oracle(Oracle {
            base_address: ORACLE,
            base_address_config: None,
            results_offset: ValidationResultsOffset::NoOffset,
        });
        oracle
            .update(&ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
                lifecycle_checks: None,
                base_address_config: Some(ExtraAccount::PreconfiguredAsset {
                    is_signer: false,
                    is_writable: false,
                }),
                results_offset: Some(ValidationResultsOffset::Anchor),
            }))
            .unwrap();
        assert_eq!(
            oracle,
            ExternalPluginAdapter::Oracle(Oracle {
                base_address: ORACLE,
                base_address_config: Some(ExtraAccount::PreconfiguredAsset {
                    is_signer: false,
                    is_writable: false
                }),
                results_offset: ValidationResultsOffset::Anchor,
            })
        );

        let mut app_data = ExternalPluginAdapter::AppData(AppData {
            data_authority: data_authority(),
            schema: ExternalPluginAdapterSchema::Binary,
        });
        app_data
            .update(&ExternalPluginAdapterUpdateInfo::AppData(
                AppDataUpdateInfo {
                    schema: Some(ExternalPluginAdapterSchema::Json),
                },
            ))
            .unwrap();
        assert_eq!(
            app_data,
            ExternalPluginAdapter::AppData(AppData {
                data_authority: data_authority(),
                schema: ExternalPluginAdapterSchema::Json,
            })
        );

        let mut linked = ExternalPluginAdapter::LinkedAppData(LinkedAppData {
            data_authority: data_authority(),
            schema: ExternalPluginAdapterSchema::Binary,
        });
        linked
            .update(&ExternalPluginAdapterUpdateInfo::LinkedAppData(
                LinkedAppDataUpdateInfo {
                    schema: Some(ExternalPluginAdapterSchema::MsgPack),
                },
            ))
            .unwrap();
        assert_eq!(
            linked,
            ExternalPluginAdapter::LinkedAppData(LinkedAppData {
                data_authority: data_authority(),
                schema: ExternalPluginAdapterSchema::MsgPack,
            })
        );

        // The hook arms, reachable only from crafted state on-chain.
        let mut hook = ExternalPluginAdapter::LifecycleHook(LifecycleHook {
            hooked_program: HOOK,
            extra_accounts: None,
            data_authority: None,
            schema: ExternalPluginAdapterSchema::Binary,
        });
        hook.update(&ExternalPluginAdapterUpdateInfo::LifecycleHook(
            LifecycleHookUpdateInfo {
                lifecycle_checks: None,
                extra_accounts: Some(vec![]),
                schema: Some(ExternalPluginAdapterSchema::Json),
            },
        ))
        .unwrap();
        assert_eq!(
            hook,
            ExternalPluginAdapter::LifecycleHook(LifecycleHook {
                hooked_program: HOOK,
                extra_accounts: Some(vec![]),
                data_authority: None,
                schema: ExternalPluginAdapterSchema::Json,
            })
        );

        let mut linked_hook = ExternalPluginAdapter::LinkedLifecycleHook(LinkedLifecycleHook {
            hooked_program: HOOK,
            extra_accounts: None,
            data_authority: None,
            schema: ExternalPluginAdapterSchema::Binary,
        });
        linked_hook
            .update(&ExternalPluginAdapterUpdateInfo::LinkedLifecycleHook(
                LinkedLifecycleHookUpdateInfo {
                    lifecycle_checks: None,
                    extra_accounts: Some(vec![]),
                    schema: Some(ExternalPluginAdapterSchema::Json),
                },
            ))
            .unwrap();
        assert_eq!(
            linked_hook,
            ExternalPluginAdapter::LinkedLifecycleHook(LinkedLifecycleHook {
                hooked_program: HOOK,
                extra_accounts: Some(vec![]),
                data_authority: None,
                schema: ExternalPluginAdapterSchema::Json,
            })
        );

        let mut agent = ExternalPluginAdapter::AgentIdentity(AgentIdentity {
            uri: "a".to_string(),
        });
        agent
            .update(&ExternalPluginAdapterUpdateInfo::AgentIdentity(
                AgentIdentityUpdateInfo {
                    uri: Some("b".to_string()),
                    lifecycle_checks: None,
                },
            ))
            .unwrap();
        assert_eq!(
            agent,
            ExternalPluginAdapter::AgentIdentity(AgentIdentity {
                uri: "b".to_string()
            })
        );

        // A mismatched pair (and every `DataSection` update, since there is no
        // `DataSection` update-info variant) is `InvalidPlugin`.
        let mut section = ExternalPluginAdapter::DataSection(DataSection {
            parent_key: LinkedDataKey::LinkedAppData(data_authority()),
            schema: ExternalPluginAdapterSchema::Binary,
        });
        let err = section
            .update(&ExternalPluginAdapterUpdateInfo::AppData(
                AppDataUpdateInfo { schema: None },
            ))
            .unwrap_err();
        assert_eq!(err, MplCoreError::InvalidPlugin.into());
    }

    // -----------------------------------------------------------------------
    // ExtraAccount::derive and transform_seeds
    // -----------------------------------------------------------------------

    fn asset_bytes(owner: Pubkey) -> Vec<u8> {
        borsh::to_vec(&AssetV1 {
            key: Key::AssetV1,
            owner,
            update_authority: UpdateAuthority::Address(owner),
            name: "Test Asset".to_string(),
            uri: "https://example.com/test".to_string(),
            seq: None,
        })
        .unwrap()
    }

    fn pda(base: &Pubkey, extra: Option<&Pubkey>) -> Pubkey {
        match extra {
            None => Pubkey::find_program_address(&[MPL_CORE_PREFIX.as_bytes()], base).0,
            Some(seed) => {
                Pubkey::find_program_address(&[MPL_CORE_PREFIX.as_bytes(), seed.as_ref()], base).0
            }
        }
    }

    fn core_err<T>(result: Result<T, ProgramError>) -> MplCoreError {
        match result {
            Err(ProgramError::Custom(code)) => {
                num_traits::FromPrimitive::from_u32(code).expect("an MplCoreError code")
            }
            Err(other) => panic!("expected a custom program error, got {other:?}"),
            Ok(_) => panic!("expected an error"),
        }
    }

    #[test]
    fn extra_account_derive_covers_every_variant() {
        let base = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let literal = Pubkey::new_unique();
        let self_authority = Authority::UpdateAuthority;

        let mut signer = FakeAccount::wallet();
        let mut collection = FakeAccount::wallet();
        let mut new_owner = FakeAccount::wallet();
        let mut asset = FakeAccount::with_data(asset_bytes(owner));
        let signer_info = signer.info();
        let collection_info = collection.info();
        let new_owner_info = new_owner.info();
        let asset_info = asset.info();

        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.asset_info = Some(&asset_info);
        ctx.collection_info = Some(&collection_info);
        ctx.new_owner = Some(&new_owner_info);

        let flags = (false, false);
        let cases: Vec<(ExtraAccount, Pubkey)> = vec![
            (
                ExtraAccount::PreconfiguredProgram {
                    is_signer: flags.0,
                    is_writable: flags.1,
                },
                pda(&base, None),
            ),
            (
                ExtraAccount::PreconfiguredCollection {
                    is_signer: flags.0,
                    is_writable: flags.1,
                },
                pda(&base, Some(collection_info.key)),
            ),
            (
                ExtraAccount::PreconfiguredOwner {
                    is_signer: flags.0,
                    is_writable: flags.1,
                },
                pda(&base, Some(&owner)),
            ),
            (
                ExtraAccount::PreconfiguredRecipient {
                    is_signer: flags.0,
                    is_writable: flags.1,
                },
                pda(&base, Some(new_owner_info.key)),
            ),
            (
                ExtraAccount::PreconfiguredAsset {
                    is_signer: flags.0,
                    is_writable: flags.1,
                },
                pda(&base, Some(asset_info.key)),
            ),
            (
                ExtraAccount::Address {
                    address: literal,
                    is_signer: flags.0,
                    is_writable: flags.1,
                },
                literal,
            ),
        ];

        for (config, expected) in cases {
            assert_eq!(
                config.derive(&base, &ctx).unwrap(),
                expected,
                "{config:?} derivation"
            );
        }

        // Every `Seed` variant, with and without a custom program id.
        let seeds = vec![
            Seed::Bytes(b"prefix".to_vec()),
            Seed::Collection,
            Seed::Owner,
            Seed::Recipient,
            Seed::Asset,
            Seed::Address(literal),
        ];
        let seed_bytes: Vec<Vec<u8>> = vec![
            b"prefix".to_vec(),
            collection_info.key.as_ref().to_vec(),
            owner.as_ref().to_vec(),
            new_owner_info.key.as_ref().to_vec(),
            asset_info.key.as_ref().to_vec(),
            literal.as_ref().to_vec(),
        ];
        let slices: Vec<&[u8]> = seed_bytes.iter().map(Vec::as_slice).collect();

        let custom_program = Pubkey::new_unique();
        for custom_program_id in [None, Some(custom_program)] {
            let config = ExtraAccount::CustomPda {
                seeds: seeds.clone(),
                custom_program_id,
                is_signer: false,
                is_writable: false,
            };
            let expected =
                Pubkey::find_program_address(&slices, custom_program_id.as_ref().unwrap_or(&base))
                    .0;
            assert_eq!(config.derive(&base, &ctx).unwrap(), expected);
        }
    }

    #[test]
    fn extra_account_derive_reports_the_missing_context() {
        let base = Pubkey::new_unique();
        let self_authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let ctx = default_ctx(&[], &signer_info, &self_authority);

        assert_eq!(
            core_err(
                ExtraAccount::PreconfiguredCollection {
                    is_signer: false,
                    is_writable: false,
                }
                .derive(&base, &ctx)
            ),
            MplCoreError::MissingCollection
        );
        assert_eq!(
            core_err(
                ExtraAccount::PreconfiguredOwner {
                    is_signer: false,
                    is_writable: false,
                }
                .derive(&base, &ctx)
            ),
            MplCoreError::MissingAsset
        );
        assert_eq!(
            core_err(
                ExtraAccount::PreconfiguredAsset {
                    is_signer: false,
                    is_writable: false,
                }
                .derive(&base, &ctx)
            ),
            MplCoreError::MissingAsset
        );
        assert_eq!(
            core_err(
                ExtraAccount::PreconfiguredRecipient {
                    is_signer: false,
                    is_writable: false,
                }
                .derive(&base, &ctx)
            ),
            MplCoreError::MissingNewOwner
        );

        for (seed, expected) in [
            (Seed::Collection, MplCoreError::MissingCollection),
            (Seed::Owner, MplCoreError::MissingAsset),
            (Seed::Recipient, MplCoreError::MissingNewOwner),
            (Seed::Asset, MplCoreError::MissingAsset),
        ] {
            assert_eq!(
                core_err(
                    ExtraAccount::CustomPda {
                        seeds: vec![seed.clone()],
                        custom_program_id: None,
                        is_signer: false,
                        is_writable: false,
                    }
                    .derive(&base, &ctx)
                ),
                expected,
                "{seed:?}"
            );
        }
    }
}

#[cfg(test)]
mod router_tests {
    use {
        super::*,
        crate::plugins::{
            test_ctx::{default_ctx, FakeAccount},
            AgentIdentity, AppData, DataSection, LifecycleHook, LifecycleHookInitInfo,
            LinkedAppData, LinkedLifecycleHook, LinkedLifecycleHookInitInfo, Oracle,
            ValidationResultsOffset,
        },
    };

    fn data_authority() -> Authority {
        Authority::Address {
            address: Pubkey::new_from_array([3u8; 32]),
        }
    }

    /// Every adapter variant, in registry order.
    fn every_adapter() -> Vec<ExternalPluginAdapter> {
        let hooked_program = Pubkey::new_from_array([4u8; 32]);
        vec![
            ExternalPluginAdapter::LifecycleHook(LifecycleHook {
                hooked_program,
                extra_accounts: None,
                data_authority: None,
                schema: ExternalPluginAdapterSchema::Binary,
            }),
            ExternalPluginAdapter::Oracle(Oracle {
                base_address: Pubkey::new_from_array([5u8; 32]),
                base_address_config: None,
                results_offset: ValidationResultsOffset::NoOffset,
            }),
            ExternalPluginAdapter::AppData(AppData {
                data_authority: data_authority(),
                schema: ExternalPluginAdapterSchema::Binary,
            }),
            ExternalPluginAdapter::LinkedLifecycleHook(LinkedLifecycleHook {
                hooked_program,
                extra_accounts: None,
                data_authority: None,
                schema: ExternalPluginAdapterSchema::Binary,
            }),
            ExternalPluginAdapter::LinkedAppData(LinkedAppData {
                data_authority: data_authority(),
                schema: ExternalPluginAdapterSchema::Binary,
            }),
            ExternalPluginAdapter::DataSection(DataSection {
                parent_key: LinkedDataKey::LinkedAppData(data_authority()),
                schema: ExternalPluginAdapterSchema::Binary,
            }),
            ExternalPluginAdapter::AgentIdentity(AgentIdentity {
                uri: "https://example.com/agent.json".to_string(),
            }),
        ]
    }

    /// With no oracle account in `ctx.accounts` an Oracle errors; every other
    /// adapter takes a constant arm. The expectation per variant is spelled
    /// out so an added variant fails the test rather than slipping through.
    #[test]
    fn lifecycle_routers_cover_every_adapter_variant() {
        let mut signer = FakeAccount::wallet();
        let mut asset = FakeAccount::wallet();
        let signer_info = signer.info();
        let asset_info = asset.info();
        // `AgentIdentity` reads `ctx.accounts.last()` unguarded, so the slice
        // must not be empty.
        let accounts = vec![signer_info.clone()];
        let self_authority = Authority::UpdateAuthority;

        for adapter in every_adapter() {
            let adapter_type = ExternalPluginAdapterType::from(&adapter);
            let mut ctx = default_ctx(&accounts, &signer_info, &self_authority);
            ctx.asset_info = Some(&asset_info);

            // `validate_create`: only the DataSection arm is a hard reject;
            // the Oracle needs its account and errors without it.
            match adapter_type {
                ExternalPluginAdapterType::Oracle => {
                    assert!(ExternalPluginAdapter::validate_create(&adapter, &ctx).is_err());
                }
                ExternalPluginAdapterType::DataSection => {
                    assert_eq!(
                        ExternalPluginAdapter::validate_create(&adapter, &ctx).unwrap(),
                        ValidationResult::Rejected
                    );
                }
                ExternalPluginAdapterType::LinkedAppData => {
                    // Rejected on an asset, allowed on a collection.
                    assert_eq!(
                        ExternalPluginAdapter::validate_create(&adapter, &ctx).unwrap(),
                        ValidationResult::Rejected
                    );
                }
                ExternalPluginAdapterType::AgentIdentity => {
                    // Demands the identity PDA as the last account.
                    assert!(ExternalPluginAdapter::validate_create(&adapter, &ctx).is_err());
                }
                _ => {
                    assert_eq!(
                        ExternalPluginAdapter::validate_create(&adapter, &ctx).unwrap(),
                        ValidationResult::Pass,
                        "{adapter_type:?}::validate_create"
                    );
                }
            }

            // `validate_update` / `validate_burn` / `validate_transfer`:
            // everything but the Oracle abstains.
            for (name, result) in [
                (
                    "update",
                    ExternalPluginAdapter::validate_update(&adapter, &ctx),
                ),
                ("burn", ExternalPluginAdapter::validate_burn(&adapter, &ctx)),
                (
                    "transfer",
                    ExternalPluginAdapter::validate_transfer(&adapter, &ctx),
                ),
            ] {
                if adapter_type == ExternalPluginAdapterType::Oracle {
                    assert!(result.is_err(), "{adapter_type:?}::validate_{name}");
                } else {
                    assert_eq!(
                        result.unwrap(),
                        ValidationResult::Pass,
                        "{adapter_type:?}::validate_{name}"
                    );
                }
            }

            // `validate_execute` abstains for every variant today; the Oracle
            // is routed to the trait default rather than through
            // `validate_helper` (roadmap section 11, finding 3).
            assert_eq!(
                ExternalPluginAdapter::validate_execute(&adapter, &ctx).unwrap(),
                ValidationResult::Pass,
                "{adapter_type:?}::validate_execute"
            );

            // `validate_add_external_plugin_adapter`: only a DataSection is
            // refused outright; `AgentIdentity` demands its identity PDA and
            // errors without it.
            let result =
                ExternalPluginAdapter::validate_add_external_plugin_adapter(&adapter, &ctx);
            match adapter_type {
                ExternalPluginAdapterType::AgentIdentity => {
                    assert!(result.is_err(), "AgentIdentity demands a signing PDA");
                }
                ExternalPluginAdapterType::DataSection => {
                    assert_eq!(result.unwrap(), ValidationResult::Rejected);
                }
                _ => assert_eq!(
                    result.unwrap(),
                    ValidationResult::Pass,
                    "{adapter_type:?}::validate_add_external_plugin_adapter"
                ),
            }
        }
    }

    #[test]
    fn validate_update_external_plugin_adapter_is_an_authority_check() {
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let self_authority = Authority::UpdateAuthority;
        let resolved = [Authority::UpdateAuthority];
        let unrelated = [Authority::Owner];

        for adapter in every_adapter() {
            let adapter_type = ExternalPluginAdapterType::from(&adapter);
            let target = adapter.clone();

            // No resolved authorities: unreachable from the processors.
            let mut ctx = default_ctx(&[], &signer_info, &self_authority);
            ctx.target_external_plugin = Some(&target);
            assert!(
                ExternalPluginAdapter::validate_update_external_plugin_adapter(&adapter, &ctx)
                    .is_err()
            );

            // The signer resolves to the record authority and the target is
            // the same adapter type: approved (except a DataSection, which the
            // inner validator refuses outright).
            let mut ctx = default_ctx(&[], &signer_info, &self_authority);
            ctx.resolved_authorities = Some(&resolved);
            ctx.target_external_plugin = Some(&target);
            let expected = if adapter_type == ExternalPluginAdapterType::DataSection {
                ValidationResult::Rejected
            } else {
                ValidationResult::Approved
            };
            assert_eq!(
                ExternalPluginAdapter::validate_update_external_plugin_adapter(&adapter, &ctx)
                    .unwrap(),
                expected,
                "{adapter_type:?} with the record authority"
            );

            // A signer that does not resolve to the record authority: the base
            // result is `Pass`, which the processor turns into
            // `InvalidAuthority`.
            let mut ctx = default_ctx(&[], &signer_info, &self_authority);
            ctx.resolved_authorities = Some(&unrelated);
            ctx.target_external_plugin = Some(&target);
            let expected = if adapter_type == ExternalPluginAdapterType::DataSection {
                ValidationResult::Rejected
            } else {
                ValidationResult::Pass
            };
            assert_eq!(
                ExternalPluginAdapter::validate_update_external_plugin_adapter(&adapter, &ctx)
                    .unwrap(),
                expected,
                "{adapter_type:?} with a foreign authority"
            );
        }
    }

    #[test]
    fn check_create_and_check_execute_hook_arms() {
        let hooked_program = Pubkey::new_unique();
        let checks = |event| vec![(event, ExternalCheckResult { flags: 0x4 })];

        for event in [
            HookableLifecycleEvent::Create,
            HookableLifecycleEvent::Execute,
        ] {
            let hook = ExternalPluginAdapterInitInfo::LifecycleHook(LifecycleHookInitInfo {
                hooked_program,
                init_plugin_authority: None,
                lifecycle_checks: checks(event.clone()),
                extra_accounts: None,
                data_authority: None,
                schema: None,
            });
            let linked =
                ExternalPluginAdapterInitInfo::LinkedLifecycleHook(LinkedLifecycleHookInitInfo {
                    hooked_program,
                    init_plugin_authority: None,
                    lifecycle_checks: checks(event.clone()),
                    extra_accounts: None,
                    data_authority: None,
                    schema: None,
                });

            let is_create = event == HookableLifecycleEvent::Create;
            for init in [hook, linked] {
                assert_eq!(
                    ExternalPluginAdapter::check_create(&init),
                    if is_create {
                        ExternalCheckResult { flags: 0x4 }
                    } else {
                        ExternalCheckResult::none()
                    }
                );
                assert_eq!(
                    ExternalPluginAdapter::check_execute(&init),
                    if is_create {
                        ExternalCheckResult::none()
                    } else {
                        ExternalCheckResult { flags: 0x4 }
                    }
                );
            }
        }

        // `check_execute` for the remaining variants (no callers in the
        // program; roadmap section 11, finding 7).
        for init in [
            ExternalPluginAdapterInitInfo::LinkedAppData(crate::plugins::LinkedAppDataInitInfo {
                data_authority: data_authority(),
                init_plugin_authority: None,
                schema: None,
            }),
            ExternalPluginAdapterInitInfo::DataSection(crate::plugins::DataSectionInitInfo {
                parent_key: LinkedDataKey::LinkedAppData(data_authority()),
                schema: ExternalPluginAdapterSchema::Binary,
            }),
            ExternalPluginAdapterInitInfo::Oracle(crate::plugins::OracleInitInfo {
                base_address: Pubkey::new_unique(),
                init_plugin_authority: None,
                lifecycle_checks: vec![],
                base_address_config: None,
                results_offset: None,
            }),
            ExternalPluginAdapterInitInfo::AgentIdentity(crate::plugins::AgentIdentityInitInfo {
                uri: "u".to_string(),
                init_plugin_authority: None,
                lifecycle_checks: vec![],
            }),
        ] {
            assert_eq!(
                ExternalPluginAdapter::check_execute(&init),
                ExternalCheckResult::none()
            );
        }
    }

    #[test]
    fn key_from_record_reads_the_account_bytes_for_every_type() {
        for adapter in every_adapter() {
            let plugin_type = ExternalPluginAdapterType::from(&adapter);
            // The adapter bytes sit at `offset`, preceded by padding so the
            // offset arithmetic is exercised.
            let offset = 16usize;
            let mut data = vec![0u8; offset];
            data.extend(borsh::to_vec(&adapter).unwrap());
            let mut account = FakeAccount::with_data(data);
            let info = account.info();

            let record = ExternalRegistryRecord {
                plugin_type,
                authority: Authority::UpdateAuthority,
                lifecycle_checks: None,
                offset,
                data_offset: None,
                data_len: None,
            };
            assert_eq!(
                ExternalPluginAdapterKey::from_record(&info, &record).unwrap(),
                crate::plugins::test_ctx::adapter_key(&adapter),
                "from_record for {plugin_type:?}"
            );
        }
    }

    #[test]
    fn load_and_save_round_trip_through_an_account() {
        let adapter = ExternalPluginAdapter::AppData(AppData {
            data_authority: data_authority(),
            schema: ExternalPluginAdapterSchema::Binary,
        });
        let bytes = borsh::to_vec(&adapter).unwrap();
        let mut account = FakeAccount::with_data(vec![0u8; bytes.len() + 8]);
        let info = account.info();

        adapter.save(&info, 8).unwrap();
        assert_eq!(ExternalPluginAdapter::load(&info, 8).unwrap(), adapter);

        // A corrupted discriminator is a clean `DeserializationError`.
        info.data.borrow_mut()[8] = 0xFF;
        assert_eq!(
            ExternalPluginAdapter::load(&info, 8).unwrap_err(),
            MplCoreError::DeserializationError.into()
        );
    }
}
