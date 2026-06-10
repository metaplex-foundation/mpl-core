#![cfg(feature = "test-sbf")]
pub mod setup;

use std::borrow::BorrowMut;

use mpl_core::{
    accounts::BaseAssetV1,
    fetch_external_plugin_adapter_data_info,
    instructions::{
        AddExternalPluginAdapterV1Builder, UpdateCollectionV1Builder, UpdatePluginV1Builder,
        UpdateV1Builder, WriteExternalPluginAdapterDataV1Builder,
    },
    types::{
        AppDataInitInfo, Attribute, Attributes, Creator, ExternalPluginAdapterInitInfo,
        ExternalPluginAdapterKey, ExternalPluginAdapterSchema, FreezeDelegate, Plugin,
        PluginAuthority, PluginAuthorityPair, Royalties, RuleSet,
    },
    Asset, Collection,
};
pub use setup::*;

use solana_program::account_info::AccountInfo;
use solana_program_test::tokio;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

// ============================================================================
// Test 1: WriteExternalPluginAdapterDataV1 — regression test for shrinking the
// first of two AppData plugins.
//
// Previously, update_external_plugin_adapter_data() in plugins/utils.rs called
// resize_or_reallocate_account() before sol_memmove, so shrinking caused
// data_len() to return the new (smaller) size and saturating_sub yielded 0,
// moving nothing and corrupting trailing plugin data and/or the registry.
//
// The fix reorders: memmove first (while the buffer is full-size), then realloc.
// This test verifies the shrunk plugin, the trailing plugin, and the registry
// all survive intact.
// ============================================================================
#[tokio::test]
async fn test_write_external_plugin_adapter_data_shrink_preserves_second_plugin() {
    let mut context = program_test().start_with_context().await;

    // Step 1: Create an asset with TWO AppData plugins (different data authorities).
    let owner = Keypair::new();
    airdrop(&mut context, &owner.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let asset = Keypair::new();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: Some(owner.pubkey()),
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
                    schema: Some(ExternalPluginAdapterSchema::Binary),
                },
            )],
        },
    )
    .await
    .unwrap();

    // Add a second AppData plugin keyed by Owner authority.
    let ix = AddExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(ExternalPluginAdapterInitInfo::AppData(AppDataInitInfo {
            init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
            data_authority: PluginAuthority::Owner,
            schema: Some(ExternalPluginAdapterSchema::Binary),
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Step 2: Write LARGE data (500 bytes) to the FIRST AppData plugin.
    let large_data: Vec<u8> = (0..500).map(|i| (i % 256) as u8).collect();
    let ix = WriteExternalPluginAdapterDataV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::AppData(
            PluginAuthority::UpdateAuthority,
        ))
        .data(large_data.clone())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Step 3: Write a known pattern to the SECOND AppData plugin.
    let second_plugin_data: Vec<u8> = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let ix = WriteExternalPluginAdapterDataV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .authority(Some(owner.pubkey()))
        .key(ExternalPluginAdapterKey::AppData(PluginAuthority::Owner))
        .data(second_plugin_data.clone())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify both plugins are readable before shrink.
    let account_before = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .unwrap()
        .unwrap();
    let size_before = account_before.data.len();
    println!("Account size before shrink: {}", size_before);

    let asset_before = Asset::from_bytes(&account_before.data).unwrap();
    assert_eq!(asset_before.external_plugin_adapter_list.app_data.len(), 2);

    // Verify second plugin data is intact.
    {
        let mut account_copy = account_before.clone();
        let binding = asset.pubkey();
        let account_info = AccountInfo::new(
            &binding,
            false,
            false,
            &mut account_copy.lamports,
            account_copy.data.borrow_mut(),
            &account_copy.owner,
            false,
        );

        let (data_offset, data_len) = fetch_external_plugin_adapter_data_info::<BaseAssetV1>(
            &account_info,
            None,
            &ExternalPluginAdapterKey::AppData(PluginAuthority::Owner),
        )
        .unwrap();

        let data_slice = &account_copy.data[data_offset..data_offset + data_len];
        assert_eq!(
            data_slice, &second_plugin_data,
            "Second plugin data should be intact before shrink"
        );
    }

    // Step 4: SHRINK the first AppData from 500 bytes to 5 bytes.
    // This exercises the memmove-before-realloc path for a large shrink (495
    // bytes). The trailing plugin data and registry must be shifted left before
    // the account is truncated.
    let small_data: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let ix = WriteExternalPluginAdapterDataV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::AppData(
            PluginAuthority::UpdateAuthority,
        ))
        .data(small_data.clone())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Step 5: Verify the asset is still intact after shrink.
    let account_after = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .unwrap()
        .unwrap();
    let size_after = account_after.data.len();
    assert!(
        size_after < size_before,
        "Expected account to shrink from {} to {}, but it did not",
        size_before,
        size_after
    );

    // Deserialize the asset — should not fail.
    let asset_after = Asset::from_bytes(&account_after.data)
        .expect("Asset deserialization should succeed after shrink — registry must remain intact");

    // Both AppData plugins should still be present.
    assert_eq!(
        asset_after.external_plugin_adapter_list.app_data.len(),
        2,
        "Both AppData plugins should survive the shrink"
    );

    // Second plugin's data should be readable and unchanged.
    {
        let mut account_copy = account_after.clone();
        let binding = asset.pubkey();
        let account_info = AccountInfo::new(
            &binding,
            false,
            false,
            &mut account_copy.lamports,
            account_copy.data.borrow_mut(),
            &account_copy.owner,
            false,
        );

        let (data_offset, data_len) = fetch_external_plugin_adapter_data_info::<BaseAssetV1>(
            &account_info,
            None,
            &ExternalPluginAdapterKey::AppData(PluginAuthority::Owner),
        )
        .expect("Should be able to fetch second plugin data after shrink");

        assert!(
            data_offset + data_len <= account_after.data.len(),
            "Second plugin data out of bounds: offset={} len={} account_size={}",
            data_offset,
            data_len,
            account_after.data.len()
        );

        let actual_data = &account_after.data[data_offset..data_offset + data_len];
        assert_eq!(
            actual_data, &second_plugin_data,
            "Second plugin data must be unchanged after shrinking the first plugin"
        );
    }

    // First plugin's data should now equal the shrunk payload.
    {
        let mut account_copy = account_after.clone();
        let binding = asset.pubkey();
        let account_info = AccountInfo::new(
            &binding,
            false,
            false,
            &mut account_copy.lamports,
            account_copy.data.borrow_mut(),
            &account_copy.owner,
            false,
        );

        let (data_offset, data_len) = fetch_external_plugin_adapter_data_info::<BaseAssetV1>(
            &account_info,
            None,
            &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
        )
        .expect("Should be able to fetch first plugin data after shrink");

        assert!(
            data_offset + data_len <= account_after.data.len(),
            "First plugin data out of bounds: offset={} len={} account_size={}",
            data_offset,
            data_len,
            account_after.data.len()
        );

        let actual_data = &account_after.data[data_offset..data_offset + data_len];
        assert_eq!(
            actual_data, &small_data,
            "First plugin data must equal the shrunk payload"
        );
    }
}

