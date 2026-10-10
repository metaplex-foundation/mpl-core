//! Post-state parsing and layout invariants.
//!
//! Everything here uses the program's own types (`mpl_core_program::state`,
//! `mpl_core_program::plugins`) rather than the generated client, so the
//! readers do not depend on the client's deserialization being correct.

use {
    borsh::{BorshDeserialize, BorshSerialize},
    mpl_core_program::{
        plugins::{
            ExternalPluginAdapter, ExternalPluginAdapterKey, ExternalPluginAdapterType,
            ExternalRegistryRecord, Plugin, PluginHeaderV1, PluginRegistryV1, PluginType,
            RegistryRecord,
        },
        state::{AssetV1, Authority, CollectionV1, DataBlob, GroupV1, Key},
    },
    solana_account::Account,
};

/// Size of a serialized `PluginHeaderV1`.
const PLUGIN_HEADER_LEN: usize = super::accounts::PLUGIN_HEADER_LEN;

/// Everything after the core account: header, registry, and the decoded
/// plugins and adapters the registry points at.
#[derive(Clone, Debug)]
pub struct ParsedAccount {
    /// Serialized length of the core account (where the header starts).
    pub core_len: usize,
    /// The plugin header, if the account has plugin metadata.
    pub header: Option<PluginHeaderV1>,
    /// The plugin registry, if the account has plugin metadata.
    pub registry: Option<PluginRegistryV1>,
    /// Internal plugins in registry order.
    pub plugins: Vec<(RegistryRecord, Plugin)>,
    /// External adapters in registry order, each with its appended data.
    pub adapters: Vec<(
        ExternalRegistryRecord,
        ExternalPluginAdapter,
        Option<Vec<u8>>,
    )>,
}

impl ParsedAccount {
    /// Whether the account has a plugin header and registry.
    pub fn has_meta(&self) -> bool {
        self.header.is_some()
    }

    /// The internal plugin of the given type, with its authority.
    pub fn plugin(&self, plugin_type: PluginType) -> Option<(Authority, Plugin)> {
        self.plugins
            .iter()
            .find(|(record, _)| record.plugin_type == plugin_type)
            .map(|(record, plugin)| (record.authority, plugin.clone()))
    }

    /// The external adapter identified by `key`.
    pub fn adapter(
        &self,
        key: &ExternalPluginAdapterKey,
    ) -> Option<(
        ExternalRegistryRecord,
        ExternalPluginAdapter,
        Option<Vec<u8>>,
    )> {
        self.adapters
            .iter()
            .find(|(_, adapter, _)| adapter_key(adapter) == *key)
            .cloned()
    }
}

/// The registry key that identifies `adapter`.
pub fn adapter_key(adapter: &ExternalPluginAdapter) -> ExternalPluginAdapterKey {
    match adapter {
        ExternalPluginAdapter::LifecycleHook(hook) => {
            ExternalPluginAdapterKey::LifecycleHook(hook.hooked_program)
        }
        ExternalPluginAdapter::Oracle(oracle) => {
            ExternalPluginAdapterKey::Oracle(oracle.base_address)
        }
        ExternalPluginAdapter::AppData(app_data) => {
            ExternalPluginAdapterKey::AppData(app_data.data_authority)
        }
        ExternalPluginAdapter::LinkedLifecycleHook(hook) => {
            ExternalPluginAdapterKey::LinkedLifecycleHook(hook.hooked_program)
        }
        ExternalPluginAdapter::LinkedAppData(app_data) => {
            ExternalPluginAdapterKey::LinkedAppData(app_data.data_authority)
        }
        ExternalPluginAdapter::DataSection(section) => {
            ExternalPluginAdapterKey::DataSection(section.parent_key)
        }
        ExternalPluginAdapter::AgentIdentity(_) => ExternalPluginAdapterKey::AgentIdentity,
    }
}

fn deserialize_at<T: BorshDeserialize>(data: &[u8], offset: usize, what: &str) -> T {
    assert!(
        offset <= data.len(),
        "{what}: offset {offset} is past the end of the account ({} bytes)",
        data.len()
    );
    T::deserialize(&mut &data[offset..])
        .unwrap_or_else(|err| panic!("{what}: failed to deserialize at offset {offset}: {err}"))
}

