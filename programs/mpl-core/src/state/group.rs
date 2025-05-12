use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    state::{DataBlob, Key, SolanaAccount},
};

/// The authority type that governs how a plugin may be managed.
#[repr(u8)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum PluginAuthorityType {
    /// No authority may manage this plugin once set.
    None,
    /// The group update authority manages this plugin.
    UpdateAuthority,
    /// A specific address manages this plugin.
    SpecificAddress(Pubkey),
}

impl PluginAuthorityType {
    const BASE_LEN: usize = 1; // 1 byte discriminator

    fn additional_len(&self) -> usize {
        match self {
            PluginAuthorityType::SpecificAddress(_) => 32,
            _ => 0,
        }
    }
}

impl DataBlob for PluginAuthorityType {
    fn len(&self) -> usize {
        Self::BASE_LEN + self.additional_len()
    }
}

/// A record describing a plugin approved for use within a group.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct AllowedPlugin {
    /// The address of the plugin program or PDA that represents the plugin.
    pub address: Pubkey, // 32
    /// The authority type that may manage the plugin.
    pub authority_type: PluginAuthorityType, // 1 + optional 32
}

impl AllowedPlugin {
    const BASE_LEN: usize = 32 // address
        + PluginAuthorityType::BASE_LEN; // discriminator
}

impl DataBlob for AllowedPlugin {
    fn len(&self) -> usize {
        Self::BASE_LEN + self.authority_type.additional_len()
    }
}

/// The Group account that allows creators to logically organise assets and collections.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq, ShankAccount)]
pub struct GroupV1 {
    /// Account discriminator.
    pub key: Key, // 1
    /// Update authority for the group.
    pub update_authority: Pubkey, // 32
    /// Optional delegated authority that may act on behalf of the update authority.
    pub delegated_authority: Option<Pubkey>, // 1 + 32
    /// Human-readable name for the group.
    pub name: String, // 4 + len
    /// URI pointing at off-chain metadata for the group.
    pub uri: String, // 4 + len
    /// Collections that belong to this group.
    pub collections: Vec<Pubkey>, // 4 + n * 32
    /// Assets that belong to this group directly.
    pub assets: Vec<Pubkey>, // 4 + n * 32
    /// Direct child groups contained within this group.
    pub child_groups: Vec<Pubkey>, // 4 + n * 32
    /// Parent groups – groups that this group has been attached to.
    pub parent_groups: Vec<Pubkey>, // 4 + n * 32
    /// Plugins approved for this group.
    pub allowed_plugins: Vec<AllowedPlugin>, // 4 + n * AllowedPlugin
}

impl GroupV1 {
    /// Base size not accounting for variable-length fields.
    const BASE_LEN: usize = 1 // key
        + 32 // update_authority
        + 1 // Option<Pubkey> discriminator
        + 32 // delegated_authority 
        + 4 // name length prefix
        + 4 // uri length prefix
        + 4 // collections length
        + 4 // assets length
        + 4 // child_groups length
        + 4 // parent_groups length
        + 4; // allowed_plugins length

    /// Create a new GroupV1 instance with empty vectors and no delegate.
    pub fn new(update_authority: Pubkey, name: String, uri: String) -> Self {
        Self {
            key: Key::GroupV1,
            update_authority,
            delegated_authority: None,
            name,
            uri,
            collections: Vec::new(),
            assets: Vec::new(),
            child_groups: Vec::new(),
            parent_groups: Vec::new(),
            allowed_plugins: Vec::new(),
        }
    }

    /// Validate that the given pubkey is either the update authority or delegated authority.
    pub fn validate_authority(&self, authority: &Pubkey) -> Result<(), ProgramError> {
        if &self.update_authority != authority {
            match self.delegated_authority {
                Some(pubkey) if &pubkey == authority => Ok(()),
                _ => Err(MplCoreError::InvalidAuthority.into()),
            }
        } else {
            Ok(())
        }
    }

    /// Compute the space required for a Group account assuming the provided maxima.
    pub fn size(
        name: &str,
        uri: &str,
        max_collections: usize,
        max_assets: usize,
        max_groups: usize,
        max_plugins: usize,
    ) -> usize {
        // Worst-case size of an AllowedPlugin entry – 32-byte address + 1-byte
        // discriminator + up to 32-byte authority payload.
        const MAX_ALLOWED_PLUGIN_LEN: usize = AllowedPlugin::BASE_LEN + 32;

        Self::BASE_LEN
            + name.len()
            + uri.len()
            + max_collections * 32
            + max_assets * 32
            // Each group may appear both as child and parent
            + max_groups * 32 // child_groups
            + max_groups * 32 // parent_groups
            + max_plugins * MAX_ALLOWED_PLUGIN_LEN
    }
}

impl SolanaAccount for GroupV1 {
    fn key() -> Key {
        Key::GroupV1
    }
}

impl DataBlob for GroupV1 {
    fn len(&self) -> usize {
        Self::BASE_LEN
            + self.name.len()
            + self.uri.len()
            + self.collections.len() * 32
            + self.assets.len() * 32
            + self.child_groups.len() * 32
            + self.parent_groups.len() * 32
            + self.allowed_plugins.iter().map(|p| p.len()).sum::<usize>()
    }
}
