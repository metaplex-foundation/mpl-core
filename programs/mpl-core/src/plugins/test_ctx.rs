//! Test-only scaffolding for the in-crate plugin unit tests.
//!
//! `PluginValidationContext` and most `validate_*` methods are `pub(crate)`,
//! so the branch-level tests for the plugin validators have to live inside the
//! crate. They need two things this module provides:
//!
//! - [`FakeAccount`], an owned buffer that hands out an `AccountInfo` pointing
//!   at it, so a validator can read an `AssetV1` out of `ctx.asset_info` or
//!   compare `authority_info.owner` without a runtime; and
//! - [`default_ctx`], a `PluginValidationContext` with every optional field
//!   `None`, which a test then fills in field by field (the fields are `pub`).
//!
//! Nothing here is compiled into the program.

use {
    crate::{
        plugins::PluginValidationContext,
        state::{Authority, Key},
    },
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
};

/// An owned account: the key, owner and data live here and [`FakeAccount::info`]
/// borrows them as an `AccountInfo`.
pub(crate) struct FakeAccount {
    /// The account address.
    pub key: Pubkey,
    /// The owning program.
    pub owner: Pubkey,
    /// The account balance.
    pub lamports: u64,
    /// The account data.
    pub data: Vec<u8>,
    /// Whether the `AccountInfo` reports the account as a signer.
    pub is_signer: bool,
}

impl FakeAccount {
    /// An empty account with a fresh key, owned by the given program.
    pub fn new(owner: Pubkey) -> Self {
        Self {
            key: Pubkey::new_unique(),
            owner,
            lamports: 1_000_000_000,
            data: vec![],
            is_signer: false,
        }
    }

    /// An empty account owned by the system program, i.e. a wallet. This is
    /// what `Royalties`' allow and deny lists see for a signer.
    pub fn wallet() -> Self {
        Self::new(solana_system_interface::program::ID)
    }

    /// A wallet with a specific address.
    pub fn wallet_at(key: Pubkey) -> Self {
        let mut account = Self::wallet();
        account.key = key;
        account
    }

    /// An mpl-core-owned account holding the serialized `value`.
    pub fn with_data(data: Vec<u8>) -> Self {
        let mut account = Self::new(crate::ID);
        account.data = data;
        account
    }

    /// An `AccountInfo` borrowing this account's fields.
    pub fn info(&mut self) -> AccountInfo<'_> {
        let Self {
            key,
            owner,
            lamports,
            data,
            is_signer,
        } = self;
        AccountInfo::new(
            key,
            *is_signer,
            false,
            lamports,
            data.as_mut_slice(),
            owner,
            false,
        )
    }
}

/// A `PluginValidationContext` with no accounts, `self_key = Key::AssetV1` and
/// every optional field `None`. Tests set the fields they care about.
pub(crate) fn default_ctx<'a, 'b>(
    accounts: &'a [AccountInfo<'a>],
    authority_info: &'a AccountInfo<'a>,
    self_authority: &'b Authority,
) -> PluginValidationContext<'a, 'b> {
    PluginValidationContext {
        accounts,
        asset_info: None,
        collection_info: None,
        self_key: Key::AssetV1,
        self_authority,
        authority_info,
        resolved_authorities: None,
        new_owner: None,
        new_asset_authority: None,
        new_collection_authority: None,
        target_plugin: None,
        target_plugin_authority: None,
        target_external_plugin: None,
        target_external_plugin_authority: None,
    }
}

/// Serializes `core` followed by a plugin header, the plugins and the
/// registry, byte for byte the way `initialize_plugin` lays an account out.
/// The unit tests use it wherever a validator reads a real account
/// (`fetch_plugin`, `list_plugins`, `UpdateDelegate::validate_update`).
pub(crate) fn core_account_bytes<T: borsh::BorshSerialize + crate::state::DataBlob>(
    core: &T,
    plugins: &[(crate::plugins::Plugin, Authority)],
) -> Vec<u8> {
    use crate::plugins::{PluginHeaderV1, PluginRegistryV1, PluginType, RegistryRecord};

    let mut data = borsh::to_vec(core).expect("core serializes");
    if plugins.is_empty() {
        return data;
    }
    let plugins_start = data.len() + 9;

    let mut body = Vec::new();
    let mut records = Vec::new();
    for (plugin, authority) in plugins {
        let offset = plugins_start + body.len();
        body.extend(borsh::to_vec(plugin).expect("plugin serializes"));
        records.push(RegistryRecord {
            plugin_type: PluginType::from(plugin),
            authority: *authority,
            offset,
        });
    }

    let registry = PluginRegistryV1 {
        key: Key::PluginRegistryV1,
        registry: records,
        external_registry: vec![],
    };
    let header = PluginHeaderV1 {
        key: Key::PluginHeaderV1,
        plugin_registry_offset: plugins_start + body.len(),
    };
    data.extend(borsh::to_vec(&header).expect("header serializes"));
    data.extend(body);
    data.extend(borsh::to_vec(&registry).expect("registry serializes"));
    data
}

/// An `AssetV1` owned and update-controlled by `owner`, with the given plugins.
pub(crate) fn asset_bytes(
    owner: Pubkey,
    plugins: &[(crate::plugins::Plugin, Authority)],
) -> Vec<u8> {
    use crate::state::{AssetV1, UpdateAuthority};

    core_account_bytes(
        &AssetV1 {
            key: Key::AssetV1,
            owner,
            update_authority: UpdateAuthority::Address(owner),
            name: "Test Asset".to_string(),
            uri: "https://example.com/test".to_string(),
            seq: None,
        },
        plugins,
    )
}

/// A `CollectionV1` controlled by `update_authority`, with the given plugins.
pub(crate) fn collection_bytes(
    update_authority: Pubkey,
    plugins: &[(crate::plugins::Plugin, Authority)],
) -> Vec<u8> {
    use crate::state::CollectionV1;

    core_account_bytes(
        &CollectionV1::new(
            update_authority,
            "Test Collection".to_string(),
            "https://example.com/test".to_string(),
            0,
            0,
        ),
        plugins,
    )
}

/// The registry key that identifies `adapter`, mirroring
/// `ExternalPluginAdapterKey::from_record` without touching an account.
pub(crate) fn adapter_key(
    adapter: &crate::plugins::ExternalPluginAdapter,
) -> crate::plugins::ExternalPluginAdapterKey {
    use crate::plugins::{ExternalPluginAdapter as A, ExternalPluginAdapterKey as K};

    match adapter {
        A::LifecycleHook(hook) => K::LifecycleHook(hook.hooked_program),
        A::Oracle(oracle) => K::Oracle(oracle.base_address),
        A::AppData(app_data) => K::AppData(app_data.data_authority),
        A::LinkedLifecycleHook(hook) => K::LinkedLifecycleHook(hook.hooked_program),
        A::LinkedAppData(app_data) => K::LinkedAppData(app_data.data_authority),
        A::DataSection(section) => K::DataSection(section.parent_key),
        A::AgentIdentity(_) => K::AgentIdentity,
    }
}
