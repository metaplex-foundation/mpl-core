//! Tests for updating group metadata
#![cfg(feature = "test-sbf")]

pub mod setup;
use mpl_core::{
    accounts::GroupV1,
    errors::MplCoreError,
    instructions::{CreateGroupBuilder, UpdateGroupMetadataBuilder},
};
pub use setup::*;

use solana_program::system_program;
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

/// Assert the metadata of a Group matches the expected values.
async fn assert_group_metadata(
    context: &mut solana_program_test::ProgramTestContext,
    group_pubkey: Pubkey,
    expected_name: &str,
    expected_uri: &str,
    expected_update_authority: Pubkey,
) {
    let group_account = context
        .banks_client
        .get_account(group_pubkey)
        .await
        .expect("get_account")
        .expect("group account not found");

    let group = GroupV1::from_bytes(&group_account.data).unwrap();

    assert_eq!(group.name, expected_name);
    assert_eq!(group.uri, expected_uri);
    assert_eq!(group.update_authority, expected_update_authority);
}

#[tokio::test]
async fn test_update_group_metadata_success() {
    let mut context = program_test().start_with_context().await;

    // Create a new group owned by the test payer.
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, None, None, None)
        .await
        .unwrap();

    // Prepare new metadata.
    let new_name = "Updated Group".to_string();
    let new_uri = "https://example.com/updated_group".to_string();
    let new_update_authority = Keypair::new();

    // Build and send the `update_group_metadata` instruction.
    let update_ix = UpdateGroupMetadataBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .name(new_name.clone())
        .uri(new_uri.clone())
        .new_update_authority(new_update_authority.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify on-chain state matches the expected updates.
    assert_group_metadata(
        &mut context,
        group.pubkey(),
        &new_name,
        &new_uri,
        new_update_authority.pubkey(),
    )
    .await;
}

#[tokio::test]
async fn test_update_group_metadata_invalid_authority() {
    let mut context = program_test().start_with_context().await;

    // Create a new group owned by the test payer.
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, None, None, None)
        .await
        .unwrap();

    // Attempt to update metadata using an invalid authority.
    let invalid_authority = Keypair::new();
    airdrop(&mut context, &invalid_authority.pubkey(), 1_000_000_000)
        .await
        .unwrap();

    let update_ix = UpdateGroupMetadataBuilder::new()
        .authority(invalid_authority.pubkey())
        .group(group.pubkey())
        .name("Should Fail".to_string())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &invalid_authority],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Ensure the transaction failed with the expected custom error.
    assert_custom_instruction_error!(0, err, MplCoreError::InvalidAuthority);

    // Verify that metadata has not changed.
    let payer_pubkey = context.payer.pubkey();
    assert_group_metadata(
        &mut context,
        group.pubkey(),
        "Test Group",
        "https://example.com/group",
        payer_pubkey,
    )
    .await;
}
