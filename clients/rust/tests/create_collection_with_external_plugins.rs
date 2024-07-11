#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    errors::MplCoreError,
    types::{
        ExternalCheckResult, ExternalPluginAdapterInitInfo, HookableLifecycleEvent,
        LifecycleHookInitInfo, OracleInitInfo, PluginAuthority,
    },
};

pub use setup::*;

use solana_program::pubkey;
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_temporarily_cannot_create_lifecycle_hook_on_collection() {
    let mut context = program_test().start_with_context().await;

    let collection = Keypair::new();
    let error = create_collection(
        &mut context,
        CreateCollectionHelperArgs {
            collection: &collection,
            update_authority: None,
            payer: None,
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::LifecycleHook(
                LifecycleHookInitInfo {
                    hooked_program: pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                    init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                    lifecycle_checks: vec![(
                        HookableLifecycleEvent::Transfer,
                        ExternalCheckResult { flags: 1 },
                    )],
                    extra_accounts: None,
                    data_authority: Some(PluginAuthority::UpdateAuthority),
                    schema: None,
                },
            )],
        },
    )
    .await
    .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::NotAvailable);
}

#[tokio::test]
async fn test_create_oracle_on_collection() {
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
}

#[tokio::test]
async fn test_cannot_create_oracle_with_duplicate_lifecycle_checks_on_collection() {
    let mut context = program_test().start_with_context().await;

    let collection = Keypair::new();
    let error = create_collection(
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
                lifecycle_checks: vec![
                    (
                        HookableLifecycleEvent::Transfer,
                        ExternalCheckResult { flags: 4 },
                    ),
                    (
                        HookableLifecycleEvent::Transfer,
                        ExternalCheckResult { flags: 4 },
                    ),
                ],
                base_address_config: None,
                results_offset: None,
            })],
        },
    )
    .await
    .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::DuplicateLifecycleChecks);
}
