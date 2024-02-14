use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct Creator {
    address: Pubkey,
    verified: bool,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct Royalties {
    creators: Vec<Creator>,
    auth_rules: Pubkey,
    seller_fee_basis_points: u16,
}
