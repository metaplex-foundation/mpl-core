//! Raw account builders.
//!
//! These lay out account data byte for byte the way the program does (see
//! `initialize_plugin` / `initialize_external_plugin_adapter` in
//! `src/plugins/utils.rs`), without running any instruction. They are the
//! only way to build adversarial layouts (foreign owners, wrong
//! discriminators, corrupt registries), so they stay even though
//! [`super::fixture::Fixture`] can create well-formed accounts through the
//! program. The `harness` self-check tests prove that a spec built here is
//! byte-identical to what `CreateV2` / `CreateCollectionV2` produce.
//!
//! On-chain layout:
//!
//! ```text
//! [core (AssetV1 | CollectionV1)]
//! [PluginHeaderV1: Key + plugin_registry_offset (9 bytes)]
//! [plugin / adapter bytes in registry order; data adapters are followed by their data]
//! [PluginRegistryV1]
//! ```
//!
//! A "bare" account has no header at all: `data.len() == core.len()`.

use {
    borsh::BorshSerialize,
    mpl_core_program::{
        plugins::{
            AgentIdentity, ExternalCheckResult, ExternalPluginAdapter, ExternalPluginAdapterType,
            ExternalRegistryRecord, HookableLifecycleEvent, OracleValidation, Plugin,
            PluginHeaderV1, PluginRegistryV1, PluginType, RegistryRecord,
        },
        state::{
            AssetV1, Authority, CollectionV1, DataBlob, GroupV1, HashedAssetV1, Key, SolanaAccount,
            UpdateAuthority,
        },
        ID as MPL_CORE_ID,
    },
    solana_account::Account,
    solana_program::pubkey::Pubkey,
    solana_system_interface::program as system_program,
};

/// Lamports given to every account built here (comfortably rent exempt).
pub const ACCOUNT_LAMPORTS: u64 = 1_000_000_000;

/// Size of a serialized `PluginHeaderV1`: `Key` (1) + `usize` offset (8).
pub const PLUGIN_HEADER_LEN: usize = 9;

/// Program id of mpl-agent-tools, the owner of `ExecutionDelegateRecordV1`.
pub const MPL_AGENT_TOOLS_ID: Pubkey = mpl_agent_tools::ID;

/// Program id of mpl-agent-identity, which owns the agent identity PDAs.
pub const MPL_AGENT_IDENTITY_ID: Pubkey = mpl_agent_identity::ID;

/// Default name for assets built by [`AssetSpec`].
pub const DEFAULT_ASSET_NAME: &str = "Test Asset";
/// Default name for collections built by [`CollectionSpec`].
pub const DEFAULT_COLLECTION_NAME: &str = "Test Collection";
/// Default URI for assets and collections built here.
pub const DEFAULT_URI: &str = "https://example.com/test";

