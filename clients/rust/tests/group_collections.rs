//! Tests for adding and removing collections to/from groups.
#![cfg(feature = "test-sbf")]

pub mod setup;
use mpl_core::accounts::GroupV1;
use mpl_core::errors::MplCoreError;
use mpl_core::instructions::{
    AddCollectionsToGroupBuilder, CreateCollectionV2Builder, CreateGroupBuilder,
    RemoveCollectionsFromGroupBuilder,
};

pub use setup::*;

use solana_program::instruction::AccountMeta;
use solana_program::system_program;
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

// Helper function to assert that the expected collections are present in the group
async fn assert_group_collections(
    context: &mut solana_program_test::ProgramTestContext,
    group_pubkey: Pubkey,
    expected_collections: &[Pubkey],
) {
    let group_account = context
        .banks_client
        .get_account(group_pubkey)
        .await
        .expect("get_account")
        .expect("group account not found");

    let group = GroupV1::from_bytes(&group_account.data).unwrap();

    for collection in expected_collections {
        assert!(
            group.collections.contains(collection),
            "Group should contain the collection"
        );
    }

    assert_eq!(
        group.collections.len(),
        expected_collections.len(),
        "Group should have the expected number of collections"
    );
}

#[tokio::test]
async fn test_add_collections_to_group() {
    let mut context = program_test().start_with_context().await;

    // Create a group
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, Some(10), None, None, None)
        .await
        .unwrap();

    // Create two collections
    let collection1 = Keypair::new();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection1,
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

    let collection2 = Keypair::new();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection2,
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

    // Add collections to group
    let add_collections_ix = AddCollectionsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .collection_indices(vec![2, 3])
        .authority_indices(vec![0, 0])
        .add_remaining_accounts(&[
            AccountMeta::new(collection1.pubkey(), false),
            AccountMeta::new(collection2.pubkey(), false),
        ])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_collections_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify collections were added
    assert_group_collections(
        &mut context,
        group.pubkey(),
        &[collection1.pubkey(), collection2.pubkey()],
    )
    .await;
}

#[tokio::test]
async fn test_remove_collections_from_group() {
    let mut context = program_test().start_with_context().await;

    // Create a group
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, Some(10), None, None, None)
        .await
        .unwrap();

    // Create two collections
    let collection1 = Keypair::new();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection1,
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

    let collection2 = Keypair::new();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection2,
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

    // Add both collections to group first
    let add_collections_ix = AddCollectionsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .collection_indices(vec![2, 3])
        .authority_indices(vec![0, 0])
        .add_remaining_accounts(&[
            AccountMeta::new(collection1.pubkey(), false),
            AccountMeta::new(collection2.pubkey(), false),
        ])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_collections_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Now remove the first collection from the group
    let remove_collections_ix = RemoveCollectionsFromGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .collection_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(collection1.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[remove_collections_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify only the second collection remains
    assert_group_collections(&mut context, group.pubkey(), &[collection2.pubkey()]).await;
}

#[tokio::test]
async fn test_add_collections_max_capacity() {
    let mut context = program_test().start_with_context().await;

    // Group with max_collections = 1
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, Some(1), None, None, None)
        .await
        .unwrap();

    // Two collections
    let col1 = Keypair::new();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &col1,
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

    let col2 = Keypair::new();
    create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &col2,
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

    // Add first collection
    let add_col1_ix = AddCollectionsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .collection_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(col1.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_col1_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Try to add second collection exceeding capacity – observe behaviour
    let add_col2_ix = AddCollectionsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .collection_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(col2.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_col2_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let _ = context.banks_client.process_transaction(tx).await;
}

#[tokio::test]
async fn test_fail_add_collections_invalid_authority() {
    let mut context = program_test().start_with_context().await;

    let group = Keypair::new();
    create_test_group(&mut context, &group, None, Some(10), None, None, None)
        .await
        .unwrap();

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

    // Invalid authority signer
    let bad_auth = Keypair::new();
    airdrop(&mut context, &bad_auth.pubkey(), 1_000_000_000)
        .await
        .unwrap();

    let add_col_ix = AddCollectionsToGroupBuilder::new()
        .authority(bad_auth.pubkey())
        .group(group.pubkey())
        .collection_indices(vec![3])
        .authority_indices(vec![1])
        .add_remaining_account(AccountMeta::new(collection.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_col_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &bad_auth],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    assert_custom_instruction_error!(0, err, MplCoreError::InvalidAuthority);
}

// Dual-signature requirement when collection UA != group UA
#[tokio::test]
async fn test_add_remove_collection_with_distinct_authorities() {
    let mut context = program_test().start_with_context().await;

    // Group with payer as update authority.
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, Some(5), None, None, None)
        .await
        .unwrap();

    // Collection with *different* update authority.
    let collection = Keypair::new();
    let collection_ua = Keypair::new();

    // Airdrop collection UA so it can sign.
    airdrop(&mut context, &collection_ua.pubkey(), 1_000_000_000)
        .await
        .unwrap();

    let create_col_ix = CreateCollectionV2Builder::new()
        .collection(collection.pubkey())
        .update_authority(Some(collection_ua.pubkey()))
        .payer(context.payer.pubkey())
        .system_program(system_program::ID)
        .name("Dual Auth Collection".into())
        .uri("https://example.com/dual_col".into())
        .plugins(vec![])
        .external_plugin_adapters(vec![])
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[create_col_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &collection],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Add collection to group – need group UA (payer) and collection UA signatures.
    let add_ix = AddCollectionsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .collection_indices(vec![2])
        .authority_indices(vec![3])
        .add_remaining_accounts(&[
            AccountMeta::new(collection.pubkey(), false),
            AccountMeta::new_readonly(collection_ua.pubkey(), true),
        ])
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &collection_ua],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    assert_group_collections(&mut context, group.pubkey(), &[collection.pubkey()]).await;

    // Remove – again both signers.
    let remove_ix = RemoveCollectionsFromGroupBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .collection_indices(vec![2])
        .authority_indices(vec![3])
        .add_remaining_account(AccountMeta::new(collection.pubkey(), false))
        .add_remaining_account(AccountMeta::new_readonly(collection_ua.pubkey(), true))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[remove_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &collection_ua],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    assert_group_collections(&mut context, group.pubkey(), &[]).await;
}
