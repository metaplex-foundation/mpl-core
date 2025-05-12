//! Tests for closing a Group account.
#![cfg(feature = "test-sbf")]

pub mod setup;

use mpl_core::{
    errors::MplCoreError,
    instructions::{
        AddAssetsToGroupBuilder, AddCollectionsToGroupBuilder, AddGroupPluginBuilder,
        AddGroupsToGroupBuilder, CloseGroupBuilder, CreateGroupBuilder,
    },
    types::PluginAuthorityType,
};

pub use setup::*;

use solana_program::instruction::AccountMeta;
use solana_program::{system_instruction, system_program};
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

#[tokio::test]
async fn test_close_empty_group() {
    let mut context = program_test().start_with_context().await;

    // Create a brand-new, empty group.
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, Some(10), None, None)
        .await
        .unwrap();

    // Record payer lamports before closing for later assertion.
    let payer_before = context
        .banks_client
        .get_account(context.payer.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;

    // Close the group.
    let close_ix = CloseGroupBuilder::new()
        .update_authority(context.payer.pubkey())
        .group(group.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Fetch the group account – it should have been truncated to 1 byte and be uninitialized.
    let group_account = context
        .banks_client
        .get_account(group.pubkey())
        .await
        .unwrap()
        .expect("group account not found");

    assert_eq!(
        group_account.data.len(),
        1,
        "Group account should be truncated to 1 byte"
    );
    assert_eq!(
        group_account.data[0],
        0u8, // discriminator for Uninitialized
        "Account discriminator should be Uninitialized",
    );

    // Ensure lamports were returned to the update authority (payer).
    let payer_after = context
        .banks_client
        .get_account(context.payer.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;

    assert!(
        payer_after > payer_before,
        "Payer should receive lamports from the closed account"
    );
}

#[tokio::test]
async fn test_fail_close_non_empty_group() {
    let mut context = program_test().start_with_context().await;

    // Create a fresh group.
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, Some(10), None, None)
        .await
        .unwrap();

    // Create a new asset that will be added to the group.
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

    // Add the asset to the group so that it is no longer empty.
    let add_assets_ix = AddAssetsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .asset_indices(vec![2]) // Index of the asset account in the accounts array.
        .authority_indices(vec![0]) // Index of authority signer (payer) in the accounts array.
        .add_remaining_accounts(&[AccountMeta::new(asset.pubkey(), false)])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_assets_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Attempt to close the now non-empty group – should fail with GroupNotEmpty.
    let close_ix = CloseGroupBuilder::new()
        .update_authority(context.payer.pubkey())
        .group(group.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    assert_custom_instruction_error!(0, error, MplCoreError::GroupNotEmpty);
}

#[tokio::test]
async fn test_fail_close_group_with_child_groups() {
    let mut context = program_test().start_with_context().await;

    // Create parent and child groups
    let parent_group = Keypair::new();
    create_test_group(
        &mut context,
        &parent_group,
        None,
        None,
        Some(10),
        None,
        None,
    )
    .await
    .unwrap();

    let child_group = Keypair::new();
    create_test_group(&mut context, &child_group, None, None, Some(5), None, None)
        .await
        .unwrap();

    // Link child to parent
    let add_groups_ix = AddGroupsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .parent_group(parent_group.pubkey())
        .group_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(child_group.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_groups_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Attempt to close the parent – should fail with GroupNotEmpty
    let close_ix = CloseGroupBuilder::new()
        .update_authority(context.payer.pubkey())
        .group(parent_group.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    assert_custom_instruction_error!(0, err, MplCoreError::GroupNotEmpty);
}

#[tokio::test]
async fn test_fail_close_group_with_collections() {
    let mut context = program_test().start_with_context().await;

    // Create group
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, Some(10), None, None)
        .await
        .unwrap();

    // Create a collection and add to group
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
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    let add_collections_ix = AddCollectionsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .collection_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(collection.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_collections_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Attempt to close – expect GroupNotEmpty
    let close_ix = CloseGroupBuilder::new()
        .update_authority(context.payer.pubkey())
        .group(group.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    assert_custom_instruction_error!(0, err, MplCoreError::GroupNotEmpty);
}

#[tokio::test]
async fn test_fail_close_group_with_parent_groups() {
    let mut context = program_test().start_with_context().await;

    let parent_group = Keypair::new();
    create_test_group(&mut context, &parent_group, None, None, Some(5), None, None)
        .await
        .unwrap();

    let child_group = Keypair::new();
    create_test_group(&mut context, &child_group, None, None, Some(5), None, None)
        .await
        .unwrap();

    // Link child under parent.
    let add_child_ix = AddGroupsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .parent_group(parent_group.pubkey())
        .group_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(child_group.pubkey(), false))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[add_child_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Attempt to close child – should fail.
    let close_ix = CloseGroupBuilder::new()
        .update_authority(context.payer.pubkey())
        .group(child_group.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    assert_custom_instruction_error!(0, err, MplCoreError::GroupNotEmpty);
}

#[tokio::test]
async fn test_fail_close_group_with_allowed_plugins() {
    let mut context = program_test().start_with_context().await;

    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, Some(5), None, None)
        .await
        .unwrap();

    // Create simple plugin account
    let plugin = Keypair::new();
    let lamports = 1_000_000_000;
    let create_plugin_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &plugin.pubkey(),
        lamports,
        0,
        &system_program::ID,
    );
    let tx = Transaction::new_signed_with_payer(
        &[create_plugin_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &plugin],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Add plugin to group (AllowedPlugin entry).
    let add_plugin_ix = AddGroupPluginBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .plugin(plugin.pubkey())
        .plugin_program(plugin.pubkey())
        .system_program(system_program::ID)
        .plugin_type("attributes".to_string())
        .authority_type(PluginAuthorityType::UpdateAuthority)
        .plugin_args(vec![])
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[add_plugin_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let close_ix = CloseGroupBuilder::new()
        .update_authority(context.payer.pubkey())
        .group(group.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    assert_custom_instruction_error!(0, err, MplCoreError::GroupNotEmpty);
}
