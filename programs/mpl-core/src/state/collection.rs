use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    instruction::accounts::{BurnCollectionAccounts, UpdateCollectionAccounts},
    plugins::{CheckResult, ValidationResult},
};

use super::{CoreAsset, DataBlob, Key, SolanaAccount, UpdateAuthority};

/// The representation of a collection of assets.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct Collection {
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

impl Collection {
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

    /// Check permissions for the transfer lifecycle event.
    pub fn check_transfer() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the burn lifecycle event.
    pub fn check_burn() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the update lifecycle event.
    pub fn check_update() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the compress lifecycle event.
    pub fn check_compress() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the decompress lifecycle event.
    pub fn check_decompress() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Validate the update lifecycle event.
    pub fn validate_update(
        &self,
        ctx: &UpdateCollectionAccounts,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.authority.key == &self.update_authority {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the burn lifecycle event.
    pub fn validate_burn(
        &self,
        ctx: &BurnCollectionAccounts,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.authority.key == &self.update_authority {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}

impl DataBlob for Collection {
    fn get_initial_size() -> usize {
        Self::BASE_LENGTH
    }

    fn get_size(&self) -> usize {
        Self::BASE_LENGTH + self.name.len() + self.uri.len()
    }
}

impl SolanaAccount for Collection {
    fn key() -> Key {
        Key::Collection
    }
}

impl CoreAsset for Collection {
    fn update_authority(&self) -> UpdateAuthority {
        UpdateAuthority::Collection(self.update_authority)
    }

    fn owner(&self) -> &Pubkey {
        &self.update_authority
    }
}
