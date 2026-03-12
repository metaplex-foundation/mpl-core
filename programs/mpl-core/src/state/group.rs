use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use super::{CoreAsset, DataBlob, Key, SolanaAccount, UpdateAuthority};

/// Maximum number of entries per vector in a `GroupV1` account (collections, groups,
/// parent_groups, assets). Prevents unbounded growth that could cause compute
/// budget exhaustion on subsequent operations.
pub const MAX_GROUP_VECTOR_SIZE: usize = 256;

/// Maximum number of parent groups a single group may belong to. Acts as a
/// practical depth limit for group nesting since on-chain traversal is not
/// feasible.
pub const MAX_GROUP_NESTING_DEPTH: usize = 8;

/// The representation of a taxonomy group which can reference collections and other groups.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct GroupV1 {
    /// The account discriminator.
    pub key: Key, // 1
    /// The update authority for the group.
    pub update_authority: Pubkey, // 32
    /// The display name of the group.
    pub name: String, // 4
    /// The URI that links to the off-chain JSON describing the group.  
    /// Same semantics as collection URI.
    pub uri: String, // 4
    /// Collections that are direct children of this group.
    pub collections: Vec<Pubkey>, // 4 + 32 * N
    /// Groups that are direct children of this group.
    pub groups: Vec<Pubkey>, // 4 + 32 * N
    /// Groups that this group is a child of.
    pub parent_groups: Vec<Pubkey>, // 4 + 32 * N
    /// Assets that are direct members of this group.
    pub assets: Vec<Pubkey>, // 4 + 32 * N
}

impl GroupV1 {
    /// The base length of a group with empty name/uri and no relationships.
    const BASE_LEN: usize = 1 // Key
        + 32 // Update Authority
        + 4 // Name length
        + 4 // URI length
        + 4 // collections vec length
        + 4 // groups vec length
        + 4 // parent_groups vec length
        + 4; // assets vec length

    /// Create a new GroupV1 instance.
    pub fn new(
        update_authority: Pubkey,
        name: String,
        uri: String,
        collections: Vec<Pubkey>,
        groups: Vec<Pubkey>,
        parent_groups: Vec<Pubkey>,
        assets: Vec<Pubkey>,
    ) -> Self {
        Self {
            key: Key::GroupV1,
            update_authority,
            name,
            uri,
            collections,
            groups,
            parent_groups,
            assets,
        }
    }
}

impl DataBlob for GroupV1 {
    fn len(&self) -> usize {
        Self::BASE_LEN
            + self.name.len()
            + self.uri.len()
            + 32 * self.collections.len()
            + 32 * self.groups.len()
            + 32 * self.parent_groups.len()
            + 32 * self.assets.len()
    }
}

impl SolanaAccount for GroupV1 {
    fn key() -> Key {
        Key::GroupV1
    }
}

impl CoreAsset for GroupV1 {
    fn update_authority(&self) -> UpdateAuthority {
        UpdateAuthority::Address(self.update_authority)
    }

    fn owner(&self) -> &Pubkey {
        &self.update_authority
    }
}

/// Specifies the category of relationship a `Group` account has with another
/// account. This enum is used only for instruction input (not stored on chain)
/// to keep the `CreateGroup` API compact.
#[repr(u8)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub enum RelationshipKind {
    /// Relationship to a `Collection` account the group directly contains.
    Collection,
    /// Relationship to a child `Group` that this group directly contains.
    ChildGroup,
    /// Relationship to a parent `Group` that contains this group.
    ParentGroup,
    /// Relationship to an `Asset` account that is a direct member of the group.
    Asset,
}

/// Compact representation of a single relationship passed into
/// `CreateGroupV1Args`.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct RelationshipEntry {
    /// The kind of relationship.
    pub kind: RelationshipKind,
    /// The public key of the related account.
    pub key: Pubkey,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_len() {
        let groups = vec![
            GroupV1 {
                key: Key::GroupV1,
                update_authority: Pubkey::default(),
                name: "".to_string(),
                uri: "".to_string(),
                collections: vec![],
                groups: vec![],
                parent_groups: vec![],
                assets: vec![],
            },
            GroupV1 {
                key: Key::GroupV1,
                update_authority: Pubkey::default(),
                name: "test".to_string(),
                uri: "test".to_string(),
                collections: vec![Pubkey::new_unique(), Pubkey::new_unique()],
                groups: vec![Pubkey::new_unique()],
                parent_groups: vec![],
                assets: vec![],
            },
        ];
        for group in groups {
            let serialized = group.try_to_vec().unwrap();
            assert_eq!(serialized.len(), group.len());
        }
    }
}
