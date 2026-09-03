use solana_program::{program_error::ProgramError, pubkey::Pubkey};

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

/// Flat fee charged on execute: 0.00004872 SOL.
///
/// Like `CREATE_FEE`, this is a fixed lamport amount so the fee is
/// independent of the network rent rate.
pub const EXECUTE_FEE: u64 = 48_720;

pub fn get_execute_fee() -> Result<u64, ProgramError> {
    Ok(EXECUTE_FEE)
}
