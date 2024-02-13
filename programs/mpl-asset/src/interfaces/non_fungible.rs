use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use super::{DataStorage, Interface};

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct NonFungible {
    pub interface: Interface,
    pub update_authority: Pubkey,
    pub owner: Pubkey,
    pub name: String,
    pub uri: String,
}

impl DataStorage for NonFungible {
    fn get_required_length(&self) {
        todo!()
    }

    fn save(&self, data: &mut [u8]) {
        todo!()
    }

    fn load(&self, data: &[u8]) {
        todo!()
    }

    fn load_mut(&self, data: &mut [u8]) {
        todo!()
    }
}
