#![cfg(feature = "test-sbf")]
pub mod setup;

use std::borrow::BorrowMut;

use mpl_core::{
    accounts::BaseAssetV1,
    fetch_external_plugin_adapter_data_info,
    instructions::{
        AddExternalPluginAdapterV1Builder, UpdatePluginV1Builder,
        WriteExternalPluginAdapterDataV1Builder,
    },
    types::{
        AppDataInitInfo, Attribute, Attributes, ExternalPluginAdapterInitInfo,
        ExternalPluginAdapterKey, ExternalPluginAdapterSchema, FreezeDelegate, Plugin,
        PluginAuthority, PluginAuthorityPair,
    },
    Asset,
};
pub use setup::*;

use solana_program::account_info::AccountInfo;
use solana_program_test::tokio;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

// ============================================================================
// Test 1: WriteExternalPluginAdapterDataV1 — shrink first AppData corrupts
// second AppData's data and/or the PluginRegistryV1.
//
// Bug location: plugins/utils.rs, update_external_plugin_adapter_data()
//   - Line 504: resize_or_reallocate_account() shrinks the account FIRST
//   - Line 522: sol_memmove uses account.data_len().saturating_sub(next_plugin_offset)
//     After realloc, data_len() returns the NEW smaller size.
//     When shrinkage > tail_size, saturating_sub yields 0 → nothing is moved.
//     Result: tail plugins' data and/or the registry are lost/corrupted.
// ============================================================================
#[tokio::test]
async fn test_write_external_plugin_adapter_data_shrink_corrupts_second_plugin() {
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
            0,
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
    // This is where the bug triggers: shrinkage (495) >> tail data size,
    // so sol_memmove moves 0 bytes, corrupting the second plugin's data
    // and/or the PluginRegistryV1.
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

    // The shrink transaction itself may succeed (no runtime panic) because
    // sol_memmove with length 0 is a no-op. But the state is now corrupt.
    let shrink_result = context.banks_client.process_transaction(tx).await;
    println!("Shrink transaction result: {:?}", shrink_result);

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
            0,
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
}

// ============================================================================
// Test 2: UpdatePluginV1 — shrink an Attributes plugin when there are other
// plugins after it. The realloc happens before sol_memmove, reading from
// memory beyond the new account size (UB on Solana).
//
// Bug location: processor/update_plugin.rs
//   - Line 201: resize_or_reallocate_account() shrinks the account FIRST
//   - Lines 207-214: sol_memmove reads from [next_plugin_offset, registry_offset)
//     When registry_offset > new_size, the source extends beyond the official
//     account boundary. On current Solana runtime the memory is physically
//     preserved, but this is undefined behavior.
// ============================================================================
#[tokio::test]
async fn test_update_plugin_shrink_attributes_with_trailing_plugins() {
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

    let update_result = context.banks_client.process_transaction(tx).await;
    println!("Update (shrink) transaction result: {:?}", update_result);

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

    let parse_result = Asset::from_bytes(&account_after.data);
    match &parse_result {
        Ok(asset_after) => {
            // Check Attributes was updated.
            let attrs_after = asset_after.plugin_list.attributes.as_ref();
            match attrs_after {
                Some(a) => {
                    assert_eq!(
                        a.attributes.attribute_list.len(),
                        1,
                        "Attributes should have 1 entry after update"
                    );
                    assert_eq!(a.attributes.attribute_list[0].key, "x");
                    assert_eq!(a.attributes.attribute_list[0].value, "y");
                }
                None => {
                    panic!("VULNERABILITY CONFIRMED: Attributes plugin lost after shrink update!");
                }
            }

            // Check FreezeDelegate is still intact.
            match &asset_after.plugin_list.freeze_delegate {
                Some(fd) => {
                    assert_eq!(
                        fd.freeze_delegate,
                        FreezeDelegate { frozen: false },
                        "FreezeDelegate should be unchanged"
                    );
                    println!("FreezeDelegate intact after shrink.");
                }
                None => {
                    panic!(
                        "VULNERABILITY CONFIRMED: FreezeDelegate plugin lost after Attributes \
                         shrink! Trailing plugin data was corrupted by realloc-before-memmove."
                    );
                }
            }
        }
        Err(e) => {
            panic!(
                "VULNERABILITY CONFIRMED: Asset deserialization failed after Attributes shrink \
                 — PluginRegistryV1 is corrupted: {:?}",
                e
            );
        }
    }
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
async fn test_write_external_plugin_adapter_data_shrink_corrupts_registry() {
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

    let parse_result = Asset::from_bytes(&account_after.data);
    match &parse_result {
        Ok(asset_after) => {
            if asset_after.external_plugin_adapter_list.app_data.is_empty() {
                panic!(
                    "VULNERABILITY CONFIRMED: AppData plugin lost after shrink — registry corrupted"
                );
            }

            // Verify the data is what we wrote.
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
                0,
            );

            let data_result = fetch_external_plugin_adapter_data_info::<BaseAssetV1>(
                &account_info,
                None,
                &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
            );

            match data_result {
                Ok((data_offset, data_len)) => {
                    if data_offset + data_len > account_after.data.len() {
                        panic!(
                            "VULNERABILITY CONFIRMED: Data region out of bounds after shrink! \
                             offset={} len={} account_size={}",
                            data_offset,
                            data_len,
                            account_after.data.len()
                        );
                    }
                    let actual = &account_after.data[data_offset..data_offset + data_len];
                    assert_eq!(
                        actual, &small_data,
                        "Data should match what was written after shrink"
                    );
                    println!("Single-plugin shrink: data appears intact.");
                }
                Err(e) => {
                    panic!(
                        "VULNERABILITY CONFIRMED: Cannot read data after shrink: {:?}",
                        e
                    );
                }
            }
        }
        Err(e) => {
            panic!(
                "VULNERABILITY CONFIRMED: Asset deserialization failed after shrink: {:?}",
                e
            );
        }
    }
}

