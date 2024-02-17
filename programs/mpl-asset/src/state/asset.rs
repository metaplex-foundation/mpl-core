use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{keccak, program_error::ProgramError, pubkey::Pubkey};

use crate::{state::Key, utils::DataBlob};

use super::Compressible;

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct Asset {
    pub key: Key,                 //1
    pub update_authority: Pubkey, //32
    pub owner: Pubkey,            //32
    pub name: String,             //4
    pub uri: String,              //4
}

impl Asset {
    pub const BASE_LENGTH: usize = 1 + 32 + 32 + 4 + 4;
}

impl Compressible for Asset {
    fn hash(&self) -> Result<[u8; 32], ProgramError> {
        let serialized_data = self.try_to_vec()?;

        Ok(keccak::hash(serialized_data.as_slice()).to_bytes())
    }
}

impl DataBlob for Asset {
    fn get_initial_size() -> usize {
        Asset::BASE_LENGTH
    }

    fn get_size(&self) -> usize {
        Asset::BASE_LENGTH + self.name.len() + self.uri.len()
    }

    fn key() -> Key {
        Key::Asset
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct HashedAsset {
    pub key: Key,       //1
    pub hash: [u8; 32], //32
}
