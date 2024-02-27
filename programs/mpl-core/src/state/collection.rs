use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use super::{DataBlob, Key, SolanaAccount};

/// The representation of a collection of assets.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct CollectionData {
    /// The account discriminator.
    pub key: Key, //1
    /// The update authority of the collection.
    pub update_authority: Pubkey, //32
    /// The name of the collection.
    pub name: String, //4
    /// The URI that links to what data to show for the collection.
    pub uri: String, //4
    /// The number of assets minted in the collection.
    pub num_minted: u32, //4
    /// The number of assets currently in the collection.
    pub current_size: u32, //4
}

impl CollectionData {
    /// The base length of the collection account with an empty name and uri.
    pub const BASE_LENGTH: usize = 1 + 32 + 4 + 4 + 4 + 4;

    /// Create a new collection.
    pub fn new(
        update_authority: Pubkey,
        name: String,
        uri: String,
        num_minted: u32,
        current_size: u32,
    ) -> Self {
        Self {
            key: Key::Collection,
            update_authority,
            name,
            uri,
            num_minted,
            current_size,
        }
    }
}

impl DataBlob for CollectionData {
    fn get_initial_size() -> usize {
        Self::BASE_LENGTH
    }

    fn get_size(&self) -> usize {
        Self::BASE_LENGTH + self.name.len() + self.uri.len()
    }
}

impl SolanaAccount for CollectionData {
    fn key() -> Key {
        Key::Collection
    }
}
