use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    plugins::{
        create_meta_idempotent, initialize_plugin, Plugin, PluginRegistryV1, RegistryRecord,
    },
    state::{
        AssetV1, Compressible, CompressionProof, HashablePluginSchema, HashedAssetSchema,
        HashedAssetV1, SolanaAccount,
    },
};

use super::resize_or_reallocate_account;

/// Take an `Asset` and Vec of `HashablePluginSchema` and rebuild the asset in account space.
pub(crate) fn rebuild_account_state_from_proof_data<'a>(
    asset: AssetV1,
    plugins: Vec<HashablePluginSchema>,
    asset_info: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let serialized_data = asset.try_to_vec()?;
    resize_or_reallocate_account(asset_info, payer, system_program, serialized_data.len())?;

    sol_memcpy(
        &mut asset_info.try_borrow_mut_data()?,
        &serialized_data,
        serialized_data.len(),
    );

    // Add the plugins.
    if !plugins.is_empty() {
        let (_, header_offset, mut plugin_header, mut plugin_registry) =
            create_meta_idempotent::<AssetV1>(asset_info, payer, system_program)?;

        for plugin in plugins {
            initialize_plugin::<AssetV1>(
                &plugin.plugin,
                &plugin.authority,
                header_offset,
                &mut plugin_header,
                &mut plugin_registry,
                asset_info,
                payer,
                system_program,
            )?;
        }
    }

    Ok(())
}

/// Take `Asset` and `PluginRegistry` for a decompressed asset, and compress into account space.
pub(crate) fn compress_into_account_space<'a>(
    mut asset: AssetV1,
    plugin_registry: Option<PluginRegistryV1>,
    asset_info: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> Result<CompressionProof, ProgramError> {
    // Initialize or increment the sequence number when compressing.
    let seq = asset.seq.unwrap_or(0).saturating_add(1);
    asset.seq = Some(seq);

    let asset_hash = asset.hash()?;
    let mut compression_proof = CompressionProof::new(asset, seq, vec![]);
    let mut plugin_hashes = vec![];
    if let Some(plugin_registry) = plugin_registry {
        let mut registry_records = plugin_registry.registry;

        // It should already be sorted but we just want to make sure.
        registry_records.sort_by(RegistryRecord::compare_offsets);

        for (i, record) in registry_records.into_iter().enumerate() {
            let plugin = Plugin::deserialize(&mut &(*asset_info.data).borrow()[record.offset..])?;

            let hashable_plugin_schema = HashablePluginSchema {
                index: i,
                authority: record.authority,
                plugin,
            };

            let plugin_hash = hashable_plugin_schema.hash()?;
            plugin_hashes.push(plugin_hash);

            compression_proof.plugins.push(hashable_plugin_schema);
        }
    }

    let hashed_asset_schema = HashedAssetSchema {
        asset_hash,
        plugin_hashes,
    };

    let hashed_asset = HashedAssetV1::new(hashed_asset_schema.hash()?);
    let serialized_data = hashed_asset.try_to_vec()?;

    resize_or_reallocate_account(asset_info, payer, system_program, serialized_data.len())?;

    sol_memcpy(
        &mut asset_info.try_borrow_mut_data()?,
        &serialized_data,
        serialized_data.len(),
    );

    Ok(compression_proof)
}

/// Check that a compression proof results in same on-chain hash.
pub(crate) fn verify_proof(
    hashed_asset: &AccountInfo,
    compression_proof: &CompressionProof,
) -> Result<(AssetV1, Vec<HashablePluginSchema>), ProgramError> {
    let asset = AssetV1::from(compression_proof.clone());
    let asset_hash = asset.hash()?;

    let mut sorted_plugins = compression_proof.plugins.clone();
    sorted_plugins.sort_by(HashablePluginSchema::compare_indeces);

    let plugin_hashes = sorted_plugins
        .iter()
        .map(|plugin| plugin.hash())
        .collect::<Result<Vec<[u8; 32]>, ProgramError>>()?;

    let hashed_asset_schema = HashedAssetSchema {
        asset_hash,
        plugin_hashes,
    };

    let hashed_asset_schema_hash = hashed_asset_schema.hash()?;

    let current_account_hash = HashedAssetV1::load(hashed_asset, 0)?.hash;
    if hashed_asset_schema_hash != current_account_hash {
        return Err(MplCoreError::IncorrectAssetHash.into());
    }

    Ok((asset, sorted_plugins))
}
