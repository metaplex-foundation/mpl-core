use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use strum::EnumCount;

use super::{AdvancedLifecycleHook, Authority, DataStore, Oracle, SimpleLifecycleHook};

/// List of third party plugin types.
#[repr(C)]
#[derive(
    Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq, EnumCount, PartialOrd, Ord,
)]
pub enum ExternalPluginKey {
    /// Simple Lifecycle Hook.  Pick from a selection of several preconfigured PDAs for hooks.
    /// Attached `Pubkey` is the hooked program, which is called at specified lifecycle events and
    /// will return a validation result and new data to store.
    SimpleLifecycleHook(Pubkey),
    /// Advanced Lifecycle Hook.  Uses extra account to store account metas.  Attached `Pubkey` is
    /// the hooked program, which is called at specified lifecycle events and will return a
    /// validation result and new data to store.
    AdvancedLifecycleHook(Pubkey),
    /// Oracle.  Get a `ValidationResult` result from an account either specified by or derived
    /// from the attached `Pubkey`.
    Oracle(Pubkey),
    /// Data Store.  Arbitrary data that can be written to by the attached `Authority`.  Note this
    /// is different then the plugin authority as it cannot add/remove/revoke authority for the
    /// plugin.
    DataStore(Authority),
}

/// Definition of the external plugin header variants, each containing a link to the plugin header
/// struct.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum ExternalPluginHeader {
    /// Simple Lifecycle Hook.
    SimpleLifecycleHook(SimpleLifecycleHook),
    /// Advanced Lifecycle Hook.
    AdvancedLifecycleHook(AdvancedLifecycleHook),
    /// Oracle.
    Oracle(Oracle),
    /// Data Store.
    DataStore(DataStore),
}

/// A PDA with preconfigured known seeds.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum PreconfiguredPda {
    /// Program-based PDA.
    Program,
    /// Collection-based PDA.
    Collection,
    /// Owner-based PDA.
    Owner,
    /// Recipient-based PDA.
    Recipient,
    /// Asset-based PDA.
    Asset,
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
