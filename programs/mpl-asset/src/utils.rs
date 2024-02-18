use crate::{error::MplAssetError, state::Key};
use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_error::ProgramError,
};

pub fn load_key(account: &AccountInfo, offset: usize) -> Result<Key, ProgramError> {
    let key = Key::from_u8((*account.data).borrow()[offset])
        .ok_or(MplAssetError::DeserializationError)?;

    Ok(key)
}
