//! Integration tests for the ExecuteV1 instruction.
#![cfg(feature = "test-sbf")]

pub mod setup;
use mpl_core::instructions::ExecuteV1Builder;
use setup::*;

use solana_program::{
    instruction::AccountMeta, pubkey::Pubkey, system_instruction, system_program,
};
use solana_program_test::tokio;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

const EXECUTE_PREFIX: &str = "mpl-core-execute";

#[tokio::test]
async fn test_execute() {
    // ----------------------------------
    // 0. Test setup
    // ----------------------------------
    let mut context = program_test().start_with_context().await;

    // Fund payer so we can deposit SOL to the asset later.
    let payer_key = context.payer.pubkey();
    airdrop(&mut context, &payer_key, 2_000_000_000)
        .await
        .unwrap();

    // ----------------------------------
    // 1. Mint an asset
    // ----------------------------------
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

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner: payer_key,
            update_authority: None,
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;

    // ----------------------------------
    // 2. Deposit backing SOL into the asset account (simulate 0.5 SOL backing)
    // ----------------------------------
    let (asset_signer, _) = Pubkey::find_program_address(
        &[EXECUTE_PREFIX.as_bytes(), asset.pubkey().as_ref()],
        &mpl_core::ID,
    );

    let backing_amount: u64 = 500_000_000; // 0.5 SOL
    let transfer_ix = system_instruction::transfer(&payer_key, &asset_signer, backing_amount);
    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&payer_key),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Capture lamports held by the asset account after deposit.
    let lamports_in_asset = context
        .banks_client
        .get_balance(asset_signer)
        .await
        .expect("get_balance failed");

    // Verify that the deposit was successful.
    assert_eq!(lamports_in_asset, backing_amount);

    // ----------------------------------
    // 3. Execute → Transfer funds from asset_signer to payer.
    // ----------------------------------
    let execute_transfer_ix =
        system_instruction::transfer(&asset_signer, &payer_key, lamports_in_asset);

    let execute_ix = ExecuteV1Builder::new()
        .asset(asset.pubkey())
        .collection(None)
        .asset_signer(asset_signer)
        .payer(payer_key)
        .authority(Some(payer_key))
        .system_program(system_program::ID)
        .program_id(system_program::ID) // use system program as harmless target
        .instruction_data(execute_transfer_ix.data)
        .add_remaining_accounts(&[
            AccountMeta {
                pubkey: asset_signer,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: payer_key,
                is_signer: false,
                is_writable: true,
            },
        ])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[execute_ix],
        Some(&payer_key),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify that the transfer was successful.
    let lamports_in_asset_after_execute = context
        .banks_client
        .get_balance(asset_signer)
        .await
        .expect("get_balance failed");

    // Verify that the transfer was successful.
    assert_eq!(lamports_in_asset_after_execute, 0);
}
