use solana_program::{program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar};

pub(crate) const COLLECT_RECIPIENT1: Pubkey =
    solana_program::pubkey!("8AT6o8Qk5T9QnZvPThMrF9bcCQLTGkyGvVZZzHgCw11v");

pub(crate) const COLLECT_RECIPIENT2: Pubkey =
    solana_program::pubkey!("MmHsqX4LxTfifxoH8BVRLUKrwDn1LPCac6YcCZTHhwt");

/// Flat fee charged on asset creation: 0.0015 SOL.
///
/// This is intentionally a fixed lamport amount rather than a value derived
/// from the rent sysvar so that changes to the network rent rate do not alter
/// the protocol fee.
pub const CREATE_FEE: u64 = 1_500_000;

pub fn get_create_fee() -> Result<u64, ProgramError> {
    Ok(CREATE_FEE)
}

const EXECUTE_FEE_SCALAR: usize = 7;
pub fn get_execute_fee() -> Result<u64, ProgramError> {
    Ok(Rent::get()?.minimum_balance(EXECUTE_FEE_SCALAR) - Rent::get()?.minimum_balance(0))
}
