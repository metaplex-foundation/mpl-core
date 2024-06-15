#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    errors::MplCoreError,
    instructions::UpdateExternalPluginAdapterV1Builder,
    types::{
        AppData, AppDataInitInfo, AppDataUpdateInfo, ExternalCheckResult, ExternalPluginAdapter,
        ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey, ExternalPluginAdapterSchema,
        ExternalPluginAdapterUpdateInfo, HookableLifecycleEvent, LifecycleHook,
        LifecycleHookInitInfo, LifecycleHookUpdateInfo, Oracle, OracleInitInfo, OracleUpdateInfo,
        PluginAuthority, UpdateAuthority, ValidationResultsOffset,
    },
};
pub use setup::*;

use solana_program::pubkey;
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

#[tokio::test]
#[ignore]
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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::LifecycleHook(
                LifecycleHookInitInfo {
                    hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                    init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                    lifecycle_checks: vec![(
                        HookableLifecycleEvent::Transfer,
                        ExternalCheckResult { flags: 1 },
                    )],
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
            external_plugin_adapters: vec![ExternalPluginAdapter::LifecycleHook(LifecycleHook {
                hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                extra_accounts: None,
                data_authority: Some(PluginAuthority::UpdateAuthority),
                schema: ExternalPluginAdapterSchema::Binary,
            })],
        },
    )
    .await;

    let ix = UpdateExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::LifecycleHook(pubkey!(
            "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"
        )))
        .update_info(ExternalPluginAdapterUpdateInfo::LifecycleHook(
            LifecycleHookUpdateInfo {
                lifecycle_checks: None,
                extra_accounts: None,
                schema: Some(ExternalPluginAdapterSchema::Json),
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
            external_plugin_adapters: vec![ExternalPluginAdapter::LifecycleHook(LifecycleHook {
                hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                extra_accounts: None,
                data_authority: Some(PluginAuthority::UpdateAuthority),
                schema: ExternalPluginAdapterSchema::Json,
            })],
        },
    )
    .await;
}

#[tokio::test]
#[ignore]
async fn test_cannot_update_lifecycle_hook_to_have_duplicate_lifecycle_checks() {
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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::LifecycleHook(
                LifecycleHookInitInfo {
                    hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                    init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                    lifecycle_checks: vec![(
                        HookableLifecycleEvent::Transfer,
                        ExternalCheckResult { flags: 1 },
                    )],
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
            external_plugin_adapters: vec![ExternalPluginAdapter::LifecycleHook(LifecycleHook {
                hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                extra_accounts: None,
                data_authority: Some(PluginAuthority::UpdateAuthority),
                schema: ExternalPluginAdapterSchema::Binary,
            })],
        },
    )
    .await;

    let ix = UpdateExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::LifecycleHook(pubkey!(
            "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"
        )))
        .update_info(ExternalPluginAdapterUpdateInfo::LifecycleHook(
            LifecycleHookUpdateInfo {
                lifecycle_checks: Some(vec![
                    (
                        HookableLifecycleEvent::Transfer,
                        ExternalCheckResult { flags: 1 },
                    ),
                    (
                        HookableLifecycleEvent::Transfer,
                        ExternalCheckResult { flags: 1 },
                    ),
                ]),
                extra_accounts: None,
                schema: Some(ExternalPluginAdapterSchema::Json),
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::DuplicateLifecycleChecks);

    // TODO: Check that registry record did not get updated.
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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
                base_address: Pubkey::default(),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Transfer,
                    ExternalCheckResult { flags: 4 },
                )],
                base_address_config: None,
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
            external_plugin_adapters: vec![ExternalPluginAdapter::Oracle(Oracle {
                base_address: Pubkey::default(),
                base_address_config: None,
                results_offset: ValidationResultsOffset::NoOffset,
            })],
        },
    )
    .await;

    let ix = UpdateExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::Oracle(Pubkey::default()))
        .update_info(ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
            lifecycle_checks: None,
            base_address_config: None,
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
            external_plugin_adapters: vec![ExternalPluginAdapter::Oracle(Oracle {
                base_address: Pubkey::default(),
                base_address_config: None,
                results_offset: ValidationResultsOffset::Custom(10),
            })],
        },
    )
    .await;
}

#[tokio::test]
async fn test_cannot_update_oracle_to_have_duplicate_lifecycle_checks() {
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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
                base_address: Pubkey::default(),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Transfer,
                    ExternalCheckResult { flags: 4 },
                )],
                base_address_config: None,
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
            external_plugin_adapters: vec![ExternalPluginAdapter::Oracle(Oracle {
                base_address: Pubkey::default(),
                base_address_config: None,
                results_offset: ValidationResultsOffset::NoOffset,
            })],
        },
    )
    .await;

    let ix = UpdateExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::Oracle(Pubkey::default()))
        .update_info(ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
            lifecycle_checks: Some(vec![
                (
                    HookableLifecycleEvent::Transfer,
                    ExternalCheckResult { flags: 4 },
                ),
                (
                    HookableLifecycleEvent::Transfer,
                    ExternalCheckResult { flags: 4 },
                ),
            ]),
            base_address_config: None,
            results_offset: Some(ValidationResultsOffset::Custom(10)),
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::DuplicateLifecycleChecks);

    // TODO: Check that registry record did not get updated.
}

#[tokio::test]
async fn test_update_app_data() {
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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::AppData(
                AppDataInitInfo {
                    init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                    data_authority: PluginAuthority::UpdateAuthority,
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
            external_plugin_adapters: vec![ExternalPluginAdapter::AppData(AppData {
                data_authority: PluginAuthority::UpdateAuthority,
                schema: ExternalPluginAdapterSchema::Binary,
            })],
        },
    )
    .await;

    let ix = UpdateExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::AppData(
            PluginAuthority::UpdateAuthority,
        ))
        .update_info(ExternalPluginAdapterUpdateInfo::AppData(
            AppDataUpdateInfo {
                schema: Some(ExternalPluginAdapterSchema::Json),
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
            external_plugin_adapters: vec![ExternalPluginAdapter::AppData(AppData {
                data_authority: PluginAuthority::UpdateAuthority,
                schema: ExternalPluginAdapterSchema::Json,
            })],
        },
    )
    .await;
}