/// A system-owned account holding `lamports` (a payer).
pub fn payer_account(lamports: u64) -> Account {
    Account {
        lamports,
        data: vec![],
        owner: system_program::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// A brand new system account: no lamports, no data. This is what a `create`
/// instruction expects for the account it is about to allocate.
pub fn empty_account() -> Account {
    payer_account(0)
}

/// A system-owned account carrying arbitrary bytes (e.g. an app-data buffer).
pub fn buffer_account(bytes: Vec<u8>) -> Account {
    Account {
        lamports: ACCOUNT_LAMPORTS,
        data: bytes,
        owner: system_program::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// An external plugin adapter as it should appear in the registry.
#[derive(Clone, Debug)]
pub struct ExternalAdapterSpec {
    /// The adapter itself.
    pub adapter: ExternalPluginAdapter,
    /// The plugin authority recorded in the registry.
    pub authority: Authority,
    /// The lifecycle checks recorded in the registry.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// Appended data (only meaningful for `LifecycleHook`, `AppData` and
    /// `DataSection`; those get `data_len = Some(0)` when this is `None`).
    pub data: Option<Vec<u8>>,
}

impl ExternalAdapterSpec {
    /// An adapter managed by the update authority with no checks and no data.
    pub fn new(adapter: ExternalPluginAdapter) -> Self {
        Self {
            adapter,
            authority: Authority::UpdateAuthority,
            lifecycle_checks: None,
            data: None,
        }
    }

    /// An `AgentIdentity` adapter with the given URI and lifecycle checks.
    pub fn agent_identity(
        uri: &str,
        lifecycle_checks: Vec<(HookableLifecycleEvent, ExternalCheckResult)>,
    ) -> Self {
        Self::new(ExternalPluginAdapter::AgentIdentity(AgentIdentity {
            uri: uri.to_string(),
        }))
        .lifecycle_checks(lifecycle_checks)
    }

    /// Sets the plugin authority.
    pub fn authority(mut self, authority: Authority) -> Self {
        self.authority = authority;
        self
    }

    /// Sets the lifecycle checks.
    pub fn lifecycle_checks(
        mut self,
        checks: Vec<(HookableLifecycleEvent, ExternalCheckResult)>,
    ) -> Self {
        self.lifecycle_checks = Some(checks);
        self
    }

    /// Sets the appended data.
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Whether the program stores `data_offset` / `data_len` for this adapter.
    pub fn carries_data(&self) -> bool {
        matches!(
            self.adapter,
            ExternalPluginAdapter::LifecycleHook(_)
                | ExternalPluginAdapter::AppData(_)
                | ExternalPluginAdapter::DataSection(_)
        )
    }
}

/// Serializes `core` followed by a plugin header, the plugins, the adapters
/// (each with its data) and the registry. Internal plugins come first, then
/// adapters, which is the order `CreateV2` initializes them in.
fn layout<T: DataBlob + BorshSerialize>(
    core: &T,
    plugins: &[(Plugin, Authority)],
    adapters: &[ExternalAdapterSpec],
) -> Vec<u8> {
    let mut data = borsh::to_vec(core).expect("core serializes");
    assert_eq!(
        data.len(),
        core.len(),
        "DataBlob::len disagrees with the serialized length of the core account"
    );
    let plugins_start = data.len() + PLUGIN_HEADER_LEN;

    let mut body = Vec::new();
    let mut registry = PluginRegistryV1 {
        key: Key::PluginRegistryV1,
        registry: vec![],
        external_registry: vec![],
    };

    for (plugin, authority) in plugins {
        let offset = plugins_start + body.len();
        body.extend(borsh::to_vec(plugin).expect("plugin serializes"));
        registry.registry.push(RegistryRecord {
            plugin_type: PluginType::from(plugin),
            authority: *authority,
            offset,
        });
    }

    for spec in adapters {
        let offset = plugins_start + body.len();
        let adapter_bytes = borsh::to_vec(&spec.adapter).expect("adapter serializes");
        body.extend(&adapter_bytes);
        let (data_offset, data_len) = if spec.carries_data() {
            let appended = spec.data.as_deref().unwrap_or(&[]);
            body.extend(appended);
            (Some(offset + adapter_bytes.len()), Some(appended.len()))
        } else {
            assert!(
                spec.data.is_none(),
                "{:?} adapters carry no data; the program never stores data for them",
                ExternalPluginAdapterType::from(&spec.adapter)
            );
            (None, None)
        };
        registry.external_registry.push(ExternalRegistryRecord {
            plugin_type: ExternalPluginAdapterType::from(&spec.adapter),
            authority: spec.authority,
            lifecycle_checks: spec.lifecycle_checks.clone(),
            offset,
            data_offset,
            data_len,
        });
    }

    let header = PluginHeaderV1 {
        key: Key::PluginHeaderV1,
        plugin_registry_offset: plugins_start + body.len(),
    };
    data.extend(borsh::to_vec(&header).expect("header serializes"));
    data.extend(body);
    data.extend(borsh::to_vec(&registry).expect("registry serializes"));
    data
}

fn owned_account(data: Vec<u8>, lamports: u64, program_owner: Pubkey) -> Account {
    Account {
        lamports,
        data,
        owner: program_owner,
        executable: false,
        rent_epoch: 0,
    }
}

/// Builds an account for `core` with the given plugins and adapters. When
/// both lists are empty the account is bare (no header); use
/// [`build_core_account_with_meta`] for an empty header + registry.
pub fn build_core_account<T: DataBlob + SolanaAccount + BorshSerialize>(
    core: &T,
    plugins: &[(Plugin, Authority)],
    adapters: &[ExternalAdapterSpec],
    lamports: u64,
    program_owner: Pubkey,
) -> Account {
    if plugins.is_empty() && adapters.is_empty() {
        let data = borsh::to_vec(core).expect("core serializes");
        owned_account(data, lamports, program_owner)
    } else {
        build_core_account_with_meta(core, plugins, adapters, lamports, program_owner)
    }
}

/// Like [`build_core_account`], but always writes the plugin header and
/// registry, even when they are empty (the state after every plugin was
/// removed).
pub fn build_core_account_with_meta<T: DataBlob + SolanaAccount + BorshSerialize>(
    core: &T,
    plugins: &[(Plugin, Authority)],
    adapters: &[ExternalAdapterSpec],
    lamports: u64,
    program_owner: Pubkey,
) -> Account {
    owned_account(layout(core, plugins, adapters), lamports, program_owner)
}

/// Describes an `AssetV1` account to build.
#[derive(Clone, Debug)]
pub struct AssetSpec {
    /// The asset owner.
    pub owner: Pubkey,
    /// The update authority (defaults to `Address(owner)`).
    pub update_authority: UpdateAuthority,
    /// The asset name.
    pub name: String,
    /// The asset URI.
    pub uri: String,
    /// The compression sequence number.
    pub seq: Option<u64>,
    /// Internal plugins with their authorities, in registry order.
    pub plugins: Vec<(Plugin, Authority)>,
    /// External adapters, in registry order (after the internal plugins).
    pub adapters: Vec<ExternalAdapterSpec>,
    /// Lamports on the account.
    pub lamports: u64,
    /// The program that owns the account (mpl-core unless testing lookalikes).
    pub program_owner: Pubkey,
    /// Write an empty header + registry even when there are no plugins.
    pub empty_meta: bool,
}

impl AssetSpec {
    /// An asset owned and update-controlled by `owner`, with default name and URI.
    pub fn new(owner: Pubkey) -> Self {
        Self {
            owner,
            update_authority: UpdateAuthority::Address(owner),
            name: DEFAULT_ASSET_NAME.to_string(),
            uri: DEFAULT_URI.to_string(),
            seq: None,
            plugins: vec![],
            adapters: vec![],
            lamports: ACCOUNT_LAMPORTS,
            program_owner: MPL_CORE_ID,
            empty_meta: false,
        }
    }

    /// Sets the update authority.
    pub fn update_authority(mut self, update_authority: UpdateAuthority) -> Self {
        self.update_authority = update_authority;
        self
    }

    /// Sets the name.
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Sets the URI.
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    /// Sets the compression sequence number.
    pub fn seq(mut self, seq: u64) -> Self {
        self.seq = Some(seq);
        self
    }

    /// Appends an internal plugin.
    pub fn plugin(mut self, plugin: Plugin, authority: Authority) -> Self {
        self.plugins.push((plugin, authority));
        self
    }

    /// Appends an external adapter.
    pub fn adapter(mut self, spec: ExternalAdapterSpec) -> Self {
        self.adapters.push(spec);
        self
    }

    /// Sets the lamports.
    pub fn lamports(mut self, lamports: u64) -> Self {
        self.lamports = lamports;
        self
    }

    /// Sets the owning program.
    pub fn program_owner(mut self, program_owner: Pubkey) -> Self {
        self.program_owner = program_owner;
        self
    }

    /// Writes an empty header + registry even with no plugins.
    pub fn with_empty_meta(mut self) -> Self {
        self.empty_meta = true;
        self
    }

    /// The core `AssetV1` this spec describes.
    pub fn core(&self) -> AssetV1 {
        AssetV1 {
            key: Key::AssetV1,
            owner: self.owner,
            update_authority: self.update_authority.clone(),
            name: self.name.clone(),
            uri: self.uri.clone(),
            seq: self.seq,
        }
    }

    /// Builds the account.
    pub fn build(&self) -> Account {
        let core = self.core();
        if self.empty_meta {
            build_core_account_with_meta(
                &core,
                &self.plugins,
                &self.adapters,
                self.lamports,
                self.program_owner,
            )
        } else {
            build_core_account(
                &core,
                &self.plugins,
                &self.adapters,
                self.lamports,
                self.program_owner,
            )
        }
    }
}

/// Describes a `CollectionV1` account to build.
#[derive(Clone, Debug)]
pub struct CollectionSpec {
    /// The collection update authority.
    pub update_authority: Pubkey,
    /// The collection name.
    pub name: String,
    /// The collection URI.
    pub uri: String,
    /// Assets ever minted into the collection.
    pub num_minted: u32,
    /// Assets currently in the collection.
    pub current_size: u32,
    /// Internal plugins with their authorities, in registry order.
    pub plugins: Vec<(Plugin, Authority)>,
    /// External adapters, in registry order (after the internal plugins).
    pub adapters: Vec<ExternalAdapterSpec>,
    /// Lamports on the account.
    pub lamports: u64,
    /// The program that owns the account.
    pub program_owner: Pubkey,
    /// Write an empty header + registry even when there are no plugins.
    pub empty_meta: bool,
}

impl CollectionSpec {
    /// An empty collection controlled by `update_authority`.
    pub fn new(update_authority: Pubkey) -> Self {
        Self {
            update_authority,
            name: DEFAULT_COLLECTION_NAME.to_string(),
            uri: DEFAULT_URI.to_string(),
            num_minted: 0,
            current_size: 0,
            plugins: vec![],
            adapters: vec![],
            lamports: ACCOUNT_LAMPORTS,
            program_owner: MPL_CORE_ID,
            empty_meta: false,
        }
    }

    /// Sets the name.
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Sets the URI.
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    /// Sets `num_minted` and `current_size`.
    pub fn sizes(mut self, num_minted: u32, current_size: u32) -> Self {
        self.num_minted = num_minted;
        self.current_size = current_size;
        self
    }

    /// Appends an internal plugin.
    pub fn plugin(mut self, plugin: Plugin, authority: Authority) -> Self {
        self.plugins.push((plugin, authority));
        self
    }

    /// Appends an external adapter.
    pub fn adapter(mut self, spec: ExternalAdapterSpec) -> Self {
        self.adapters.push(spec);
        self
    }

    /// Sets the lamports.
    pub fn lamports(mut self, lamports: u64) -> Self {
        self.lamports = lamports;
        self
    }

    /// Sets the owning program.
    pub fn program_owner(mut self, program_owner: Pubkey) -> Self {
        self.program_owner = program_owner;
        self
    }

    /// Writes an empty header + registry even with no plugins.
    pub fn with_empty_meta(mut self) -> Self {
        self.empty_meta = true;
        self
    }

    /// The core `CollectionV1` this spec describes.
    pub fn core(&self) -> CollectionV1 {
        CollectionV1::new(
            self.update_authority,
            self.name.clone(),
            self.uri.clone(),
            self.num_minted,
            self.current_size,
        )
    }

    /// Builds the account.
    pub fn build(&self) -> Account {
        let core = self.core();
        if self.empty_meta {
            build_core_account_with_meta(
                &core,
                &self.plugins,
                &self.adapters,
                self.lamports,
                self.program_owner,
            )
        } else {
            build_core_account(
                &core,
                &self.plugins,
                &self.adapters,
                self.lamports,
                self.program_owner,
            )
        }
    }
}

/// A `GroupV1` account owned by mpl-core.
pub fn group_account(
    update_authority: Pubkey,
    name: &str,
    uri: &str,
    collections: Vec<Pubkey>,
    groups: Vec<Pubkey>,
    parent_groups: Vec<Pubkey>,
    assets: Vec<Pubkey>,
) -> Account {
    let group = GroupV1::new(
        update_authority,
        name.to_string(),
        uri.to_string(),
        collections,
        groups,
        parent_groups,
        assets,
    );
    owned_account(
        borsh::to_vec(&group).expect("group serializes"),
        ACCOUNT_LAMPORTS,
        MPL_CORE_ID,
    )
}

/// `n` fresh unique keys, for the group size-limit tests.
pub fn random_keys(n: usize) -> Vec<Pubkey> {
    (0..n).map(|_| Pubkey::new_unique()).collect()
}

/// A one-byte account carrying only the `HashedAssetV1` discriminator, for
/// the `NotAvailable` guards on compressed assets.
pub fn hashed_asset_placeholder() -> Account {
    owned_account(
        vec![Key::HashedAssetV1 as u8],
        ACCOUNT_LAMPORTS,
        MPL_CORE_ID,
    )
}

/// A full `HashedAssetV1` account owned by mpl-core.
pub fn hashed_asset_account(hash: [u8; 32]) -> Account {
    owned_account(
        borsh::to_vec(&HashedAssetV1::new(hash)).expect("hashed asset serializes"),
        ACCOUNT_LAMPORTS,
        MPL_CORE_ID,
    )
}

/// An oracle account: `offset` zero bytes followed by the borsh-serialized
/// validation. The program does not check the owner, so a fresh key is used.
pub fn oracle_account(validation: &OracleValidation, offset: usize) -> Account {
    let mut data = vec![0u8; offset];
    data.extend(borsh::to_vec(validation).expect("oracle validation serializes"));
    owned_account(data, ACCOUNT_LAMPORTS, Pubkey::new_unique())
}

/// An `ExecutionDelegateRecordV1` account owned by mpl-agent-tools, serialized
/// from the client's own account type (104 bytes: key, bump, 6 bytes padding,
/// executive profile, authority, agent asset).
pub fn execution_delegate_record(
    executive_profile: Pubkey,
    authority: Pubkey,
    agent_asset: Pubkey,
) -> Account {
    let record = mpl_agent_tools::accounts::ExecutionDelegateRecordV1 {
        key: mpl_agent_tools::types::Key::ExecutionDelegateRecordV1,
        bump: 255,
        padding: [0u8; 6],
        executive_profile,
        authority,
        agent_asset,
    };
    let data = borsh::to_vec(&record).expect("delegate record serializes");
    assert_eq!(
        data.len(),
        mpl_agent_tools::accounts::ExecutionDelegateRecordV1::LEN,
        "ExecutionDelegateRecordV1 serialized to an unexpected length"
    );
    owned_account(data, ACCOUNT_LAMPORTS, MPL_AGENT_TOOLS_ID)
}

/// The agent identity PDA of `asset` under mpl-agent-identity.
pub fn agent_identity_pda(asset: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"agent_identity", asset.as_ref()], &MPL_AGENT_IDENTITY_ID)
}

/// The signing PDA mpl-core uses for `ExecuteV1` on `asset`.
pub fn asset_signer_pda(asset: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"mpl-core-execute", asset.as_ref()], &MPL_CORE_ID)
}