// ============================================================================
// Test 2: UpdatePluginV1 — regression test for shrinking an Attributes plugin
// when a FreezeDelegate plugin follows it.
//
// Previously, process_update_plugin() in processor/update_plugin.rs called
// resize_or_reallocate_account() before sol_memmove, so on shrink the memmove
// source region could extend beyond the new (truncated) account boundary.
//
// The fix reorders: memmove first (reads from the full-size buffer), then
// realloc. This test verifies the Attributes content, the trailing
// FreezeDelegate, and the registry all survive intact.
// ============================================================================
#[tokio::test]
async fn test_update_plugin_shrink_attributes_preserves_trailing_plugins() {
    let mut context = program_test().start_with_context().await;

    // Step 1: Create an asset with a LARGE Attributes plugin and a FreezeDelegate.
    // Attributes is variable-size (Vec<Attribute>), so we can shrink it.
    let asset = Keypair::new();

    // Create with many attributes to make it large.
    let large_attributes: Vec<Attribute> = (0..30)
        .map(|i| Attribute {
            key: format!("key_{:03}", i),
            value: format!(
                "value_{:03}_padding_to_make_this_larger_{}",
                i,
                "x".repeat(20)
            ),
        })
        .collect();

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
            plugins: vec![
                PluginAuthorityPair {
                    plugin: Plugin::Attributes(Attributes {
                        attribute_list: large_attributes.clone(),
                    }),
                    authority: None,
                },
                PluginAuthorityPair {
                    plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                    authority: None,
                },
            ],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    // Verify initial state.
    let account_before = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .unwrap()
        .unwrap();
    let size_before = account_before.data.len();
    println!("Account size before shrink: {}", size_before);

    let asset_before = Asset::from_bytes(&account_before.data).unwrap();
    assert!(asset_before.plugin_list.freeze_delegate.is_some());
    let attrs = asset_before.plugin_list.attributes.as_ref().unwrap();
    assert_eq!(attrs.attributes.attribute_list.len(), 30);

    // Step 2: Update Attributes to have very few attributes (massive shrink).
    let small_attributes = vec![Attribute {
        key: "x".to_string(),
        value: "y".to_string(),
    }];

    let ix = UpdatePluginV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .plugin(Plugin::Attributes(Attributes {
            attribute_list: small_attributes.clone(),
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Attributes shrink transaction should succeed");

    // Step 3: Verify the asset is still fully readable and FreezeDelegate intact.
    let account_after = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .unwrap()
        .unwrap();
    let size_after = account_after.data.len();
    assert!(
        size_after < size_before,
        "Expected account to shrink from {} to {}, but it did not",
        size_before,
        size_after
    );

    let asset_after = Asset::from_bytes(&account_after.data)
        .expect("Asset deserialization should succeed after Attributes shrink");

    // Check Attributes has the exact expected content.
    let attrs_after = asset_after
        .plugin_list
        .attributes
        .as_ref()
        .expect("Attributes plugin must still be present after shrink");
    assert_eq!(
        attrs_after.attributes.attribute_list.len(),
        1,
        "Attributes should have exactly 1 entry after update"
    );
    assert_eq!(attrs_after.attributes.attribute_list[0].key, "x");
    assert_eq!(attrs_after.attributes.attribute_list[0].value, "y");

    // Check FreezeDelegate is still intact.
    let fd = asset_after
        .plugin_list
        .freeze_delegate
        .as_ref()
        .expect("FreezeDelegate must still be present after Attributes shrink");
    assert_eq!(
        fd.freeze_delegate,
        FreezeDelegate { frozen: false },
        "FreezeDelegate should be unchanged"
    );
}

// ============================================================================
// Test 3: WriteExternalPluginAdapterDataV1 — shrink with a single AppData
// plugin (regression/coverage guard).
//
// With only one AppData plugin, sol_memmove has no trailing plugin data to
// shift — the tail length is just the registry, which is re-serialized from
// the in-memory PluginRegistryV1 after the move anyway. So this case does
// not exercise the specific realloc-before-memmove corruption path that
// multi-plugin layouts hit. It still guards against shrink-related regressions
// (e.g. incorrect new_size, data_offset math, or registry save errors).
// ============================================================================
#[tokio::test]
async fn test_write_external_plugin_adapter_data_single_plugin_shrink() {
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
                    schema: Some(ExternalPluginAdapterSchema::Binary),
                },
            )],
        },
    )
    .await
    .unwrap();

    // Write large data.
    let large_data: Vec<u8> = vec![0xAB; 800];
    let ix = WriteExternalPluginAdapterDataV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::AppData(
            PluginAuthority::UpdateAuthority,
        ))
        .data(large_data.clone())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify pre-shrink.
    let account_before = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .unwrap()
        .unwrap();
    let size_before = account_before.data.len();
    println!("Account size before shrink: {}", size_before);

    let asset_before = Asset::from_bytes(&account_before.data).unwrap();
    assert_eq!(asset_before.external_plugin_adapter_list.app_data.len(), 1);

    // Shrink to tiny data.
    let small_data: Vec<u8> = vec![0x01];
    let ix = WriteExternalPluginAdapterDataV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::AppData(
            PluginAuthority::UpdateAuthority,
        ))
        .data(small_data.clone())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Shrink transaction should succeed — program must handle shrinking gracefully");

    // Verify post-shrink: can we still deserialize and read the data?
    let account_after = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .unwrap()
        .unwrap();
    let size_after = account_after.data.len();
    assert!(
        size_after < size_before,
        "Expected account to shrink from {} to {}, but it did not",
        size_before,
        size_after
    );

    let asset_after = Asset::from_bytes(&account_after.data)
        .expect("Asset deserialization should succeed after single-plugin shrink");

    assert_eq!(
        asset_after.external_plugin_adapter_list.app_data.len(),
        1,
        "AppData plugin must still be present after shrink"
    );

    // Verify the data content matches what we wrote.
    {
        let mut account_copy = account_after.clone();
        let binding = asset.pubkey();
        let account_info = AccountInfo::new(
            &binding,
            false,
            false,
            &mut account_copy.lamports,
            account_copy.data.borrow_mut(),
            &account_copy.owner,
            false,
        );

        let (data_offset, data_len) = fetch_external_plugin_adapter_data_info::<BaseAssetV1>(
            &account_info,
            None,
            &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
        )
        .expect("Should be able to fetch AppData after single-plugin shrink");

        assert!(
            data_offset + data_len <= account_after.data.len(),
            "AppData region out of bounds: offset={} len={} account_size={}",
            data_offset,
            data_len,
            account_after.data.len()
        );

        let actual = &account_after.data[data_offset..data_offset + data_len];
        assert_eq!(
            actual, &small_data,
            "AppData content must match the shrunk payload"
        );
    }
}

