use borsh::{BorshDeserialize, BorshSerialize};

use crate::state::DataBlob;

#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Delegate {
    pub frozen: bool, // 1
}

impl Delegate {
    pub fn new() -> Self {
        Self { frozen: false }
    }
}

impl Default for Delegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for Delegate {
    fn get_initial_size() -> usize {
        1
    }

    fn get_size(&self) -> usize {
        1
    }
}