// ---------------------------------------------------------------------------
// Added with the `account_ownership` port.
// ---------------------------------------------------------------------------

/// An account owned by mpl-core carrying arbitrary bytes (a wrong
/// discriminator, garbage, or no data at all), for the `load_key` guards.
pub fn core_bytes_account(bytes: Vec<u8>) -> Account {
    owned_account(bytes, ACCOUNT_LAMPORTS, MPL_CORE_ID)
}

// ---------------------------------------------------------------------------
// --- added for m3 ---
//
// Group accounts with a configurable owner and lamports, the hashed-asset
// fixture that matches a `CompressionProof`, and the rent helper the close /
// resize paths are asserted against.
// ---------------------------------------------------------------------------

use mpl_core_program::state::{
    Compressible, CompressionProof, HashablePluginSchema, HashedAssetSchema,
};

/// Describes a `GroupV1` account to build. The sibling of [`AssetSpec`] and
/// [`CollectionSpec`]; [`group_account`] is the terse form for the common
/// case (owned by mpl-core, rent exempt).
#[derive(Clone, Debug)]
pub struct GroupSpec {
    /// The group update authority.
    pub update_authority: Pubkey,
    /// The group name.
    pub name: String,
    /// The group URI.
    pub uri: String,
    /// Collections that are direct children of the group.
    pub collections: Vec<Pubkey>,
    /// Groups that are direct children of the group.
    pub groups: Vec<Pubkey>,
    /// Groups the group is a child of.
    pub parent_groups: Vec<Pubkey>,
    /// Assets that are direct members of the group.
    pub assets: Vec<Pubkey>,
    /// Lamports on the account.
    pub lamports: u64,
    /// The program that owns the account (mpl-core unless testing lookalikes).
    pub program_owner: Pubkey,
}

