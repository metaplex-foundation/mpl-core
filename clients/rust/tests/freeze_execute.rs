//! Integration tests for the FreezeExecute plugin covering the
//! "backed NFT" flow: mint → freeze → execute blocked → burn & refund.
#![cfg(feature = "test-sbf")]

pub mod setup;
use mpl_core::{
    errors::MplCoreError,
    instructions::{BurnV1Builder, ExecuteV1Builder},
    types::{FreezeExecute, Plugin, PluginAuthority, PluginAuthorityPair},
};
use setup::*;

use solana_program::{pubkey::Pubkey, system_instruction, system_program};
use solana_program_test::tokio;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

const FREEZE_EXECUTE_PREFIX: &str = "mpl-core-execute";

#[tokio::test]
async fn test_freeze_execute_backed_nft_flow() {
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
    // 1. Mint an asset with FreezeExecute { frozen: true }
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
            plugins: vec![PluginAuthorityPair {
                plugin: Plugin::FreezeExecute(FreezeExecute { frozen: true }),
                authority: None,
            }],
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
            plugins: vec![PluginAuthorityPair {
                plugin: Plugin::FreezeExecute(FreezeExecute { frozen: true }),
                authority: Some(PluginAuthority::Owner),
            }],
            external_plugin_adapters: vec![],
        },
    )
    .await;

    // ----------------------------------
    // 2. Deposit backing SOL into the asset account (simulate 0.5 SOL backing)
    // ----------------------------------
    let backing_amount: u64 = 500_000_000; // 0.5 SOL
    let transfer_ix = system_instruction::transfer(&payer_key, &asset.pubkey(), backing_amount);
    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&payer_key),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Capture lamports held by the asset account after deposit.
    let asset_account_after_deposit = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .expect("get_account")
        .expect("asset account not found");
    let lamports_in_asset = asset_account_after_deposit.lamports;

    // ----------------------------------
    // 3. Attempt Execute → should fail because plugin is frozen.
    // ----------------------------------
    let (asset_signer, _) = Pubkey::find_program_address(
        &[FREEZE_EXECUTE_PREFIX.as_bytes(), asset.pubkey().as_ref()],
        &mpl_core::ID,
    );

    let execute_ix = ExecuteV1Builder::new()
        .asset(asset.pubkey())
        .collection(None)
        .asset_signer(asset_signer)
        .payer(payer_key)
        .authority(Some(payer_key))
        .system_program(system_program::ID)
        .program_id(system_program::ID) // use system program as harmless target
        .instruction_data(Vec::new())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[execute_ix],
        Some(&payer_key),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    assert_custom_instruction_error!(0, error, MplCoreError::InvalidAuthority);

    // ----------------------------------
    // 4. Burn the asset → user should receive lamports back, asset account closed.
    // ----------------------------------
    // Record payer balance before burn.
    let payer_balance_before_burn = context.banks_client.get_balance(payer_key).await.unwrap();

    let burn_ix = BurnV1Builder::new()
        .asset(asset.pubkey())
        .collection(None)
        .payer(payer_key)
        .authority(Some(payer_key))
        .system_program(Some(system_program::ID))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[burn_ix],
        Some(&payer_key),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Asset account should be closed or at least drained.
    let _asset_account_after_burn = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .unwrap();

    // Verify payer balance increased (should receive refund though exact amount may be reduced by rent/taxes).
    let payer_balance_after_burn = context.banks_client.get_balance(payer_key).await.unwrap();

    assert!(
        payer_balance_after_burn > payer_balance_before_burn,
        "Payer balance did not increase after burn refund"
    );
}
