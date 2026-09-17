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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_returns_the_inner_address() {
        let address = Pubkey::new_unique();
        assert_eq!(UpdateAuthority::Address(address).key(), address);

        let collection = Pubkey::new_unique();
        assert_eq!(UpdateAuthority::Collection(collection).key(), collection);
    }

    /// `None` resolves to the default pubkey, which is the system program id.
    /// `AssetV1::validate_update` compares the signer against this value, so
    /// the arm is safe only because the system program cannot sign. Asserted
    /// here so a refactor cannot quietly turn `None` into "anyone" (roadmap
    /// section 13, note 9).
    #[test]
    fn key_of_none_is_the_system_program_id() {
        assert_eq!(UpdateAuthority::None.key(), Pubkey::default());
        assert_eq!(
            UpdateAuthority::None.key(),
            solana_system_interface::program::ID
        );
        assert_ne!(UpdateAuthority::None.key(), Pubkey::new_unique());
    }
}
