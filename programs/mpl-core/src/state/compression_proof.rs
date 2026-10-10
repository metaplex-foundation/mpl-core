use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::state::{AssetV1, HashablePluginSchema, UpdateAuthority, Wrappable};

/// A simple struct to store the compression proof of an asset.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CompressionProof {
    /// The owner of the asset.
    pub owner: Pubkey, //32
    /// The update authority of the asset.
    pub update_authority: UpdateAuthority, //33
    /// The name of the asset.
    pub name: String, //4
    /// The URI of the asset that points to the off-chain data.
    pub uri: String, //4
    /// The sequence number used for indexing with compression.
    pub seq: u64, //8
    /// The plugins for the asset.
    pub plugins: Vec<HashablePluginSchema>, //4
}

impl CompressionProof {
    /// Create a new `CompressionProof`.  Note this uses a passed-in `seq` rather than
    /// the one contained in `asset` to avoid errors.
    pub fn new(asset: AssetV1, seq: u64, plugins: Vec<HashablePluginSchema>) -> Self {
        Self {
            owner: asset.owner,
            update_authority: asset.update_authority,
            name: asset.name,
            uri: asset.uri,
            seq,
            plugins,
        }
    }
}

impl Wrappable for CompressionProof {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        plugins::{Attributes, Plugin},
        state::{Authority, Key},
    };

    fn asset(seq: Option<u64>) -> AssetV1 {
        AssetV1 {
            key: Key::AssetV1,
            owner: Pubkey::new_unique(),
            update_authority: UpdateAuthority::Address(Pubkey::new_unique()),
            name: "name".to_string(),
            uri: "uri".to_string(),
            seq,
        }
    }

    fn plugins() -> Vec<HashablePluginSchema> {
        vec![HashablePluginSchema {
            index: 0,
            authority: Authority::UpdateAuthority,
            plugin: Plugin::Attributes(Attributes::new()),
        }]
    }

    /// `new` copies the asset's fields but takes `seq` from its argument, not
    /// from `asset.seq`.
    #[test]
    fn new_uses_the_passed_in_seq() {
        let asset = asset(Some(1));
        let proof = CompressionProof::new(asset.clone(), 7, plugins());

        assert_eq!(proof.owner, asset.owner);
        assert_eq!(proof.update_authority, asset.update_authority);
        assert_eq!(proof.name, asset.name);
        assert_eq!(proof.uri, asset.uri);
        assert_eq!(proof.seq, 7);
        assert_eq!(proof.plugins, plugins());
    }

    /// Round trip: rebuilding the asset from a proof restores every field and
    /// sets `seq` to `Some(proof.seq)`, which is how `decompress` and the
    /// hashed `burn`/`transfer` paths reconstruct account state.
    #[test]
    fn asset_from_proof_round_trips_with_seq_some() {
        let original = asset(None);
        let proof = CompressionProof::new(original.clone(), 7, plugins());
        let rebuilt = AssetV1::from(proof);

        assert_eq!(
            rebuilt,
            AssetV1 {
                seq: Some(7),
                ..original
            }
        );
        assert_eq!(rebuilt.key, Key::AssetV1);
    }

    /// The rebuilt asset keeps a `Collection` update authority intact.
    #[test]
    fn asset_from_proof_preserves_a_collection_update_authority() {
        let collection = Pubkey::new_unique();
        let mut original = asset(None);
        original.update_authority = UpdateAuthority::Collection(collection);

        let rebuilt = AssetV1::from(CompressionProof::new(original, 0, vec![]));

        assert_eq!(
            rebuilt.update_authority,
            UpdateAuthority::Collection(collection)
        );
        assert_eq!(rebuilt.seq, Some(0));
    }
}
