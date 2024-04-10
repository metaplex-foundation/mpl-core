use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use strum::EnumCount;

use super::{Authority, DataStore, ExternalCheckResult, LifecycleEvent, LifecycleHook, Oracle};

/// List of third party plugin types.
#[repr(C)]
#[derive(
    Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq, EnumCount, PartialOrd, Ord,
)]
pub enum ExternalPluginKey {
    /// Lifecycle Hook.  Extra accounts are specified in the external plugin header.  The attached
    /// `Pubkey` is the hooked program, which is called at specified lifecycle events and will
    /// return a validation result and new data to store.
    LifecycleHook(Pubkey),
    /// Oracle.  Get a `ValidationResult` result from an account either specified by or derived
    /// from the attached `Pubkey`.
    Oracle(Pubkey),
    /// Data Store.  Arbitrary data that can be written to by the attached `Authority`.  Note this
    /// is different then the plugin authority as it cannot update/revoke authority for the plugin.
    DataStore(Authority),
}

/// Definition of the external plugin variants, each containing a link to the external plugin
/// struct.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ExternalPlugin {
    /// Lifecycle Hook.
    LifecycleHook(LifecycleHook),
    /// Oracle.
    Oracle(Oracle),
    /// Data Store.
    DataStore(DataStore),
}

/// Required and optional information needed to initialize an external plugin.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct ExternalPluginInitInfo {
    /// External plugin key.
    pub plugin_key: ExternalPluginKey,
    /// External plugin.
    pub plugin: ExternalPlugin,
    /// Initial authority.
    pub init_authority: Option<Authority>,
    /// The lifecyle events for which the the external plugin is active.
    pub lifecycle_checks: Option<Vec<(LifecycleEvent, ExternalCheckResult)>>,
    /// External plugin initial data.
    pub data: Option<Vec<u8>>,
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
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ExternalPluginSchema {
    /// Raw binary data.
    Binary,
    /// JSON.
    Json,
    /// MessagePack serialized data.
    MsgPack,
}