/// Default name for groups built by [`GroupSpec`].
pub const DEFAULT_GROUP_NAME: &str = "Test Group";

impl GroupSpec {
    /// An empty group controlled by `update_authority`.
    pub fn new(update_authority: Pubkey) -> Self {
        Self {
            update_authority,
            name: DEFAULT_GROUP_NAME.to_string(),
            uri: DEFAULT_URI.to_string(),
            collections: vec![],
            groups: vec![],
            parent_groups: vec![],
            assets: vec![],
            lamports: ACCOUNT_LAMPORTS,
            program_owner: MPL_CORE_ID,
        }
    }

    /// Sets the name.
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Sets the URI.
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    /// Sets the member collections.
    pub fn collections(mut self, collections: Vec<Pubkey>) -> Self {
        self.collections = collections;
        self
    }

    /// Sets the child groups.
    pub fn groups(mut self, groups: Vec<Pubkey>) -> Self {
        self.groups = groups;
        self
    }

    /// Sets the parent groups.
    pub fn parent_groups(mut self, parent_groups: Vec<Pubkey>) -> Self {
        self.parent_groups = parent_groups;
        self
    }

    /// Sets the member assets.
    pub fn assets(mut self, assets: Vec<Pubkey>) -> Self {
        self.assets = assets;
        self
    }

