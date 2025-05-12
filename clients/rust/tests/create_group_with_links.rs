//! Tests for creating a `GroupV1` with initial collections and parent groups.
//! Covers atomic graph linking paths described in `collection groups.txt`.
#![cfg(feature = "test-sbf")]

pub mod setup;

use mpl_core::accounts::GroupV1;
pub use setup::*;
use solana_program::{instruction::AccountMeta, system_program};
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

use mpl_core::instructions::CreateGroupBuilder;

async fn assert_relationships(
    context: &mut solana_program_test::ProgramTestContext,
    group: Pubkey,
    collection: Pubkey,
    parent_group: Pubkey,
) {
    // Freshly fetched group state
    let group_state = {
        let acc = context
            .banks_client
            .get_account(group)
            .await
            .unwrap()
            .unwrap();
        GroupV1::from_bytes(&acc.data).unwrap()
    };
    assert!(group_state.collections.contains(&collection));
    assert!(group_state.parent_groups.contains(&parent_group));

    // Parent group must reference its child.
    let parent_state = {
        let acc = context
            .banks_client
            .get_account(parent_group)
            .await
            .unwrap()
            .unwrap();
        GroupV1::from_bytes(&acc.data).unwrap()
    };
    assert!(parent_state.child_groups.contains(&group));
}

#[tokio::test]
async fn test_create_group_with_initial_links() {
    let mut context = program_test().start_with_context().await;

    // Parent group
    let parent_group = Keypair::new();
    // Reuse simplified inline group creation helper so this test is self-contained.
    {
        let mut ix = CreateGroupBuilder::new()
            .payer(context.payer.pubkey())
            .group(parent_group.pubkey())
            .update_authority(context.payer.pubkey())
            .system_program(system_program::ID)
            .name("Parent Group".to_owned())
            .uri("https://example.com/parent".to_owned())
            .max_collections(5)
            .max_assets(5)
            .max_groups(5)
            .max_plugins(0)
            .initial_collections(vec![])
            .initial_parent_groups(vec![])
            .initial_assets(vec![])
            .instruction();

        if let Some(meta) = ix.accounts.get_mut(1) {
            meta.is_signer = true;
        }

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &parent_group],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();
    }

    // Collection
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

    // Now create a *new* group that links the above collection & parent.
    let new_group = Keypair::new();

    let mut create_ix = CreateGroupBuilder::new()
        .payer(context.payer.pubkey())
        .group(new_group.pubkey())
        .update_authority(context.payer.pubkey())
        .system_program(system_program::ID)
        .name("Linked Group".to_owned())
        .uri("https://example.com/linked_group".to_owned())
        .max_collections(2)
        .max_assets(2)
        .max_groups(2)
        .max_plugins(0)
        .initial_collections(vec![collection.pubkey()])
        .initial_parent_groups(vec![parent_group.pubkey()])
        .initial_assets(vec![])
        // Accounts after the core 4:
        //   [collection, collection_update_auth] + [parent_group, parent_update_auth]
        .add_remaining_accounts(&[
            AccountMeta::new(collection.pubkey(), false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
            AccountMeta::new(parent_group.pubkey(), false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .instruction();

    // Group account must be signer for creation via system program.
    if let Some(meta) = create_ix.accounts.get_mut(1) {
        meta.is_signer = true;
    }

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &new_group],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    assert_relationships(
        &mut context,
        new_group.pubkey(),
        collection.pubkey(),
        parent_group.pubkey(),
    )
    .await;
}
