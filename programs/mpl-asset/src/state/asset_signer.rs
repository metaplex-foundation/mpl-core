use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct AssetSigner {
    pub key: Key, //1
}

impl DataBlob for AssetSigner {
    fn get_initial_size() -> usize {
        1
    }

    fn get_size(&self) -> usize {
        1
    }
}

impl SolanaAccount for AssetSigner {
    fn key() -> Key {
        Key::AssetSigner
    }
}

impl Default for AssetSigner {
    fn default() -> Self {
        AssetSigner {
            key: Key::AssetSigner,
        }
    }
}