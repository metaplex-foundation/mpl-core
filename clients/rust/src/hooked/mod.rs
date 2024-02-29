pub mod plugins;

pub use plugins::*;

use crate::accounts::{Asset, CollectionData};

impl Asset {
    /// The base length of the asset account with an empty name and uri.
    pub const BASE_LENGTH: usize = 1 + 32 + 33 + 4 + 4;

    pub fn get_size(&self) -> usize {
        Asset::BASE_LENGTH + self.name.len() + self.uri.len()
    }
}

impl CollectionData {
    /// The base length of the collection account with an empty name and uri.
    pub const BASE_LENGTH: usize = 1 + 32 + 4 + 4 + 4 + 4;

    fn get_size(&self) -> usize {
        Self::BASE_LENGTH + self.name.len() + self.uri.len()
    }
}
