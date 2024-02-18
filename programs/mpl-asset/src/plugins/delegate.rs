use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Delegate {
    frozen: bool,
}