/// Parses a core account of type `T` and the plugin metadata behind it.
/// Panics with a descriptive message on malformed data; use
/// [`assert_registry_consistent`] for the full invariant check.
pub fn parse_core<T: BorshDeserialize + DataBlob>(data: &[u8]) -> (T, ParsedAccount) {
    let core: T = deserialize_at(data, 0, "core account");
    let core_len = core.len();
    assert!(
        core_len <= data.len(),
        "core account claims {core_len} bytes but the account holds {}",
        data.len()
    );

    if core_len == data.len() {
        return (
            core,
            ParsedAccount {
                core_len,
                header: None,
                registry: None,
                plugins: vec![],
                adapters: vec![],
            },
        );
    }

    let header: PluginHeaderV1 = deserialize_at(data, core_len, "plugin header");
    let registry: PluginRegistryV1 =
        deserialize_at(data, header.plugin_registry_offset, "plugin registry");

    let plugins = registry
        .registry
        .iter()
        .map(|record| {
            let plugin: Plugin = deserialize_at(
                data,
                record.offset,
                &format!("{:?} plugin", record.plugin_type),
            );
            (record.clone(), plugin)
        })
        .collect();

    let adapters = registry
        .external_registry
        .iter()
        .map(|record| {
            let adapter: ExternalPluginAdapter = deserialize_at(
                data,
                record.offset,
                &format!("{:?} adapter", record.plugin_type),
            );
            let appended = match (record.data_offset, record.data_len) {
                (Some(offset), Some(len)) => {
                    assert!(
                        offset + len <= data.len(),
                        "{:?} adapter data [{offset}..{}) is past the end of the account",
                        record.plugin_type,
                        offset + len
                    );
                    Some(data[offset..offset + len].to_vec())
                }
                _ => None,
            };
            (record.clone(), adapter, appended)
        })
        .collect();

    (
        core,
        ParsedAccount {
            core_len,
            header: Some(header),
            registry: Some(registry),
            plugins,
            adapters,
        },
    )
}

/// Parses an `AssetV1` account.
pub fn parse_asset(data: &[u8]) -> (AssetV1, ParsedAccount) {
    parse_core::<AssetV1>(data)
}

/// Parses a `CollectionV1` account.
pub fn parse_collection(data: &[u8]) -> (CollectionV1, ParsedAccount) {
    parse_core::<CollectionV1>(data)
}

/// Parses the account by its discriminator (asset or collection).
pub fn parse_any(data: &[u8]) -> ParsedAccount {
    match key_of(data) {
        Key::AssetV1 => parse_asset(data).1,
        Key::CollectionV1 => parse_collection(data).1,
        other => panic!("expected an AssetV1 or CollectionV1 account, found {other:?}"),
    }
}

/// The discriminator of an account.
pub fn key_of(data: &[u8]) -> Key {
    deserialize_at(data, 0, "account key")
}

/// Reads the `AssetV1` at the start of the account.
pub fn read_asset(account: &Account) -> AssetV1 {
    parse_asset(&account.data).0
}

/// Reads the `CollectionV1` at the start of the account.
pub fn read_collection(account: &Account) -> CollectionV1 {
    parse_collection(&account.data).0
}

/// Reads a `GroupV1` account.
pub fn read_group(account: &Account) -> GroupV1 {
    deserialize_at(&account.data, 0, "group account")
}

/// Reads an internal plugin and its authority from an asset or collection.
pub fn read_plugin(account: &Account, plugin_type: PluginType) -> Option<(Authority, Plugin)> {
    parse_any(&account.data).plugin(plugin_type)
}

/// Reads an external adapter (record, adapter, appended data) from an asset or
/// collection.
pub fn read_adapter(
    account: &Account,
    key: &ExternalPluginAdapterKey,
) -> Option<(
    ExternalRegistryRecord,
    ExternalPluginAdapter,
    Option<Vec<u8>>,
)> {
    parse_any(&account.data).adapter(key)
}

fn serialized_len<T: BorshSerialize>(value: &T) -> usize {
    borsh::to_vec(value).expect("value serializes").len()
}

