use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{keccak, program_error::ProgramError, pubkey::Pubkey};

use super::Compressible;

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub enum Key {
    Uninitialized,
    Asset,
    HashedAsset,
    Collection,
    HashedCollection,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct Asset {
    pub key: Key,                 //2
    pub update_authority: Pubkey, //32
    pub owner: Pubkey,            //32
    pub name: String,             //4
    pub uri: String,              //4
}

impl Compressible for Asset {
    fn hash(&self) -> Result<[u8; 32], ProgramError> {
        let serialized_data = self.try_to_vec()?;

        Ok(keccak::hash(serialized_data.as_slice()).to_bytes())
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct HashedAsset {
    pub key: Key,                    //2
    pub hash: [u8; 32],              //32
    pub watermark_slot: Option<u64>, //1 | 9
}
