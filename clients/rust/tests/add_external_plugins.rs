#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    instructions::AddExternalPluginV1Builder,
    types::{
        DataStore, DataStoreInitInfo, ExternalCheckResult, ExternalPlugin, ExternalPluginInitInfo,
        ExternalPluginSchema, HookableLifecycleEvent, LifecycleHook, LifecycleHookInitInfo, Oracle,
        OracleInitInfo, PluginAuthority, UpdateAuthority,
    },
};
pub use setup::*;

use solana_program::pubkey;
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

#[tokio::test]
async fn test_add_lifecycle_hook() {
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
            external_plugins: vec![],
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
            external_plugins: vec![],
        },
    )
    .await;

    let add_external_plugin_ix = AddExternalPluginV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(ExternalPluginInitInfo::LifecycleHook(
            LifecycleHookInitInfo {
                hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: Some(vec![(
                    HookableLifecycleEvent::Transfer,
                    ExternalCheckResult { flags: 1 },
                )]),
                extra_accounts: None,
                data_authority: Some(PluginAuthority::UpdateAuthority),
                schema: None,
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_external_plugin_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugins: vec![ExternalPlugin::LifecycleHook(LifecycleHook {
                hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                extra_accounts: None,
                data_authority: Some(PluginAuthority::UpdateAuthority),
                schema: ExternalPluginSchema::Binary,
                data_offset: 119,
                data_len: 0,
            })],
        },
    )
    .await;
}

#[tokio::test]
async fn test_add_oracle() {
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
            external_plugins: vec![],
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
            external_plugins: vec![],
        },
    )
    .await;

    let add_external_plugin_ix = AddExternalPluginV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(ExternalPluginInitInfo::Oracle(OracleInitInfo {
            base_address: Pubkey::default(),
            init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
            lifecycle_checks: Some(vec![(
                HookableLifecycleEvent::Transfer,
                ExternalCheckResult { flags: 1 },
            )]),
            pda: None,
            results_offset: None,
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_external_plugin_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugins: vec![ExternalPlugin::Oracle(Oracle {
                base_address: Pubkey::default(),
                pda: None,
            })],
        },
    )
    .await;
}

#[tokio::test]
async fn test_add_data_store() {
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
            external_plugins: vec![],
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
            external_plugins: vec![],
        },
    )
    .await;

    let add_external_plugin_ix = AddExternalPluginV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(ExternalPluginInitInfo::DataStore(DataStoreInitInfo {
            init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
            data_authority: PluginAuthority::UpdateAuthority,
            schema: None,
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_external_plugin_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugins: vec![ExternalPlugin::DataStore(DataStore {
                data_authority: PluginAuthority::UpdateAuthority,
                schema: ExternalPluginSchema::Binary,
                data_offset: 119,
                data_len: 0,
            })],
        },
    )
    .await;
}