/// Checks the plugin layout invariants of an asset or collection account and
/// panics with a precise message on the first violation:
///
/// - header present iff registry present; header at `core.len()`;
/// - `plugin_registry_offset == data.len() - registry.len()`;
/// - every internal record deserializes to a `Plugin` of the recorded type;
/// - every external record deserializes to an adapter of the recorded type,
///   with `data_offset` / `data_len` either both absent or both present,
///   the data right after the adapter and before the registry;
/// - plugin, adapter and data bytes are contiguous in offset order and
///   exactly fill `core.len() + 9 .. plugin_registry_offset`.
pub fn assert_registry_consistent(account: &Account) {
    let data = &account.data;
    let core_len = match key_of(data) {
        Key::AssetV1 => deserialize_at::<AssetV1>(data, 0, "asset").len(),
        Key::CollectionV1 => deserialize_at::<CollectionV1>(data, 0, "collection").len(),
        other => panic!("expected an AssetV1 or CollectionV1 account, found {other:?}"),
    };
    assert!(
        core_len <= data.len(),
        "core account claims {core_len} bytes but the account holds {}",
        data.len()
    );
    if core_len == data.len() {
        // Bare account: no header, no registry.
        return;
    }

    assert!(
        data.len() >= core_len + PLUGIN_HEADER_LEN,
        "account has {} bytes after the core account, too few for a plugin header",
        data.len() - core_len
    );
    let header: PluginHeaderV1 = deserialize_at(data, core_len, "plugin header");
    assert_eq!(
        header.key,
        Key::PluginHeaderV1,
        "byte {core_len} should be the PluginHeaderV1 discriminator"
    );
    let registry_offset = header.plugin_registry_offset;
    assert!(
        registry_offset >= core_len + PLUGIN_HEADER_LEN && registry_offset < data.len(),
        "plugin_registry_offset {registry_offset} is outside {}..{}",
        core_len + PLUGIN_HEADER_LEN,
        data.len()
    );
    let registry: PluginRegistryV1 = deserialize_at(data, registry_offset, "plugin registry");
    assert_eq!(
        registry.key,
        Key::PluginRegistryV1,
        "byte {registry_offset} should be the PluginRegistryV1 discriminator"
    );
    assert_eq!(
        registry_offset,
        data.len() - serialized_len(&registry),
        "the registry must end exactly at the end of the account"
    );

    // (start, end, description) of every byte range the registry points at.
    let mut segments: Vec<(usize, usize, String)> = Vec::new();

    for record in &registry.registry {
        let what = format!("{:?} record", record.plugin_type);
        assert!(
            record.offset >= core_len + PLUGIN_HEADER_LEN && record.offset < registry_offset,
            "{what}: offset {} is outside the plugin area {}..{registry_offset}",
            record.offset,
            core_len + PLUGIN_HEADER_LEN
        );
        let plugin: Plugin = deserialize_at(data, record.offset, &what);
        assert_eq!(
            PluginType::from(&plugin),
            record.plugin_type,
            "{what}: the plugin at offset {} is a different type",
            record.offset
        );
        segments.push((record.offset, record.offset + serialized_len(&plugin), what));
    }

    for record in &registry.external_registry {
        let what = format!("{:?} adapter record", record.plugin_type);
        assert!(
            record.offset >= core_len + PLUGIN_HEADER_LEN && record.offset < registry_offset,
            "{what}: offset {} is outside the plugin area {}..{registry_offset}",
            record.offset,
            core_len + PLUGIN_HEADER_LEN
        );
        let adapter: ExternalPluginAdapter = deserialize_at(data, record.offset, &what);
        assert_eq!(
            ExternalPluginAdapterType::from(&adapter),
            record.plugin_type,
            "{what}: the adapter at offset {} is a different type",
            record.offset
        );
        let adapter_end = record.offset + serialized_len(&adapter);
        let end = match (record.data_offset, record.data_len) {
            (None, None) => adapter_end,
            (Some(data_offset), Some(data_len)) => {
                assert_eq!(
                    data_offset, adapter_end,
                    "{what}: data_offset must point right after the adapter bytes"
                );
                assert!(
                    data_offset + data_len <= registry_offset,
                    "{what}: data [{data_offset}..{}) overlaps the registry at {registry_offset}",
                    data_offset + data_len
                );
                data_offset + data_len
            }
            (data_offset, data_len) => panic!(
                "{what}: data_offset ({data_offset:?}) and data_len ({data_len:?}) must both be set or both be absent"
            ),
        };
        segments.push((record.offset, end, what));
    }

    segments.sort_by_key(|(start, _, _)| *start);
    let mut cursor = core_len + PLUGIN_HEADER_LEN;
    for (start, end, what) in &segments {
        assert_eq!(
            *start, cursor,
            "{what} starts at {start} but the previous entry ended at {cursor} (gap or overlap)"
        );
        cursor = *end;
    }
    assert_eq!(
        cursor, registry_offset,
        "plugin bytes end at {cursor} but the registry starts at {registry_offset} (trailing gap)"
    );
}
