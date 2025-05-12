#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::types::{
    DataState, FreezeDelegate, Plugin, PluginAuthority, PluginAuthorityPair, UpdateAuthority,
};
pub use setup::*;

use solana_program_test::tokio;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer};

#[tokio::test]
async fn create_asset_in_collection() {
    let mut context = program_test().start_with_context().await;

    // Create a collection first
    let collection = Keypair::new();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection,
            update_authority: None,
            payer: None,
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    // Create an asset within that collection
    let asset = Keypair::new();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset,
            data_state: None,
            name: None,
            uri: None,
            authority: None,
            update_authority: None,
            collection: Some(collection.pubkey()),
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    // Verify the asset is properly created with collection as update authority
    let owner = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Collection(collection.pubkey())),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;

    let payer_pubkey = context.payer.pubkey();

    // Verify the collection size has been incremented
    assert_collection(
        &mut context,
        AssertCollectionHelperArgs {
            collection: collection.pubkey(),
            update_authority: payer_pubkey,
            name: None,
            uri: None,
            num_minted: 1,
            current_size: 1,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn create_multiple_assets_in_collection() {
    let mut context = program_test().start_with_context().await;

    // Create a collection first
    let collection = Keypair::new();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection,
            update_authority: None,
            payer: None,
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    // Create three assets in the collection
    let asset1 = Keypair::new();
    let asset2 = Keypair::new();
    let asset3 = Keypair::new();

    for asset in [&asset1, &asset2, &asset3] {
        create_asset(
            &mut context,
            CreateAssetHelperArgs {
                owner: None,
                payer: None,
                asset,
                data_state: None,
                name: None,
                uri: None,
                authority: None,
                update_authority: None,
                collection: Some(collection.pubkey()),
                plugins: vec![],
                external_plugin_adapters: vec![],
            },
        )
        .await
        .unwrap();
    }

    let payer_pubkey = context.payer.pubkey();

    // Verify the collection size has been incremented to 3
    assert_collection(
        &mut context,
        AssertCollectionHelperArgs {
            collection: collection.pubkey(),
            update_authority: payer_pubkey,
            name: None,
            uri: None,
            num_minted: 3,
            current_size: 3,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn create_asset_in_collection_with_plugins() {
    let mut context = program_test().start_with_context().await;

    // Create a collection first with a freeze delegate plugin
    let collection = Keypair::new();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection,
            update_authority: None,
            payer: None,
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    // Create an asset within that collection with its own freeze delegate
    let asset = Keypair::new();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset,
            data_state: None,
            name: None,
            uri: None,
            authority: None,
            update_authority: None,
            collection: Some(collection.pubkey()),
            plugins: vec![PluginAuthorityPair {
                plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                authority: None,
            }],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    // Verify the asset has proper plugins
    let owner = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Collection(collection.pubkey())),
            name: None,
            uri: None,
            plugins: vec![PluginAuthorityPair {
                plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                authority: Some(PluginAuthority::Owner),
            }],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn create_asset_in_collection_with_custom_owner() {
    let mut context = program_test().start_with_context().await;

    // Create a collection first
    let collection = Keypair::new();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection,
            update_authority: None,
            payer: None,
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    // Create an asset with a different owner
    let asset = Keypair::new();
    let owner = Keypair::new();
    airdrop(&mut context, &owner.pubkey(), LAMPORTS_PER_SOL)
        .await
        .unwrap();

    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: Some(owner.pubkey()),
            payer: None,
            asset: &asset,
            data_state: None,
            name: None,
            uri: None,
            authority: None,
            update_authority: None,
            collection: Some(collection.pubkey()),
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    // Verify the asset is created with correct owner but collection as update authority
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner: owner.pubkey(),
            update_authority: Some(UpdateAuthority::Collection(collection.pubkey())),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}
