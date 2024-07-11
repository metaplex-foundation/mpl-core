#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    errors::MplCoreError,
    instructions::UpdateCollectionExternalPluginAdapterV1Builder,
    types::{
        ExternalCheckResult, ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey,
        ExternalPluginAdapterUpdateInfo, HookableLifecycleEvent, OracleInitInfo, OracleUpdateInfo,
        PluginAuthority, ValidationResultsOffset,
    },
};
pub use setup::*;

use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

#[tokio::test]
async fn test_update_oracle_on_collection() {
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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
                base_address: Pubkey::default(),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Transfer,
                    ExternalCheckResult { flags: 4 },
                )],
                base_address_config: None,
                results_offset: None,
            })],
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
        },
    )
    .await;

    let ix = UpdateCollectionExternalPluginAdapterV1Builder::new()
        .collection(collection.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::Oracle(Pubkey::default()))
        .update_info(ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
            lifecycle_checks: None,
            base_address_config: None,
            results_offset: Some(ValidationResultsOffset::Custom(10)),
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

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
        },
    )
    .await;
}

#[tokio::test]
async fn test_cannot_update_oracle_to_have_duplicate_lifecycle_checks_on_collection() {
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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
                base_address: Pubkey::default(),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Transfer,
                    ExternalCheckResult { flags: 4 },
                )],
                base_address_config: None,
                results_offset: None,
            })],
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
        },
    )
    .await;

    let ix = UpdateCollectionExternalPluginAdapterV1Builder::new()
        .collection(collection.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::Oracle(Pubkey::default()))
        .update_info(ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
            lifecycle_checks: Some(vec![
                (
                    HookableLifecycleEvent::Transfer,
                    ExternalCheckResult { flags: 4 },
                ),
                (
                    HookableLifecycleEvent::Transfer,
                    ExternalCheckResult { flags: 4 },
                ),
            ]),
            base_address_config: None,
            results_offset: Some(ValidationResultsOffset::Custom(10)),
        }))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::DuplicateLifecycleChecks);

    // TODO: Check that registry record did not get updated.
}