// ============================================================================
// Test 4: UpdatePluginV1 — regression test for shrinking an Attributes plugin
// when an AppData external plugin is also present.
//
// Previously, process_update_plugin() in processor/update_plugin.rs called
// resize_or_reallocate_account() before sol_memmove, so on shrink the
// memmove's source region could extend beyond the truncated account boundary,
// corrupting the external plugin data stored after Attributes.
//
// The fix reorders: memmove first, then realloc. This test verifies the
// Attributes content, the trailing AppData plugin, and the registry all
// survive intact.
// ============================================================================
#[tokio::test]
async fn test_update_plugin_shrink_attributes_preserves_external_plugin() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();

    // Create with large Attributes + an AppData external plugin.
    let large_attributes: Vec<Attribute> = (0..25)
        .map(|i| Attribute {
            key: format!("attr_{:03}", i),
            value: format!("val_{:03}_{}", i, "abcdefghijklmnopqrstuvwxyz".repeat(2)),
        })
        .collect();

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
                plugin: Plugin::Attributes(Attributes {
                    attribute_list: large_attributes.clone(),
                }),
                authority: None,
            }],
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::AppData(
                AppDataInitInfo {
                    init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                    data_authority: PluginAuthority::UpdateAuthority,
                    schema: Some(ExternalPluginAdapterSchema::Binary),
                },
            )],
        },
    )
    .await
    .unwrap();

    // Write data to AppData.
    let app_data_content: Vec<u8> = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE];
    let ix = WriteExternalPluginAdapterDataV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::AppData(
            PluginAuthority::UpdateAuthority,
        ))
        .data(app_data_content.clone())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify pre-shrink state.
    let account_before = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .unwrap()
        .unwrap();
    let size_before = account_before.data.len();

    let asset_before = Asset::from_bytes(&account_before.data).unwrap();
    assert!(asset_before.plugin_list.attributes.is_some());
    assert_eq!(asset_before.external_plugin_adapter_list.app_data.len(), 1);
    println!("Account size before shrink: {}", size_before);

    // Shrink Attributes drastically.
    let small_attributes = vec![Attribute {
        key: "a".to_string(),
        value: "b".to_string(),
    }];

    let ix = UpdatePluginV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .plugin(Plugin::Attributes(Attributes {
            attribute_list: small_attributes,
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Attributes shrink transaction should succeed");

    // Verify post-shrink.
    let account_after = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .unwrap()
        .unwrap();
    let size_after = account_after.data.len();
    assert!(
        size_after < size_before,
        "Expected account to shrink from {} to {}, but it did not",
        size_before,
        size_after
    );

    let asset_after = Asset::from_bytes(&account_after.data)
        .expect("Asset deserialization should succeed after Attributes shrink");

    // Check Attributes has the exact expected content.
    let attrs_after = asset_after
        .plugin_list
        .attributes
        .as_ref()
        .expect("Attributes plugin must still be present after shrink");
    assert_eq!(
        attrs_after.attributes.attribute_list.len(),
        1,
        "Attributes should have exactly 1 entry after update"
    );
    assert_eq!(attrs_after.attributes.attribute_list[0].key, "a");
    assert_eq!(attrs_after.attributes.attribute_list[0].value, "b");

    // Verify AppData external plugin is still present.
    assert_eq!(
        asset_after.external_plugin_adapter_list.app_data.len(),
        1,
        "AppData plugin must still be present after Attributes shrink"
    );

    // Verify the AppData content is unchanged.
    {
        let mut account_copy = account_after.clone();
        let binding = asset.pubkey();
        let account_info = AccountInfo::new(
            &binding,
            false,
            false,
            &mut account_copy.lamports,
            account_copy.data.borrow_mut(),
            &account_copy.owner,
            false,
        );

        let (data_offset, data_len) = fetch_external_plugin_adapter_data_info::<BaseAssetV1>(
            &account_info,
            None,
            &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
        )
        .expect("Should be able to fetch AppData after Attributes shrink");

        assert!(
            data_offset + data_len <= account_after.data.len(),
            "AppData region out of bounds: offset={} len={} account_size={}",
            data_offset,
            data_len,
            account_after.data.len()
        );

        let actual = &account_after.data[data_offset..data_offset + data_len];
        assert_eq!(
            actual, &app_data_content,
            "AppData content must be unchanged after Attributes shrink"
        );
    }
}

// Regression: shrinking asset name + uri must preserve attached plugin data.
#[tokio::test]
async fn test_update_v1_shrink_name_uri_preserves_plugin() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();

    let long_name = "x".repeat(100);
    let long_uri = format!("https://example.com/{}", "y".repeat(200));

    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset,
            data_state: None,
            name: Some(long_name.clone()),
            uri: Some(long_uri.clone()),
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

    let account_before = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .unwrap()
        .unwrap();
    let size_before = account_before.data.len();
    println!("Account size before name/uri shrink: {}", size_before);

    let asset_before = Asset::from_bytes(&account_before.data).unwrap();
    assert_eq!(asset_before.base.name, long_name);
    assert_eq!(asset_before.base.uri, long_uri);
    assert!(asset_before.plugin_list.freeze_delegate.is_some());

    let short_name = "a".to_string();
    let short_uri = "b".to_string();

    let ix = UpdateV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .new_name(short_name.clone())
        .new_uri(short_uri.clone())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Asset name/uri shrink transaction should succeed");

    let account_after = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .unwrap()
        .unwrap();
    let size_after = account_after.data.len();
    assert!(
        size_after < size_before,
        "Expected account to shrink from {} to {}, but it did not",
        size_before,
        size_after
    );

    let asset_after = Asset::from_bytes(&account_after.data).expect(
        "Asset deserialization should succeed after name/uri shrink; registry must remain intact",
    );

    assert_eq!(asset_after.base.name, short_name, "Name should be updated");
    assert_eq!(asset_after.base.uri, short_uri, "URI should be updated");

    let fd = asset_after
        .plugin_list
        .freeze_delegate
        .as_ref()
        .expect("FreezeDelegate must still be present after name/uri shrink");
    assert_eq!(
        fd.freeze_delegate,
        FreezeDelegate { frozen: false },
        "FreezeDelegate state should be unchanged"
    );
}

// Regression: shrinking collection name + uri must preserve attached plugin data.
#[tokio::test]
async fn test_update_collection_v1_shrink_name_uri_preserves_plugin() {
    let mut context = program_test().start_with_context().await;

    let collection = Keypair::new();

    let long_name = "c".repeat(100);
    let long_uri = format!("https://example.com/collection/{}", "d".repeat(200));

    let royalties = Royalties {
        basis_points: 500,
        creators: vec![Creator {
            address: context.payer.pubkey(),
            percentage: 100,
        }],
        rule_set: RuleSet::None,
    };

    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection,
            update_authority: None,
            payer: None,
            name: Some(long_name.clone()),
            uri: Some(long_uri.clone()),
            plugins: vec![PluginAuthorityPair {
                plugin: Plugin::Royalties(royalties.clone()),
                authority: None,
            }],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    let account_before = context
        .banks_client
        .get_account(collection.pubkey())
        .await
        .unwrap()
        .unwrap();
    let size_before = account_before.data.len();
    println!("Collection size before name/uri shrink: {}", size_before);

    let collection_before = Collection::from_bytes(&account_before.data).unwrap();
    assert_eq!(collection_before.base.name, long_name);
    assert_eq!(collection_before.base.uri, long_uri);
    assert!(collection_before.plugin_list.royalties.is_some());

    let short_name = "e".to_string();
    let short_uri = "f".to_string();

    let ix = UpdateCollectionV1Builder::new()
        .collection(collection.pubkey())
        .payer(context.payer.pubkey())
        .new_name(short_name.clone())
        .new_uri(short_uri.clone())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Collection name/uri shrink transaction should succeed");

    let account_after = context
        .banks_client
        .get_account(collection.pubkey())
        .await
        .unwrap()
        .unwrap();
    let size_after = account_after.data.len();
    assert!(
        size_after < size_before,
        "Expected collection to shrink from {} to {}, but it did not",
        size_before,
        size_after
    );

    let collection_after = Collection::from_bytes(&account_after.data).expect(
        "Collection deserialization should succeed after name/uri shrink; registry must remain intact",
    );

    assert_eq!(
        collection_after.base.name, short_name,
        "Collection name should be updated"
    );
    assert_eq!(
        collection_after.base.uri, short_uri,
        "Collection URI should be updated"
    );

    let royalties_after = collection_after
        .plugin_list
        .royalties
        .as_ref()
        .expect("Royalties plugin must still be present after name/uri shrink");
    assert_eq!(
        royalties_after.royalties, royalties,
        "Royalties plugin contents should be unchanged"
    );
}
