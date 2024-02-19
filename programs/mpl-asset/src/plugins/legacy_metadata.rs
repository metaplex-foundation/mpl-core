use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::state::DataBlob;

#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct LegacyMetadata {
    pub mint: Pubkey, // 32
}

impl DataBlob for LegacyMetadata {
    fn get_initial_size() -> usize {
        32
    }

    fn get_size(&self) -> usize {
        32
    }
}