// ============================================================================
// Test 4: UpdatePluginV1 — shrink Attributes with an AppData external plugin
// also present. This tests cross-plugin-type corruption: internal plugin
// shrink affecting external plugin data stored after it.
// ============================================================================
#[tokio::test]
async fn test_update_plugin_shrink_attributes_corrupts_external_plugin() {
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

    let update_result = context.banks_client.process_transaction(tx).await;
    println!("Attributes shrink result: {:?}", update_result);

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

    let parse_result = Asset::from_bytes(&account_after.data);
    match &parse_result {
        Ok(asset_after) => {
            // Verify Attributes updated.
            let attrs = asset_after.plugin_list.attributes.as_ref();
            match attrs {
                Some(a) => {
                    assert_eq!(a.attributes.attribute_list.len(), 1);
                }
                None => {
                    panic!("VULNERABILITY CONFIRMED: Attributes plugin lost after shrink!");
                }
            }

            // Verify AppData external plugin and its data are intact.
            if asset_after.external_plugin_adapter_list.app_data.is_empty() {
                panic!(
                    "VULNERABILITY CONFIRMED: AppData external plugin lost after Attributes \
                     shrink! The realloc-before-memmove corrupted the external plugin registry."
                );
            }

            // Verify the actual data content.
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
                0,
            );

            let data_result = fetch_external_plugin_adapter_data_info::<BaseAssetV1>(
                &account_info,
                None,
                &ExternalPluginAdapterKey::AppData(PluginAuthority::UpdateAuthority),
            );

            match data_result {
                Ok((data_offset, data_len)) => {
                    if data_offset + data_len > account_after.data.len() {
                        panic!(
                            "VULNERABILITY CONFIRMED: AppData data region out of bounds! \
                             offset={} len={} account_size={}",
                            data_offset,
                            data_len,
                            account_after.data.len()
                        );
                    }
                    let actual = &account_after.data[data_offset..data_offset + data_len];
                    if actual != &app_data_content {
                        panic!(
                            "VULNERABILITY CONFIRMED: AppData content corrupted after Attributes \
                             shrink!\nExpected: {:?}\nActual:   {:?}",
                            app_data_content, actual
                        );
                    }
                    println!("AppData content intact after Attributes shrink.");
                }
                Err(e) => {
                    panic!(
                        "VULNERABILITY CONFIRMED: Cannot read AppData after Attributes shrink: {:?}",
                        e
                    );
                }
            }
        }
        Err(e) => {
            panic!(
                "VULNERABILITY CONFIRMED: Asset deserialization failed after Attributes shrink \
                 — registry corrupted: {:?}",
                e
            );
        }
    }
}