    /// Sets the lamports.
    pub fn lamports(mut self, lamports: u64) -> Self {
        self.lamports = lamports;
        self
    }

    /// Sets the owning program.
    pub fn program_owner(mut self, program_owner: Pubkey) -> Self {
        self.program_owner = program_owner;
        self
    }

    /// The core `GroupV1` this spec describes.
    pub fn core(&self) -> GroupV1 {
        GroupV1::new(
            self.update_authority,
            self.name.clone(),
            self.uri.clone(),
            self.collections.clone(),
            self.groups.clone(),
            self.parent_groups.clone(),
            self.assets.clone(),
        )
    }

    /// Builds the account.
    pub fn build(&self) -> Account {
        owned_account(
            borsh::to_vec(&self.core()).expect("group serializes"),
            self.lamports,
            self.program_owner,
        )
    }
}

/// The rent-exempt minimum balance for an account of `len` bytes, under the
/// default rent Mollusk runs with. Used to assert the refunds and top-ups of
/// `close_program_account` and `resize_or_reallocate_account`.
pub fn rent_exempt_balance(len: usize) -> u64 {
    solana_program::rent::Rent::default().minimum_balance(len)
}

/// The `HashedAssetV1` account a `CompressionProof` hashes to, built exactly
/// the way `utils::compression::verify_proof` recomputes it: the asset hash
/// over `AssetV1::from(proof)` (so `seq == Some(proof.seq)`) and one hash per
/// plugin, in ascending `index` order.
///
/// Pass the proof's plugins unsorted to make the sort in `verify_proof`
/// meaningful; the fixture sorts a copy, as the program does.
pub fn hashed_asset_for_proof(proof: &CompressionProof) -> Account {
    hashed_asset_account(hashed_asset_schema_for_proof(proof))
}

