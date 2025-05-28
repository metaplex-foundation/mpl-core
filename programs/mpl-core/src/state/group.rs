use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    plugins::{abstain, approve, CheckResult, ExternalPluginAdapter, Plugin, ValidationResult},
};

use super::{Authority, CoreAsset, DataBlob, Key, SolanaAccount, UpdateAuthority};

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

    /*
        Permission & validation helpers.
        Group accounts are meant to be lightweight; they do not impose permissions on
        asset/collection lifecycle events. They only enforce that the update authority
        (or its delegate) is the signer when the group itself is mutated.
    */

    /// Check permissions for mutating the group (add/remove/update parent/children, etc.).
    pub fn check_update() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Validate the authority for mutating the group.  Approves when the authority
    /// matches the group's update authority.
    pub fn validate_update(
        &self,
        authority_info: &AccountInfo,
        _plugin: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority_info.key == &self.update_authority {
            approve!()
        } else {
            abstain!()
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
