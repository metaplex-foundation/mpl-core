#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    errors::MplCoreError, instructions::UpdateCollectionInfoV1Builder, types::UpdateType,
};
pub use setup::*;

use solana_program::{pubkey::Pubkey, system_instruction::create_account};
use solana_program_test::tokio;
use solana_sdk::{pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

pub const BUBBLEGUM_PROGRAM_ADDRESS: Pubkey =
    pubkey!("BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY");

#[tokio::test]
async fn test_cannot_update_collection_info_with_incorrect_signer() {
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
            external_plugin_adapters: vec![],
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
            external_plugin_adapters: vec![],
        },
    )
    .await;

    // Create a fake Bubblegum signer which is a normal keypair owned by the Bubblegum
    // program.
    let fake_bubblegum_signer = Keypair::new();
    let lamports = context
        .banks_client
        .get_rent()
        .await
        .unwrap()
        .minimum_balance(0);

    let ix = create_account(
        &context.payer.pubkey(),
        &fake_bubblegum_signer.pubkey(),
        lamports,
        0,
        &BUBBLEGUM_PROGRAM_ADDRESS,
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &fake_bubblegum_signer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify the fake Bubblegum signer is owned by the Bubblegum program.
    let fake_bubblegum_signer_account = context
        .banks_client
        .get_account(fake_bubblegum_signer.pubkey())
        .await
        .expect("get_account")
        .expect("fake Bubblegum signer account not found");

    assert_eq!(
        fake_bubblegum_signer_account.owner,
        BUBBLEGUM_PROGRAM_ADDRESS
    );
    assert_eq!(fake_bubblegum_signer_account.data.len(), 0);

    // Verify that calling `UpdateCollectionInfoV1` with the fake signer results in an error.
    let ix = UpdateCollectionInfoV1Builder::new()
        .collection(collection.pubkey())
        .bubblegum_signer(fake_bubblegum_signer.pubkey())
        .update_type(UpdateType::Mint)
        .amount(1)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &fake_bubblegum_signer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::InvalidAuthority);
}
