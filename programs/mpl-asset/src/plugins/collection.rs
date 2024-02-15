use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct Collection {
    collection_address: Pubkey,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct CollectionData {
    pub key: Key,                 //2
    pub update_authority: Pubkey, //32
    pub owner: Pubkey,            //32
    pub name: String,             //4
    pub uri: String,              //4
}
