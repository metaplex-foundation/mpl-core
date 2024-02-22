use crate::{error::MplAssetError, state::Key, utils::load_key};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, keccak, msg, program::invoke,
    program_error::ProgramError,
};

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

pub trait Compressible: BorshSerialize + BorshDeserialize {
    fn hash(&self) -> Result<[u8; 32], ProgramError> {
        let serialized_data = self.try_to_vec()?;
        Ok(keccak::hash(serialized_data.as_slice()).to_bytes())
    }

    fn wrap(&self) -> ProgramResult {
        let serialized_data = self.try_to_vec()?;
        invoke(&spl_noop::instruction(serialized_data), &[])
    }
}
