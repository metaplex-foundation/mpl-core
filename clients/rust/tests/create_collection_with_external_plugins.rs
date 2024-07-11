#![cfg(feature = "test-sbf")]
pub mod setup;
use serde_json::json;
use std::borrow::BorrowMut;

use mpl_core::{
    accounts::BaseCollectionV1,
    convert_external_plugin_adapter_data_to_string,
    errors::MplCoreError,
    fetch_external_plugin_adapter, fetch_external_plugin_adapter_data_info,
    fetch_wrapped_external_plugin_adapter,
    instructions::WriteCollectionExternalPluginAdapterDataV1Builder,
    types::{
        AppData, AppDataInitInfo, ExternalCheckResult, ExternalPluginAdapter,
        ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey, ExternalPluginAdapterSchema,
        HookableLifecycleEvent, LifecycleHookInitInfo, Oracle, OracleInitInfo, PluginAuthority,
        ValidationResultsOffset,
    },
    Collection,
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
async fn test_create_lifecycle_hook_on_collection() {
    // Note we can reference the asset version of this test when ready to implement.
    todo!()
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_lifecycle_hook_with_duplicate_lifecycle_checks_on_collection() {
    // Note we can reference the asset version of this test when ready to implement.
    todo!()
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
async fn test_create_oracle_on_collection() {
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
async fn test_cannot_create_oracle_with_duplicate_lifecycle_checks_on_collection() {
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
async fn test_create_app_data_on_collection() {
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
                    schema: None,
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
                schema: ExternalPluginAdapterSchema::Binary,
            })],
        },
    )
    .await;
}

#[tokio::test]
async fn test_create_and_fetch_app_data_on_collection() {
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

    // Fetch external plugin adapter two ways.
    // First, get the external plugin adapter in its enum.
    let (registry_record, external_plugin) =
        fetch_wrapped_external_plugin_adapter::<BaseCollectionV1>(
            &account_info,
            None,
            &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
        )
        .unwrap();

    let inner_app_data = match external_plugin {
        ExternalPluginAdapter::AppData(app_data) => app_data,
        _ => panic!("Unexpected external plugin adapter"),
    };

    // Second, get the inner `AppData` object directly.
    let (auth, app_data, offset) = fetch_external_plugin_adapter::<BaseCollectionV1, AppData>(
        &account_info,
        None,
        &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
    )
    .unwrap();

    // Validate the data matches between the two fetches.
    assert_eq!(registry_record.authority, auth);
    assert_eq!(inner_app_data, app_data);

    println!("App data: {:#?}", app_data);
    println!("Auth: {:#?}", auth);
    println!("Offset: {:#?}", offset);

    // Fetch the actual app data.  Validate multiple methods.

    // First, get app data data offset and length directly.
    let (data_offset, data_len) = fetch_external_plugin_adapter_data_info::<BaseCollectionV1>(
        &account_info,
        None,
        &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
    )
    .unwrap();

    // Second, get app data offset and length from a full `Collection` deserialization.
    let full_collection = Collection::from_bytes(&account.data).unwrap();
    let app_data_with_data = full_collection
        .external_plugin_adapter_list
        .app_data
        .first()
        .unwrap();

    // Validate data matches between two methods.
    assert_eq!(data_offset, app_data_with_data.data_offset);
    assert_eq!(data_len, app_data_with_data.data_len);

    // Convert data to string.
    let data_end = data_offset.checked_add(data_len).unwrap();
    let data_slice = &account.data[data_offset..data_end];
    let data_string = convert_external_plugin_adapter_data_to_string(&app_data.schema, data_slice);

    // Validate app data.
    assert_eq!(data_string, test_json_str);
    assert_eq!(data_len, 36);

    println!("Data string: {:#?}", data_string);
    println!("Data offset: {:#?}", data_offset);
    println!("Data len: {:#?}", data_len);
}