/// The hash `verify_proof` compares the account against (see
/// [`hashed_asset_for_proof`]).
pub fn hashed_asset_schema_for_proof(proof: &CompressionProof) -> [u8; 32] {
    let asset = AssetV1::from(proof.clone());
    let mut plugins = proof.plugins.clone();
    plugins.sort_by(HashablePluginSchema::compare_indeces);
    let schema = HashedAssetSchema {
        asset_hash: asset.hash().expect("asset hashes"),
        plugin_hashes: plugins
            .iter()
            .map(|plugin| plugin.hash().expect("plugin hashes"))
            .collect(),
    };
    schema.hash().expect("schema hashes")
}

/// Rewrites the registry record of the `from` plugin so it claims to be a
/// `to` plugin, leaving the plugin bytes themselves untouched. `PluginType`
/// is a one-byte discriminant, so the account length does not change.
///
/// Produces a corrupt account no instruction of the program can write, which
/// is exactly what the `InvalidPlugin` arms guard against.
pub fn retype_registry_record(account: &Account, from: PluginType, to: PluginType) -> Account {
    let parsed = super::read::parse_any(&account.data);
    let header = parsed.header.expect("account has no plugin header");
    let mut registry = parsed.registry.expect("account has no plugin registry");
    let record = registry
        .registry
        .iter_mut()
        .find(|record| record.plugin_type == from)
        .unwrap_or_else(|| panic!("account has no {from:?} registry record"));
    record.plugin_type = to;
    overwrite_registry(account, header.plugin_registry_offset, &registry)
}

