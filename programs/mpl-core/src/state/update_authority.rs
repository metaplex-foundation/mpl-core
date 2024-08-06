use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// An enum representing the types of accounts that can update data on an asset.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub enum UpdateAuthority {
    /// No update authority, used for immutability.
    None,
    /// A standard address or PDA.
    Address(Pubkey),
    /// Authority delegated to a collection.
    Collection(Pubkey),
}

impl UpdateAuthority {
    /// Get the address of the update authority.
    pub fn key(&self) -> Pubkey {
        match self {
            Self::None => Pubkey::default(),
            Self::Address(address) => *address,
            Self::Collection(address) => *address,
        }
    }
}
