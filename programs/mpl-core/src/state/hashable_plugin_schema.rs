use borsh::{BorshDeserialize, BorshSerialize};
use std::cmp::Ordering;

use crate::{
    plugins::Plugin,
    state::{Authority, Compressible},
};

/// A type that stores a plugin's authority and deserialized data into a
/// schema that will be later hashed into a hashed asset.  Also used in
/// `CompressionProof`.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct HashablePluginSchema {
    /// This is the order the plugins are stored in the account, allowing us
    /// to keep track of their order in the hashing.
    pub index: usize,
    /// The authority who has permission to utilize a plugin.
    pub authority: Authority,
    /// The deserialized plugin.
    pub plugin: Plugin,
}

impl HashablePluginSchema {
    /// Associated function for sorting `RegistryRecords` by offset.
    pub fn compare_indeces(a: &HashablePluginSchema, b: &HashablePluginSchema) -> Ordering {
        a.index.cmp(&b.index)
    }
}

impl Compressible for HashablePluginSchema {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::{Attributes, FreezeDelegate};
    use solana_program::pubkey::Pubkey;

    fn schema(index: usize, plugin: Plugin) -> HashablePluginSchema {
        HashablePluginSchema {
            index,
            authority: Authority::UpdateAuthority,
            plugin,
        }
    }

    #[test]
    fn compare_indeces_orders_by_index() {
        let low = schema(0, Plugin::FreezeDelegate(FreezeDelegate { frozen: false }));
        let high = schema(1, Plugin::Attributes(Attributes::new()));

        assert_eq!(
            HashablePluginSchema::compare_indeces(&low, &high),
            Ordering::Less
        );
        assert_eq!(
            HashablePluginSchema::compare_indeces(&high, &low),
            Ordering::Greater
        );
        assert_eq!(
            HashablePluginSchema::compare_indeces(&low, &low.clone()),
            Ordering::Equal
        );
    }

    /// `verify_proof` sorts the proof's plugins with this comparator before
    /// hashing them, so an out-of-order proof still hashes to the stored value.
    #[test]
    fn compare_indeces_sorts_an_unordered_vector() {
        let mut plugins = vec![
            schema(1, Plugin::Attributes(Attributes::new())),
            schema(0, Plugin::FreezeDelegate(FreezeDelegate { frozen: false })),
        ];
        plugins.sort_by(HashablePluginSchema::compare_indeces);

        assert_eq!(
            plugins.iter().map(|p| p.index).collect::<Vec<_>>(),
            vec![0, 1]
        );
    }

    /// The authority is part of the hashed schema, so two records that differ
    /// only in authority hash differently.
    #[test]
    fn hash_covers_the_authority() {
        let base = schema(0, Plugin::Attributes(Attributes::new()));
        let mut delegated = base.clone();
        delegated.authority = Authority::Address {
            address: Pubkey::new_unique(),
        };

        assert_ne!(base.hash().unwrap(), delegated.hash().unwrap());
    }
}
