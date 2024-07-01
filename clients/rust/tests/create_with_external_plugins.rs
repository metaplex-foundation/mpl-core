#![cfg(feature = "test-sbf")]
pub mod setup;
use serde_json::json;
use std::borrow::BorrowMut;

use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    errors::MplCoreError,
    fetch_external_plugin_adapter, fetch_external_plugin_adapter_data_as_string,
    fetch_external_plugin_adapter_data_info,
    instructions::{
        WriteCollectionExternalPluginAdapterDataV1Builder, WriteExternalPluginAdapterDataV1Builder,
    },
    types::{
        AppData, AppDataInitInfo, ExternalCheckResult, ExternalPluginAdapter,
        ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey, ExternalPluginAdapterSchema,
        HookableLifecycleEvent, LifecycleHook, LifecycleHookInitInfo, Oracle, OracleInitInfo,
        PluginAuthority, UpdateAuthority, ValidationResultsOffset,
    },
    Asset, Collection,
};
pub use setup::*;

use solana_program::pubkey;
use solana_program_test::tokio;
use solana_sdk::{
    account_info::AccountInfo, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::LifecycleHook(
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
    .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::NotAvailable);
}

#[tokio::test]
async fn test_temporarily_cannot_create_lifecycle_hook_on_collection() {
    let mut context = program_test().start_with_context().await;

    let collection = Keypair::new();
    let error = create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection,
            update_authority: None,
            payer: None,
            name: None,
            uri: None,
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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
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
                base_address_config: None,
                results_offset: None,
            })],
        },
    )
    .await
    .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::DuplicateLifecycleChecks);
}

#[tokio::test]
async fn test_create_app_data() {
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
}

#[tokio::test]
async fn test_create_and_fetch_app_data() {
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
                    schema: Some(ExternalPluginAdapterSchema::Json),
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
                schema: ExternalPluginAdapterSchema::Json,
            })],
        },
    )
    .await;

    // Test JSON.
    let test_json_obj = json!({
        "message": "Hello",
        "target": "world"
    });
    let test_json_str = serde_json::to_string(&test_json_obj).unwrap();
    let test_json_vec = test_json_str.as_bytes().to_vec();

    // Write data.
    let ix = WriteExternalPluginAdapterDataV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::AppData(
            PluginAuthority::UpdateAuthority,
        ))
        .data(test_json_vec)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Get account.
    let mut account = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .unwrap()
        .unwrap();

    let binding = asset.pubkey();
    let account_info = AccountInfo::new(
        &binding,
        false,
        false,
        &mut account.lamports,
        account.data.borrow_mut(),
        &account.owner,
        false,
        0,
    );
    let (auth, app_data, offset) = fetch_external_plugin_adapter::<BaseAssetV1, AppData>(
        &account_info,
        None,
        &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
    )
    .unwrap();

    println!("App data: {:#?}", app_data);
    println!("Auth: {:#?}", auth);
    println!("Offset: {:#?}", offset);

    // Fetch the actual app data.  Validate multiple methods.

    // First, get app data data offset and length.
    let (data_offset_1, data_len_1) = fetch_external_plugin_adapter_data_info::<BaseAssetV1>(
        &account_info,
        None,
        &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
    )
    .unwrap();

    // Second, get app data as a String.
    let (data_string, data_offset_2, data_len_2) =
        fetch_external_plugin_adapter_data_as_string::<BaseAssetV1>(
            &account_info,
            None,
            &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
        )
        .unwrap();

    // Validate app data.
    assert_eq!(data_string, test_json_str);
    assert_eq!(data_len_1, 36);

    // Validate data matches.
    assert_eq!(data_offset_1, data_offset_2);
    assert_eq!(data_len_1, data_len_2);

    // Third, get app data offset and length from a full `Asset` deserialization.
    let asset = Asset::from_bytes(&account.data).unwrap();
    let app_data_with_data = asset.external_plugin_adapter_list.app_data.first().unwrap();

    // Validate data matches.
    assert_eq!(data_offset_1, app_data_with_data.data_offset);
    assert_eq!(data_len_1, app_data_with_data.data_len);

    println!("Data string: {:#?}", data_string);
    println!("Data offset: {:#?}", data_offset_1);
    println!("Data len: {:#?}", data_len_1);
}

#[tokio::test]
async fn test_collection_create_and_fetch_app_data() {
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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::AppData(
                AppDataInitInfo {
                    init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                    data_authority: PluginAuthority::UpdateAuthority,
                    schema: Some(ExternalPluginAdapterSchema::Json),
                },
            )],
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
            external_plugin_adapters: vec![ExternalPluginAdapter::AppData(AppData {
                data_authority: PluginAuthority::UpdateAuthority,
                schema: ExternalPluginAdapterSchema::Json,
            })],
        },
    )
    .await;

    // Test JSON.
    let test_json_obj = json!({
        "message": "Hello",
        "target": "world"
    });
    let test_json_str = serde_json::to_string(&test_json_obj).unwrap();
    let test_json_vec = test_json_str.as_bytes().to_vec();

    // Write data.
    let ix = WriteCollectionExternalPluginAdapterDataV1Builder::new()
        .collection(collection.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::AppData(
            PluginAuthority::UpdateAuthority,
        ))
        .data(test_json_vec)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Get account.
    let mut account = context
        .banks_client
        .get_account(collection.pubkey())
        .await
        .unwrap()
        .unwrap();

    let binding = collection.pubkey();
    let account_info = AccountInfo::new(
        &binding,
        false,
        false,
        &mut account.lamports,
        account.data.borrow_mut(),
        &account.owner,
        false,
        0,
    );
    let (auth, app_data, offset) = fetch_external_plugin_adapter::<BaseCollectionV1, AppData>(
        &account_info,
        None,
        &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
    )
    .unwrap();

    println!("App data: {:#?}", app_data);
    println!("Auth: {:#?}", auth);
    println!("Offset: {:#?}", offset);

    // Fetch the actual app data.  Validate multiple methods.

    // First, get app data data offset and length.
    let (data_offset_1, data_len_1) = fetch_external_plugin_adapter_data_info::<BaseCollectionV1>(
        &account_info,
        None,
        &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
    )
    .unwrap();

    // Second, get app data as a String.
    let (data_string, data_offset_2, data_len_2) =
        fetch_external_plugin_adapter_data_as_string::<BaseCollectionV1>(
            &account_info,
            None,
            &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
        )
        .unwrap();

    // Validate app data.
    assert_eq!(data_string, test_json_str);
    assert_eq!(data_len_1, 36);

    // Validate data matches.
    assert_eq!(data_offset_1, data_offset_2);
    assert_eq!(data_len_1, data_len_2);

    // Third, get app data offset and length from a full `Asset` deserialization.
    let collection = Collection::from_bytes(&account.data).unwrap();
    let app_data_with_data = collection
        .external_plugin_adapter_list
        .app_data
        .first()
        .unwrap();

    // Validate data matches.
    assert_eq!(data_offset_1, app_data_with_data.data_offset);
    assert_eq!(data_len_1, app_data_with_data.data_len);

    println!("Data string: {:#?}", data_string);
    println!("Data offset: {:#?}", data_offset_1);
    println!("Data len: {:#?}", data_len_1);
}
