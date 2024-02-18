use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{keccak, program_error::ProgramError, pubkey::Pubkey};

use crate::state::{CompressionProof, DataBlob, Key, SolanaAccount};

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
}

impl SolanaAccount for Asset {
    fn key() -> Key {
        Key::Asset
    }
}

impl From<CompressionProof> for Asset {
    fn from(compression_proof: CompressionProof) -> Self {
        Self {
            key: Self::key(),
            update_authority: compression_proof.update_authority,
            owner: compression_proof.owner,
            name: compression_proof.name,
            uri: compression_proof.uri,
        }
    }
}
