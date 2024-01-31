#![cfg(feature = "test-sbf")]

use borsh::BorshDeserialize;
use mpl_project_name::{accounts::MyAccount, instructions::CreateBuilder};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn create() {
    let mut context = ProgramTest::new("mpl_project_name_program", mpl_project_name::ID, None)
        .start_with_context()
        .await;

    // Given a new keypair.

    let address = Keypair::new();

    let ix = CreateBuilder::new()
        .address(address.pubkey())
        .authority(context.payer.pubkey())
        .payer(context.payer.pubkey())
        .arg1(1)
        .arg2(2)
        .instruction();

    // When we create a new account.

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &address],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then an account was created with the correct data.

    let account = context
        .banks_client
        .get_account(address.pubkey())
        .await
        .unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    assert_eq!(account.data.len(), MyAccount::LEN);

    let mut account_data = account.data.as_ref();
    let my_account = MyAccount::deserialize(&mut account_data).unwrap();
    assert_eq!(my_account.data.field1, 1);
    assert_eq!(my_account.data.field2, 2);
}
