#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::types::{Creator, Plugin, PluginAuthority, PluginAuthorityPair, Royalties, RuleSet};
pub use setup::*;

use solana_program_test::tokio;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_create_collection() {
    let mut context = program_test().start_with_context().await;

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

    let update_authority = context.payer.pubkey();
    assert_collection(
        &mut context,
        AssertCollectionHelperArgs {
            collection: collection.pubkey(),
            update_authority,
            name: None,
            uri: None,
            num_minted: 0,
            current_size: 0,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn create_collection_with_different_payer() {
    let mut context = program_test().start_with_context().await;

    let collection = Keypair::new();
    let payer = Keypair::new();
    airdrop(&mut context, &payer.pubkey(), LAMPORTS_PER_SOL)
        .await
        .unwrap();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection,
            update_authority: None,
            payer: Some(&payer),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    assert_collection(
        &mut context,
        AssertCollectionHelperArgs {
            collection: collection.pubkey(),
            update_authority: payer.pubkey(),
            name: None,
            uri: None,
            num_minted: 0,
            current_size: 0,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn create_collection_with_plugins() {
    let mut context = program_test().start_with_context().await;

    let collection = Keypair::new();
    let update_authority = context.payer.pubkey();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection,
            update_authority: None,
            payer: None,
            name: None,
            uri: None,
            plugins: vec![PluginAuthorityPair {
                authority: None,
                plugin: Plugin::Royalties(Royalties {
                    basis_points: 500,
                    creators: vec![Creator {
                        address: update_authority,
                        percentage: 100,
                    }],
                    rule_set: RuleSet::ProgramDenyList(vec![]),
                }),
            }],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    assert_collection(
        &mut context,
        AssertCollectionHelperArgs {
            collection: collection.pubkey(),
            update_authority,
            name: None,
            uri: None,
            num_minted: 0,
            current_size: 0,
            plugins: vec![PluginAuthorityPair {
                authority: None,
                plugin: Plugin::Royalties(Royalties {
                    basis_points: 500,
                    creators: vec![Creator {
                        address: update_authority,
                        percentage: 100,
                    }],
                    rule_set: RuleSet::ProgramDenyList(vec![]),
                }),
            }],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn create_collection_with_different_update_authority() {
    let mut context = program_test().start_with_context().await;

    let collection = Keypair::new();
    let payer = Keypair::new();
    airdrop(&mut context, &payer.pubkey(), LAMPORTS_PER_SOL)
        .await
        .unwrap();
    let update_authority = Keypair::new();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection,
            update_authority: Some(update_authority.pubkey()),
            payer: Some(&payer),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    assert_collection(
        &mut context,
        AssertCollectionHelperArgs {
            collection: collection.pubkey(),
            update_authority: update_authority.pubkey(),
            name: None,
            uri: None,
            num_minted: 0,
            current_size: 0,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn create_collection_with_plugins_with_different_plugin_authority() {
    let mut context = program_test().start_with_context().await;

    let collection = Keypair::new();
    let update_authority = context.payer.pubkey();
    let royalties_authority = Keypair::new();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection,
            update_authority: Some(update_authority),
            payer: None,
            name: None,
            uri: None,
            plugins: vec![PluginAuthorityPair {
                authority: Some(PluginAuthority::Address {
                    address: royalties_authority.pubkey(),
                }),
                plugin: Plugin::Royalties(Royalties {
                    basis_points: 500,
                    creators: vec![Creator {
                        address: update_authority,
                        percentage: 100,
                    }],
                    rule_set: RuleSet::ProgramDenyList(vec![]),
                }),
            }],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    assert_collection(
        &mut context,
        AssertCollectionHelperArgs {
            collection: collection.pubkey(),
            update_authority,
            name: None,
            uri: None,
            num_minted: 0,
            current_size: 0,
            plugins: vec![PluginAuthorityPair {
                authority: Some(PluginAuthority::Address {
                    address: royalties_authority.pubkey(),
                }),
                plugin: Plugin::Royalties(Royalties {
                    basis_points: 500,
                    creators: vec![Creator {
                        address: update_authority,
                        percentage: 100,
                    }],
                    rule_set: RuleSet::ProgramDenyList(vec![]),
                }),
            }],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}
