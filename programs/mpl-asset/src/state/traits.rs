use crate::{error::MplAssetError, state::Key};
use borsh::{BorshDeserialize, BorshSerialize};
use num_traits::FromPrimitive;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;

pub trait DataBlob: BorshSerialize + BorshDeserialize {
    fn get_initial_size() -> usize;
    fn get_size(&self) -> usize;
}

pub trait SolanaAccount: BorshSerialize + BorshDeserialize {
    fn key() -> Key;

    fn load(account: &AccountInfo, offset: usize) -> Result<Self, ProgramError> {
        let key = load_key(account, offset)?;

        if key != Self::key() {
            return Err(MplAssetError::DeserializationError.into());
        }

        let mut bytes: &[u8] = &(*account.data).borrow()[offset..];
        Self::deserialize(&mut bytes).map_err(|error| {
            msg!("Error: {}", error);
            MplAssetError::DeserializationError.into()
        })
    }

    fn save(&self, account: &AccountInfo, offset: usize) -> ProgramResult {
        borsh::to_writer(&mut account.data.borrow_mut()[offset..], self).map_err(|error| {
            msg!("Error: {}", error);
            MplAssetError::SerializationError.into()
        })
    }
}

pub fn load_key(account: &AccountInfo, offset: usize) -> Result<Key, ProgramError> {
    let key = Key::from_u8((*account.data).borrow()[offset])
        .ok_or(MplAssetError::DeserializationError)?;

    Ok(key)
}
