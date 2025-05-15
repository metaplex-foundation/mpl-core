//! Tests for adding and removing child groups to/from parent groups.
#![cfg(feature = "test-sbf")]

pub mod setup;
use mpl_core::accounts::GroupV1;
use mpl_core::errors::MplCoreError;
use mpl_core::instructions::{
    AddGroupsToGroupBuilder, CreateGroupBuilder, RemoveGroupsFromGroupBuilder,
};

pub use setup::*;

use solana_program::instruction::AccountMeta;
use solana_program::system_program;
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

/// Asserts that `parent_pubkey` contains exactly the `expected_children` and that
/// each child has a back-reference to the parent.
async fn assert_group_relationships(
    context: &mut solana_program_test::ProgramTestContext,
    parent_pubkey: Pubkey,
    expected_children: &[Pubkey],
) {
    // Load and deserialize the parent group.
    let parent_account = context
        .banks_client
        .get_account(parent_pubkey)
        .await
        .expect("get_account")
        .expect("parent group account not found");
    let parent_group = GroupV1::from_bytes(&parent_account.data).unwrap();

    // Validate child pointers.
    for child in expected_children {
        assert!(
            parent_group.child_groups.contains(child),
            "Parent group should reference the child group"
        );

        // Validate back-pointer on child.
        let child_account = context
            .banks_client
            .get_account(*child)
            .await
            .expect("get_account")
            .expect("child group account not found");
        let child_group = GroupV1::from_bytes(&child_account.data).unwrap();
        assert!(
            child_group.parent_groups.contains(&parent_pubkey),
            "Child group should reference its parent"
        );
    }

    // Ensure no extraneous children are present.
    assert_eq!(
        parent_group.child_groups.len(),
        expected_children.len(),
        "Parent group should contain the expected number of children",
    );
}

#[tokio::test]
async fn test_add_groups_to_group() {
    let mut context = program_test().start_with_context().await;

    // Create a parent group.
    let parent_group = Keypair::new();
    create_test_group(
        &mut context,
        &parent_group,
        None,
        None,
        None,
        Some(10),
        None,
    )
    .await
    .unwrap();

    // Create two child groups.
    let child1 = Keypair::new();
    create_test_group(&mut context, &child1, None, None, None, Some(5), None)
        .await
        .unwrap();

    let child2 = Keypair::new();
    create_test_group(&mut context, &child2, None, None, None, Some(5), None)
        .await
        .unwrap();

    // Add the child groups to the parent group.
    let add_groups_ix = AddGroupsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .parent_group(parent_group.pubkey())
        .group_indices(vec![2, 3])
        .authority_indices(vec![0, 0])
        .add_remaining_accounts(&[
            AccountMeta::new(child1.pubkey(), false),
            AccountMeta::new(child2.pubkey(), false),
        ])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_groups_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Validate that the relationship was established on-chain.
    assert_group_relationships(
        &mut context,
        parent_group.pubkey(),
        &[child1.pubkey(), child2.pubkey()],
    )
    .await;
}

#[tokio::test]
async fn test_remove_groups_from_group() {
    let mut context = program_test().start_with_context().await;

    // Create a parent group.
    let parent_group = Keypair::new();
    create_test_group(
        &mut context,
        &parent_group,
        None,
        None,
        None,
        Some(10),
        None,
    )
    .await
    .unwrap();

    // Create two child groups.
    let child1 = Keypair::new();
    create_test_group(&mut context, &child1, None, None, None, Some(5), None)
        .await
        .unwrap();

    let child2 = Keypair::new();
    create_test_group(&mut context, &child2, None, None, None, Some(5), None)
        .await
        .unwrap();

    // First, add both children to the parent group.
    let add_groups_ix = AddGroupsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .parent_group(parent_group.pubkey())
        .group_indices(vec![2, 3])
        .authority_indices(vec![0, 0])
        .add_remaining_accounts(&[
            AccountMeta::new(child1.pubkey(), false),
            AccountMeta::new(child2.pubkey(), false),
        ])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_groups_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Now, remove the first child from the parent group.
    let remove_groups_ix = RemoveGroupsFromGroupBuilder::new()
        .authority(context.payer.pubkey())
        .parent_group(parent_group.pubkey())
        .group_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(child1.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[remove_groups_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Validate that only the second child remains linked.
    assert_group_relationships(&mut context, parent_group.pubkey(), &[child2.pubkey()]).await;
}

#[tokio::test]
async fn test_fail_add_child_groups_invalid_authority() {
    let mut context = program_test().start_with_context().await;

    let parent_group = Keypair::new();
    create_test_group(&mut context, &parent_group, None, None, None, Some(5), None)
        .await
        .unwrap();

    let child_group = Keypair::new();
    create_test_group(&mut context, &child_group, None, None, None, Some(5), None)
        .await
        .unwrap();

    // Invalid authority (not update authority)
    let invalid_auth = Keypair::new();
    airdrop(&mut context, &invalid_auth.pubkey(), 1_000_000_000)
        .await
        .unwrap();

    let add_child_ix = AddGroupsToGroupBuilder::new()
        .authority(invalid_auth.pubkey())
        .parent_group(parent_group.pubkey())
        .group_indices(vec![2])
        .authority_indices(vec![1])
        .add_remaining_account(AccountMeta::new(child_group.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_child_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &invalid_auth],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(0, err, MplCoreError::InvalidAuthority);
}

#[tokio::test]
async fn test_circular_group_reference_prevention() {
    let mut context = program_test().start_with_context().await;

    // Create two groups A and B.
    let group_a = Keypair::new();
    let group_b = Keypair::new();
    create_test_group(&mut context, &group_a, None, None, None, Some(5), None)
        .await
        .unwrap();
    create_test_group(&mut context, &group_b, None, None, None, Some(5), None)
        .await
        .unwrap();

    // Add B as child of A – ok.
    let add_b_to_a_ix = AddGroupsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .parent_group(group_a.pubkey())
        .group_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(group_b.pubkey(), false))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[add_b_to_a_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Attempt to add A as child of B – should error with CircularGroupReference.
    let add_a_to_b_ix = AddGroupsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .parent_group(group_b.pubkey())
        .group_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(group_a.pubkey(), false))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[add_a_to_b_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    assert_custom_instruction_error!(0, err, MplCoreError::CircularGroupReference);
}

#[tokio::test]
async fn test_backpointer_removed_after_child_removed() {
    let mut context = program_test().start_with_context().await;

    let parent = Keypair::new();
    let child = Keypair::new();
    create_test_group(&mut context, &parent, None, None, None, Some(5), None)
        .await
        .unwrap();
    create_test_group(&mut context, &child, None, None, None, Some(5), None)
        .await
        .unwrap();

    // Add child to parent.
    let add_ix = AddGroupsToGroupBuilder::new()
        .authority(context.payer.pubkey())
        .parent_group(parent.pubkey())
        .group_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(child.pubkey(), false))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Remove child.
    let remove_ix = RemoveGroupsFromGroupBuilder::new()
        .authority(context.payer.pubkey())
        .parent_group(parent.pubkey())
        .group_indices(vec![2])
        .authority_indices(vec![0])
        .add_remaining_account(AccountMeta::new(child.pubkey(), false))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[remove_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Load child – ensure it no longer contains parent in parent_groups vec.
    let child_acc = context
        .banks_client
        .get_account(child.pubkey())
        .await
        .unwrap()
        .unwrap();
    let child_state = GroupV1::from_bytes(&child_acc.data).unwrap();
    assert!(!child_state.parent_groups.contains(&parent.pubkey()));
}
