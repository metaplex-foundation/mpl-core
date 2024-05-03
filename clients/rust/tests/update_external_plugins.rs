#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    instructions::UpdateExternalPluginV1Builder,
    types::{
        DataStore, DataStoreInitInfo, DataStoreUpdateInfo, ExternalCheckResult, ExternalPlugin,
        ExternalPluginInitInfo, ExternalPluginKey, ExternalPluginSchema, ExternalPluginUpdateInfo,
        HookableLifecycleEvent, LifecycleHook, LifecycleHookInitInfo, LifecycleHookUpdateInfo,
        Oracle, OracleInitInfo, OracleUpdateInfo, PluginAuthority, UpdateAuthority,
        ValidationResultsOffset,
    },
};
pub use setup::*;

use solana_program::pubkey;
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

#[tokio::test]
async fn test_update_lifecycle_hook() {
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
            external_plugins: vec![ExternalPluginInitInfo::LifecycleHook(
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
            )],
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
            external_plugins: vec![ExternalPlugin::LifecycleHook(LifecycleHook {
                hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                extra_accounts: None,
                data_authority: Some(PluginAuthority::UpdateAuthority),
                schema: ExternalPluginSchema::Binary,
            })],
        },
    )
    .await;

    let ix = UpdateExternalPluginV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginKey::LifecycleHook(pubkey!(
            "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"
        )))
        .update_info(ExternalPluginUpdateInfo::LifecycleHook(
            LifecycleHookUpdateInfo {
                lifecycle_checks: None,
                extra_accounts: None,
                schema: Some(ExternalPluginSchema::Json),
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
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
                schema: ExternalPluginSchema::Json,
            })],
        },
    )
    .await;
}

#[tokio::test]
async fn test_update_oracle() {
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
            external_plugins: vec![ExternalPluginInitInfo::Oracle(OracleInitInfo {
                base_address: Pubkey::default(),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: Some(vec![(
                    HookableLifecycleEvent::Transfer,
                    ExternalCheckResult { flags: 2 },
                )]),
                pda: None,
                results_offset: None,
            })],
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
            external_plugins: vec![ExternalPlugin::Oracle(Oracle {
                base_address: Pubkey::default(),
                pda: None,
                results_offset: ValidationResultsOffset::NoOffset,
            })],
        },
    )
    .await;

    let ix = UpdateExternalPluginV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginKey::Oracle(Pubkey::default()))
        .update_info(ExternalPluginUpdateInfo::Oracle(OracleUpdateInfo {
            lifecycle_checks: None,
            pda: None,
            results_offset: Some(ValidationResultsOffset::Custom(10)),
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
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
                results_offset: ValidationResultsOffset::Custom(10),
            })],
        },
    )
    .await;
}

#[tokio::test]
async fn test_update_data_store() {
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
            external_plugins: vec![ExternalPluginInitInfo::DataStore(DataStoreInitInfo {
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                data_authority: PluginAuthority::UpdateAuthority,
                schema: None,
            })],
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
            external_plugins: vec![ExternalPlugin::DataStore(DataStore {
                data_authority: PluginAuthority::UpdateAuthority,
                schema: ExternalPluginSchema::Binary,
            })],
        },
    )
    .await;

    let ix = UpdateExternalPluginV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginKey::DataStore(
            PluginAuthority::UpdateAuthority,
        ))
        .update_info(ExternalPluginUpdateInfo::DataStore(DataStoreUpdateInfo {
            schema: Some(ExternalPluginSchema::Json),
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
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
                schema: ExternalPluginSchema::Json,
            })],
        },
    )
    .await;
}
