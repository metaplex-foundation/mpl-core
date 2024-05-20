#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::types::{
    FreezeDelegate, Plugin, PluginAuthority, PluginAuthorityPair, UpdateAuthority,
};
pub use setup::*;

use solana_program_test::tokio;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer};

#[tokio::test]
async fn create_asset_in_account_state() {
    let mut context = program_test().start_with_context().await;

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
            collection: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    let owner = context.payer.pubkey();
    let update_authority = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn create_asset_with_different_payer() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();
    let payer = Keypair::new();
    airdrop(&mut context, &payer.pubkey(), LAMPORTS_PER_SOL)
        .await
        .unwrap();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: Some(&payer),
            asset: &asset,
            data_state: None,
            name: None,
            uri: None,
            authority: None,
            update_authority: None,
            collection: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner: payer.pubkey(),
            update_authority: Some(UpdateAuthority::Address(payer.pubkey())),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn create_asset_with_plugins() {
    let mut context = program_test().start_with_context().await;

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
            collection: None,
            plugins: vec![PluginAuthorityPair {
                plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                authority: None,
            }],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    let owner = context.payer.pubkey();
    let update_authority = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
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
async fn create_asset_with_different_update_authority() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();
    let update_authority = Keypair::new();
    airdrop(&mut context, &update_authority.pubkey(), LAMPORTS_PER_SOL)
        .await
        .unwrap();
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
            update_authority: Some(update_authority.pubkey()),
            collection: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    let owner = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority.pubkey())),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn create_asset_with_plugins_with_different_update_authority() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();
    let update_authority = Keypair::new();
    airdrop(&mut context, &update_authority.pubkey(), LAMPORTS_PER_SOL)
        .await
        .unwrap();
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
            update_authority: Some(update_authority.pubkey()),
            collection: None,
            plugins: vec![PluginAuthorityPair {
                plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                authority: None,
            }],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    let owner = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority.pubkey())),
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
