use solana_program::{program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar};

use crate::error::MplCoreError;

pub(crate) const COLLECT_RECIPIENT1: Pubkey =
    solana_program::pubkey!("8AT6o8Qk5T9QnZvPThMrF9bcCQLTGkyGvVZZzHgCw11v");

pub(crate) const COLLECT_RECIPIENT2: Pubkey =
    solana_program::pubkey!("MmHsqX4LxTfifxoH8BVRLUKrwDn1LPCac6YcCZTHhwt");

const CREATE_FEE_SCALAR: usize = 87;
const CREATE_FEE_OFFSET: u64 = 3600;
pub fn get_create_fee() -> Result<u64, ProgramError> {
    let rent = Rent::get()?.minimum_balance(CREATE_FEE_SCALAR);

    Ok(rent
        .checked_add(CREATE_FEE_OFFSET)
        .ok_or(MplCoreError::NumericalOverflowError)?)
}
