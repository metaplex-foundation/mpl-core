use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    state::Key,
    state::{DataBlob, SolanaAccount},
};

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Delegate {
    pub key: Key,     // 1
    pub frozen: bool, // 1
}

impl Delegate {
    pub fn new() -> Self {
        Self {
            key: Key::Delegate,
            frozen: false,
        }
    }
}

impl Default for Delegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for Delegate {
    fn get_initial_size() -> usize {
        2
    }

    fn get_size(&self) -> usize {
        2
    }
}

impl SolanaAccount for Delegate {
    fn key() -> crate::state::Key {
        Key::Delegate
    }
}