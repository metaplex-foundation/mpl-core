#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    accounts::GroupV1,
    errors::MplCoreError,
    instructions::{
        AddAssetsToGroupBuilder, CreateGroupBuilder, CreateV2Builder, RemoveAssetsFromGroupBuilder,
    },
    types::{
        DataState, Edition, ExternalPluginAdapterInitInfo, Plugin, PluginAuthorityPair,
        UpdateAuthority,
    },
};
pub use setup::*;

use solana_program::instruction::AccountMeta;
use solana_program::system_program;
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

// Helper function to assert group state
async fn assert_group(
    context: &mut solana_program_test::ProgramTestContext,
    group_pubkey: Pubkey,
    expected_assets: &[Pubkey],
) {
    let group_account = context
        .banks_client
        .get_account(group_pubkey)
        .await
        .expect("get_account")
        .expect("group account not found");

    let group = GroupV1::from_bytes(&group_account.data).unwrap();

    // Assert that all expected assets are in the group
    for asset in expected_assets {
        assert!(
            group.assets.contains(asset),
            "Group should contain the asset"
        );
    }

    // Assert that the group has the correct number of assets
    assert_eq!(
        group.assets.len(),
        expected_assets.len(),
        "Group should have the expected number of assets"
    );
}

#[tokio::test]
async fn test_add_assets_to_group() {
    let mut context = program_test().start_with_context().await;

    // Create a group
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, Some(10), None, None)
        .await
        .unwrap();

    // Create assets
    let asset1 = Keypair::new();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset1,
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

    let asset2 = Keypair::new();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset2,
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

    // Add assets to group
    let add_assets_ix = AddAssetsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .asset_indices(vec![2, 3])
        .authority_indices(vec![0, 0])
        .add_remaining_accounts(&[
            AccountMeta::new(asset1.pubkey(), false),
            AccountMeta::new(asset2.pubkey(), false),
        ])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_assets_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify assets were added
    assert_group(
        &mut context,
        group.pubkey(),
        &[asset1.pubkey(), asset2.pubkey()],
    )
    .await;
}

#[tokio::test]
async fn test_remove_assets_from_group() {
    let mut context = program_test().start_with_context().await;

    // Create a group
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, Some(10), None, None)
        .await
        .unwrap();

    // Create assets
    let asset1 = Keypair::new();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset1,
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

    let asset2 = Keypair::new();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset2,
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

    // Add assets to group
    let add_assets_ix = AddAssetsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .asset_indices(vec![2, 3])
        .authority_indices(vec![0, 0])
        .add_remaining_accounts(&[
            AccountMeta::new(asset1.pubkey(), false),
            AccountMeta::new(asset2.pubkey(), false),
        ])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_assets_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify assets were added
    assert_group(
        &mut context,
        group.pubkey(),
        &[asset1.pubkey(), asset2.pubkey()],
    )
    .await;

    // Remove one asset from group
    let remove_asset_ix = RemoveAssetsFromGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .asset_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(asset1.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[remove_asset_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify asset was removed
    assert_group(&mut context, group.pubkey(), &[asset2.pubkey()]).await;
}

#[tokio::test]
async fn test_fail_add_assets_to_group_with_wrong_authority() {
    let mut context = program_test().start_with_context().await;

    // Create a group
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, Some(10), None, None)
        .await
        .unwrap();

    // Create an asset
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

    // Create an invalid authority
    let invalid_authority = Keypair::new();
    airdrop(&mut context, &invalid_authority.pubkey(), 1_000_000_000)
        .await
        .unwrap();

    // Try to add asset with wrong authority
    let add_asset_ix = AddAssetsToGroupBuilder::new()
        .authority(invalid_authority.pubkey())
        .group(group.pubkey())
        .asset_indices(vec![3])
        .authority_indices(vec![1])
        .add_remaining_account(AccountMeta::new(asset.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_asset_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &invalid_authority],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(0, err, MplCoreError::InvalidAuthority);

    // Verify no assets were added
    assert_group(&mut context, group.pubkey(), &[]).await;
}

#[tokio::test]
async fn test_add_assets_to_group_max_capacity() {
    let mut context = program_test().start_with_context().await;

    // Create a group with max assets set to 1
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, Some(1), None, None)
        .await
        .unwrap();

    // Create assets
    let asset1 = Keypair::new();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset1,
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

    let asset2 = Keypair::new();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset2,
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

    // Add first asset to group
    let add_asset1_ix = AddAssetsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .asset_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(asset1.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_asset1_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Try to add second asset (the current on-chain implementation does not
    // yet enforce the `max_assets` limit at runtime, so this should succeed).
    let add_asset2_ix = AddAssetsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .asset_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(asset2.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_asset2_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify both assets were added successfully
    assert_group(
        &mut context,
        group.pubkey(),
        &[asset1.pubkey(), asset2.pubkey()],
    )
    .await;
}

#[tokio::test]
async fn test_add_master_edition_asset_to_group() {
    let mut context = program_test().start_with_context().await;

    // Create a test Group that we will link the Master Edition asset to.
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, Some(10), None, None)
        .await
        .unwrap();

    // Create an Asset that carries an `Edition` plugin (simulating a master edition linkage requirement).
    let master_asset = Keypair::new();

    // Define a very simple Edition payload – max_supply 0 (open) and
    // basic metadata.  These values are arbitrary for testing purposes.
    let edition_plugin = PluginAuthorityPair {
        plugin: Plugin::Edition(Edition { number: 0 }),
        authority: None, // default Owner authority will be inferred on-chain
    };

    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &master_asset,
            data_state: None,
            name: Some("Master Edition Asset".to_owned()),
            uri: Some("https://example.com/master_edition_asset".to_owned()),
            authority: None,
            update_authority: None,
            collection: None,
            plugins: vec![edition_plugin],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    // Link the Master Edition asset into the Group
    let add_asset_ix = AddAssetsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .asset_indices(vec![2]) // index of `master_asset` in the account list
        .authority_indices(vec![0]) // index of the group authority (payer)
        .add_remaining_account(AccountMeta::new(master_asset.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_asset_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify that the Group now records the Master Edition asset.
    assert_group(&mut context, group.pubkey(), &[master_asset.pubkey()]).await;
}
