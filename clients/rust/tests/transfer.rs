#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    instructions::TransferV1Builder,
    types::{Creator, Plugin, PluginAuthorityPair, Royalties, RuleSet, UpdateAuthority},
};
pub use setup::*;

use solana_program_test::tokio;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

#[tokio::test]
async fn transfer_asset_as_owner() {
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

    let new_owner = Keypair::new();
    let transfer_ix = TransferV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .new_owner(new_owner.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    let update_authority = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner: new_owner.pubkey(),
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
async fn transfer_asset_with_royalties() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();
    let update_authority = context.payer.pubkey();
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

    let new_owner = Keypair::new();
    let transfer_ix = TransferV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .new_owner(new_owner.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner: new_owner.pubkey(),
            update_authority: Some(UpdateAuthority::Address(update_authority)),
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
    .await;
}
