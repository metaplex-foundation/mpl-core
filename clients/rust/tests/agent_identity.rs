#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    errors::MplCoreError,
    instructions::{
        AddCollectionExternalPluginAdapterV1Builder, AddExternalPluginAdapterV1Builder,
        RemoveExternalPluginAdapterV1Builder, UpdateExternalPluginAdapterV1Builder,
    },
    types::{
        AgentIdentity, AgentIdentityInitInfo, AgentIdentityUpdateInfo, ExternalCheckResult,
        ExternalPluginAdapter, ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey,
        ExternalPluginAdapterUpdateInfo, HookableLifecycleEvent, PluginAuthority, UpdateAuthority,
    },
};
pub use setup::*;

use solana_program_test::tokio;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

// ---------------------------------------------------------------------------
// Create asset with AgentIdentity
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_create_asset_with_agent_identity() {
    let mut context = program_test().start_with_context().await;

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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::AgentIdentity(
                AgentIdentityInitInfo {
                    uri: String::from("https://example.com/agent.json"),
                    init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                    lifecycle_checks: vec![(
                        HookableLifecycleEvent::Execute,
                        ExternalCheckResult { flags: 4 },
                    )],
                },
            )],
        },
    )
    .await
    .unwrap();

    let owner = context.payer.pubkey();
    let update_authority = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![ExternalPluginAdapter::AgentIdentity(AgentIdentity {
                uri: String::from("https://example.com/agent.json"),
            })],
        },
    )
    .await;
}

// ---------------------------------------------------------------------------
// Add AgentIdentity to an existing asset
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_add_agent_identity() {
    let mut context = program_test().start_with_context().await;

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

    let owner = context.payer.pubkey();
    let update_authority = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;

    let add_ix = AddExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(ExternalPluginAdapterInitInfo::AgentIdentity(
            AgentIdentityInitInfo {
                uri: String::from("https://example.com/agent.json"),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Execute,
                    ExternalCheckResult { flags: 4 },
                )],
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![ExternalPluginAdapter::AgentIdentity(AgentIdentity {
                uri: String::from("https://example.com/agent.json"),
            })],
        },
    )
    .await;
}

// ---------------------------------------------------------------------------
// Cannot add AgentIdentity to a collection
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_cannot_add_agent_identity_to_collection() {
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

    let add_ix = AddCollectionExternalPluginAdapterV1Builder::new()
        .collection(collection.pubkey())
        .payer(context.payer.pubkey())
        .init_info(ExternalPluginAdapterInitInfo::AgentIdentity(
            AgentIdentityInitInfo {
                uri: String::from("https://example.com/agent.json"),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Execute,
                    ExternalCheckResult { flags: 4 },
                )],
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::InvalidPluginAdapterTarget);
}

// ---------------------------------------------------------------------------
// Cannot add duplicate AgentIdentity
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_cannot_add_duplicate_agent_identity() {
    let mut context = program_test().start_with_context().await;

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

    let add_ix0 = AddExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(ExternalPluginAdapterInitInfo::AgentIdentity(
            AgentIdentityInitInfo {
                uri: String::from("https://example.com/agent.json"),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Execute,
                    ExternalCheckResult { flags: 4 },
                )],
            },
        ))
        .instruction();

    let add_ix1 = AddExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(ExternalPluginAdapterInitInfo::AgentIdentity(
            AgentIdentityInitInfo {
                uri: String::from("https://example.com/agent2.json"),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Execute,
                    ExternalCheckResult { flags: 4 },
                )],
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_ix0, add_ix1],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(1, error, MplCoreError::ExternalPluginAdapterAlreadyExists);
}

// ---------------------------------------------------------------------------
// Cannot add AgentIdentity with duplicate lifecycle checks
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_cannot_add_agent_identity_with_duplicate_lifecycle_checks() {
    let mut context = program_test().start_with_context().await;

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

    let add_ix = AddExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .init_info(ExternalPluginAdapterInitInfo::AgentIdentity(
            AgentIdentityInitInfo {
                uri: String::from("https://example.com/agent.json"),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![
                    (
                        HookableLifecycleEvent::Execute,
                        ExternalCheckResult { flags: 4 },
                    ),
                    (
                        HookableLifecycleEvent::Execute,
                        ExternalCheckResult { flags: 4 },
                    ),
                ],
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_ix],
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
}

// ---------------------------------------------------------------------------
// Update AgentIdentity URI
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_update_agent_identity_uri() {
    let mut context = program_test().start_with_context().await;

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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::AgentIdentity(
                AgentIdentityInitInfo {
                    uri: String::from("https://example.com/agent.json"),
                    init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                    lifecycle_checks: vec![(
                        HookableLifecycleEvent::Execute,
                        ExternalCheckResult { flags: 4 },
                    )],
                },
            )],
        },
    )
    .await
    .unwrap();

    let owner = context.payer.pubkey();
    let update_authority = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![ExternalPluginAdapter::AgentIdentity(AgentIdentity {
                uri: String::from("https://example.com/agent.json"),
            })],
        },
    )
    .await;

    let update_ix = UpdateExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::AgentIdentity)
        .update_info(ExternalPluginAdapterUpdateInfo::AgentIdentity(
            AgentIdentityUpdateInfo {
                uri: Some(String::from("https://example.com/agent-v2.json")),
                lifecycle_checks: None,
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![ExternalPluginAdapter::AgentIdentity(AgentIdentity {
                uri: String::from("https://example.com/agent-v2.json"),
            })],
        },
    )
    .await;
}

// ---------------------------------------------------------------------------
// Remove AgentIdentity
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_remove_agent_identity() {
    let mut context = program_test().start_with_context().await;

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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::AgentIdentity(
                AgentIdentityInitInfo {
                    uri: String::from("https://example.com/agent.json"),
                    init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                    lifecycle_checks: vec![(
                        HookableLifecycleEvent::Execute,
                        ExternalCheckResult { flags: 4 },
                    )],
                },
            )],
        },
    )
    .await
    .unwrap();

    let owner = context.payer.pubkey();
    let update_authority = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![ExternalPluginAdapter::AgentIdentity(AgentIdentity {
                uri: String::from("https://example.com/agent.json"),
            })],
        },
    )
    .await;

    let remove_ix = RemoveExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::AgentIdentity)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[remove_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}

// ---------------------------------------------------------------------------
// Non-authority cannot add AgentIdentity
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_non_authority_cannot_add_agent_identity() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();
    let other = Keypair::new();
    airdrop(&mut context, &other.pubkey(), 1_000_000_000)
        .await
        .unwrap();

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

    let add_ix = AddExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(other.pubkey())
        .authority(Some(other.pubkey()))
        .init_info(ExternalPluginAdapterInitInfo::AgentIdentity(
            AgentIdentityInitInfo {
                uri: String::from("https://example.com/agent.json"),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(
                    HookableLifecycleEvent::Execute,
                    ExternalCheckResult { flags: 4 },
                )],
            },
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&other.pubkey()),
        &[&other],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::NoApprovals);
}
