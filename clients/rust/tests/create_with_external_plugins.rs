#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    errors::MplCoreError,
    types::{
        DataStore, DataStoreInitInfo, ExternalCheckResult, ExternalPlugin, ExternalPluginInitInfo,
        ExternalPluginSchema, HookableLifecycleEvent, LifecycleHook, LifecycleHookInitInfo, Oracle,
        OracleInitInfo, PluginAuthority, UpdateAuthority, ValidationResultsOffset,
    },
};
pub use setup::*;

use solana_program::pubkey;
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

#[tokio::test]
#[ignore]
async fn test_create_lifecycle_hook() {
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
            external_plugins: vec![ExternalPlugin::LifecycleHook(LifecycleHook {
                hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                extra_accounts: None,
                data_authority: Some(PluginAuthority::UpdateAuthority),
                schema: ExternalPluginSchema::Binary,
            })],
        },
    )
    .await;
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_lifecycle_hook_with_duplicate_lifecycle_checks() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();
    let error = create_asset(
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
                    lifecycle_checks: vec![
                        (
                            HookableLifecycleEvent::Transfer,
                            ExternalCheckResult { flags: 1 },
                        ),
                        (
                            HookableLifecycleEvent::Transfer,
                            ExternalCheckResult { flags: 1 },
                        ),
                    ],
                    extra_accounts: None,
                    data_authority: Some(PluginAuthority::UpdateAuthority),
                    schema: None,
                },
            )],
        },
    )
    .await
    .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::DuplicateLifecycleChecks);
}

#[tokio::test]
async fn test_temporarily_cannot_create_lifecycle_hook() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();
    let error = create_asset(
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
    .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::NotAvailable);
}

#[tokio::test]
async fn test_create_oracle() {
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
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Transfer,
                    ExternalCheckResult { flags: 4 },
                )],
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
}

#[tokio::test]
async fn test_cannot_create_oracle_with_duplicate_lifecycle_checks() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();
    let error = create_asset(
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
                lifecycle_checks: vec![
                    (
                        HookableLifecycleEvent::Transfer,
                        ExternalCheckResult { flags: 4 },
                    ),
                    (
                        HookableLifecycleEvent::Transfer,
                        ExternalCheckResult { flags: 4 },
                    ),
                ],
                pda: None,
                results_offset: None,
            })],
        },
    )
    .await
    .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::DuplicateLifecycleChecks);
}

#[tokio::test]
#[ignore]
async fn test_create_data_store() {
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
}

#[tokio::test]
async fn test_temporarily_cannot_create_data_store() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();
    let error = create_asset(
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
    .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::NotAvailable);
}
