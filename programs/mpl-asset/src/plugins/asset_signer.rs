use borsh::{BorshDeserialize, BorshSerialize};

use crate::state::DataBlob;

#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct AssetSigner {}

impl DataBlob for AssetSigner {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}
