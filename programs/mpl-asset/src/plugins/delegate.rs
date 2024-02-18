use borsh::{BorshDeserialize, BorshSerialize};

use crate::{state::Key, utils::DataBlob};

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Delegate {
    key: Key,     // 1
    frozen: bool, // 1
}

impl DataBlob for Delegate {
    fn get_initial_size() -> usize {
        2
    }

    fn get_size(&self) -> usize {
        2
    }

    fn key() -> crate::state::Key {
        Key::Delegate
    }
}
