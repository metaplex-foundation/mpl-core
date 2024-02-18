use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, keccak, program::invoke,
    program_error::ProgramError, pubkey::Pubkey,
};

use crate::{
    error::MplAssetError,
    state::{Compressible, CompressionProof, DataBlob, HashedAsset, Key, SolanaAccount},
};

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

    // Check that a compression proof results in same on-chain hash.
    pub fn verify_proof(
        hashed_asset: &AccountInfo,
        compression_proof: CompressionProof,
    ) -> Result<Asset, ProgramError> {
        let asset = Self::from(compression_proof);
        let asset_hash = asset.hash()?;
        let current_account_hash = HashedAsset::load(hashed_asset, 0)?.hash;
        if asset_hash != current_account_hash {
            return Err(MplAssetError::IncorrectAssetHash.into());
        }

        Ok(asset)
    }
}

impl Compressible for Asset {
    fn hash(&self) -> Result<[u8; 32], ProgramError> {
        let serialized_data = self.try_to_vec()?;
        Ok(keccak::hash(serialized_data.as_slice()).to_bytes())
    }

    fn wrap(&self) -> ProgramResult {
        let serialized_data = self.try_to_vec()?;
        invoke(&spl_noop::instruction(serialized_data), &[])
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
