#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    errors::MplCoreError,
    instructions::{AddCollectionPluginAdapterV1Builder, AddPluginAdapterV1Builder},
    types::{
        AdapterCheckResult, DataStore, DataStoreInitInfo, HookableLifecycleEvent, LifecycleHook,
        LifecycleHookInitInfo, Oracle, OracleInitInfo, PluginAdapter, PluginAdapterInitInfo,
        PluginAdapterSchema, PluginAuthority, UpdateAuthority, ValidationResultsOffset,
    },
};
pub use setup::*;

use solana_program::pubkey;
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

#[tokio::test]
#[ignore]
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
            plugin_adapters: vec![],
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
            plugin_adapters: vec![],
        },
    )
    .await;

    let add_plugin_adapter_ix = AddPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(PluginAdapterInitInfo::LifecycleHook(
            LifecycleHookInitInfo {
                hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Transfer,
                    AdapterCheckResult { flags: 1 },
                )],
                extra_accounts: None,
                data_authority: Some(PluginAuthority::UpdateAuthority),
                schema: None,
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_plugin_adapter_ix],
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
            plugin_adapters: vec![PluginAdapter::LifecycleHook(LifecycleHook {
                hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                extra_accounts: None,
                data_authority: Some(PluginAuthority::UpdateAuthority),
                schema: PluginAdapterSchema::Binary,
            })],
        },
    )
    .await;
}

#[tokio::test]
#[ignore]
async fn test_cannot_add_lifecycle_hook_with_duplicate_lifecycle_checks() {
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
            plugin_adapters: vec![],
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
            plugin_adapters: vec![],
        },
    )
    .await;

    let add_plugin_adapter_ix = AddPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(PluginAdapterInitInfo::LifecycleHook(
            LifecycleHookInitInfo {
                hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![
                    (
                        HookableLifecycleEvent::Transfer,
                        AdapterCheckResult { flags: 1 },
                    ),
                    (
                        HookableLifecycleEvent::Transfer,
                        AdapterCheckResult { flags: 1 },
                    ),
                ],
                extra_accounts: None,
                data_authority: Some(PluginAuthority::UpdateAuthority),
                schema: None,
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_plugin_adapter_ix],
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

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn test_temporarily_cannot_add_lifecycle_hook() {
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
            plugin_adapters: vec![],
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
            plugin_adapters: vec![],
        },
    )
    .await;

    let add_plugin_adapter_ix = AddPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(PluginAdapterInitInfo::LifecycleHook(
            LifecycleHookInitInfo {
                hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Transfer,
                    AdapterCheckResult { flags: 1 },
                )],
                extra_accounts: None,
                data_authority: Some(PluginAuthority::UpdateAuthority),
                schema: None,
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_plugin_adapter_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::NotAvailable);

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn test_temporarily_cannot_add_lifecycle_hook_on_collection() {
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
            plugin_adapters: vec![],
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
        },
    )
    .await;

    let add_plugin_adapter_ix = AddCollectionPluginAdapterV1Builder::new()
        .collection(collection.pubkey())
        .payer(context.payer.pubkey())
        .init_info(PluginAdapterInitInfo::LifecycleHook(
            LifecycleHookInitInfo {
                hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Transfer,
                    AdapterCheckResult { flags: 1 },
                )],
                extra_accounts: None,
                data_authority: Some(PluginAuthority::UpdateAuthority),
                schema: None,
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_plugin_adapter_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::NotAvailable);

    // TODO add collection assert.
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
            plugin_adapters: vec![],
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
            plugin_adapters: vec![],
        },
    )
    .await;

    let add_plugin_adapter_ix = AddPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(PluginAdapterInitInfo::Oracle(OracleInitInfo {
            base_address: Pubkey::default(),
            init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
            lifecycle_checks: vec![(
                HookableLifecycleEvent::Transfer,
                AdapterCheckResult { flags: 4 },
            )],
            pda: None,
            results_offset: None,
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_plugin_adapter_ix],
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
            plugin_adapters: vec![PluginAdapter::Oracle(Oracle {
                base_address: Pubkey::default(),
                pda: None,
                results_offset: ValidationResultsOffset::NoOffset,
            })],
        },
    )
    .await;
}

#[tokio::test]
async fn test_cannot_add_oracle_with_duplicate_lifecycle_checks() {
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
            plugin_adapters: vec![],
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
            plugin_adapters: vec![],
        },
    )
    .await;

    let add_plugin_adapter_ix = AddPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(PluginAdapterInitInfo::Oracle(OracleInitInfo {
            base_address: Pubkey::default(),
            init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
            lifecycle_checks: vec![
                (
                    HookableLifecycleEvent::Transfer,
                    AdapterCheckResult { flags: 4 },
                ),
                (
                    HookableLifecycleEvent::Transfer,
                    AdapterCheckResult { flags: 4 },
                ),
            ],
            pda: None,
            results_offset: None,
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_plugin_adapter_ix],
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

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
#[ignore]
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
            plugin_adapters: vec![],
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
            plugin_adapters: vec![],
        },
    )
    .await;

    let add_plugin_adapter_ix = AddPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(PluginAdapterInitInfo::DataStore(DataStoreInitInfo {
            init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
            data_authority: PluginAuthority::UpdateAuthority,
            schema: None,
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_plugin_adapter_ix],
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
            plugin_adapters: vec![PluginAdapter::DataStore(DataStore {
                data_authority: PluginAuthority::UpdateAuthority,
                schema: PluginAdapterSchema::Binary,
            })],
        },
    )
    .await;
}

#[tokio::test]
async fn test_temporarily_cannot_add_data_store() {
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
            plugin_adapters: vec![],
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
            plugin_adapters: vec![],
        },
    )
    .await;

    let add_plugin_adapter_ix = AddPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(PluginAdapterInitInfo::DataStore(DataStoreInitInfo {
            init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
            data_authority: PluginAuthority::UpdateAuthority,
            schema: None,
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_plugin_adapter_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::NotAvailable);

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn test_temporarily_cannot_add_data_store_on_collection() {
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
            plugin_adapters: vec![],
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
        },
    )
    .await;

    let add_plugin_adapter_ix = AddCollectionPluginAdapterV1Builder::new()
        .collection(collection.pubkey())
        .payer(context.payer.pubkey())
        .init_info(PluginAdapterInitInfo::DataStore(DataStoreInitInfo {
            init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
            data_authority: PluginAuthority::UpdateAuthority,
            schema: None,
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_plugin_adapter_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::NotAvailable);

    // TODO: add collection assert.
}

#[tokio::test]
async fn test_cannot_add_duplicate_plugin_adapter() {
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
            plugin_adapters: vec![],
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
            plugin_adapters: vec![],
        },
    )
    .await;

    let add_plugin_adapter_ix0 = AddPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(PluginAdapterInitInfo::Oracle(OracleInitInfo {
            base_address: Pubkey::default(),
            init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
            lifecycle_checks: vec![(
                HookableLifecycleEvent::Transfer,
                AdapterCheckResult { flags: 4 },
            )],
            pda: None,
            results_offset: None,
        }))
        .instruction();

    let add_plugin_adapter_ix1 = AddPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(PluginAdapterInitInfo::Oracle(OracleInitInfo {
            base_address: Pubkey::default(),
            init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
            lifecycle_checks: vec![(
                HookableLifecycleEvent::Transfer,
                AdapterCheckResult { flags: 4 },
            )],
            pda: None,
            results_offset: None,
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_plugin_adapter_ix0, add_plugin_adapter_ix1],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(1, error, MplCoreError::PluginAdapterAlreadyExists);
}