/// Points the registry record of `plugin_type` at `offset`, which may be
/// anywhere in (or past) the account.
pub fn repoint_registry_record(
    account: &Account,
    plugin_type: PluginType,
    offset: usize,
) -> Account {
    let parsed = super::read::parse_any(&account.data);
    let header = parsed.header.expect("account has no plugin header");
    let mut registry = parsed.registry.expect("account has no plugin registry");
    let record = registry
        .registry
        .iter_mut()
        .find(|record| record.plugin_type == plugin_type)
        .unwrap_or_else(|| panic!("account has no {plugin_type:?} registry record"));
    record.offset = offset;
    overwrite_registry(account, header.plugin_registry_offset, &registry)
}

/// Serializes `registry` over the account data at `offset`; the caller must
/// keep the serialized length unchanged.
fn overwrite_registry(account: &Account, offset: usize, registry: &PluginRegistryV1) -> Account {
    let bytes = borsh::to_vec(registry).expect("registry serializes");
    let mut account = account.clone();
    assert_eq!(
        offset + bytes.len(),
        account.data.len(),
        "the rewritten registry must keep the account length"
    );
    account.data[offset..].copy_from_slice(&bytes);
    account
}

/// Cuts an account's data down to `len` bytes, keeping its owner and
/// lamports: a valid discriminator followed by a truncated payload.
pub fn truncate_account(account: &Account, len: usize) -> Account {
    let mut account = account.clone();
    assert!(
        len <= account.data.len(),
        "cannot truncate {} bytes to {len}",
        account.data.len()
    );
    account.data.truncate(len);
    account
}

/// Overwrites the account's bytes from `offset` with `bytes`, in place.
pub fn overwrite_bytes(account: &Account, offset: usize, bytes: &[u8]) -> Account {
    let mut account = account.clone();
    account.data[offset..offset + bytes.len()].copy_from_slice(bytes);
    account
}
