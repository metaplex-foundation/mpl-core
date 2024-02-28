use crate::{error::MplCoreError, state::Key, utils::load_key};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, keccak, msg, program::invoke,
    program_error::ProgramError, pubkey::Pubkey,
};

/// A trait for generic blobs of data that have size.
pub trait DataBlob: BorshSerialize + BorshDeserialize {
    /// Get the size of an empty instance of the data blob.
    fn get_initial_size() -> usize;
    /// Get the current size of the data blob.
    fn get_size(&self) -> usize;
}

/// A trait for Solana accounts.
pub trait SolanaAccount: BorshSerialize + BorshDeserialize {
    /// Get the discriminator key for the account.
    fn key() -> Key;

    /// Load the account from the given account info starting at the offset.
    fn load(account: &AccountInfo, offset: u32) -> Result<Self, ProgramError> {
        let key = load_key(account, offset as usize)?;

        if key != Self::key() {
            return Err(MplCoreError::DeserializationError.into());
        }

        let mut bytes: &[u8] = &(*account.data).borrow()[(offset as usize)..];
        Self::deserialize(&mut bytes).map_err(|error| {
            msg!("Error: {}", error);
            MplCoreError::DeserializationError.into()
        })
    }

    /// Save the account to the given account info starting at the offset.
    fn save(&self, account: &AccountInfo, offset: u32) -> ProgramResult {
        borsh::to_writer(&mut account.data.borrow_mut()[(offset as usize)..], self).map_err(
            |error| {
                msg!("Error: {}", error);
                MplCoreError::SerializationError.into()
            },
        )
    }
}

/// A trait for assets that can be compressed and hashed.
pub trait Compressible: BorshSerialize + BorshDeserialize {
    /// Get the hash of the compressed data.
    fn hash(&self) -> Result<[u8; 32], ProgramError> {
        let serialized_data = self.try_to_vec()?;
        Ok(keccak::hash(serialized_data.as_slice()).to_bytes())
    }

    /// Write the data to ledger state by wrapping it in a noop instruction.
    fn wrap(&self) -> ProgramResult {
        let serialized_data = self.try_to_vec()?;
        invoke(&spl_noop::instruction(serialized_data), &[])
    }
}

/// A trait for core assets.
pub trait CoreAsset {
    /// Get the update authority of the asset.
    fn update_authority(&self) -> &Pubkey;

    /// Get the owner of the asset.
    fn owner(&self) -> &Pubkey;
}
