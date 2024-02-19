use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Creator {
    address: Pubkey,
    verified: bool,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub enum RuleSet {
    ProgramAllowList(Vec<Pubkey>),
    ProgramDenyList(Vec<Pubkey>),
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct Royalties {
    seller_fee_basis_points: u16,
    creators: Vec<Creator>,
    rule_set: RuleSet,
}
