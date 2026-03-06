#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    errors::MplCoreError,
    instructions::{
        AddCollectionExternalPluginAdapterV1Builder, AddExternalPluginAdapterV1Builder,
        ExecuteV1Builder, RemoveExternalPluginAdapterV1Builder,
        UpdateExternalPluginAdapterV1Builder,
    },
    types::{
        AgentIdentity, AgentIdentityInitInfo, AgentIdentityUpdateInfo, ExternalCheckResult,
        ExternalPluginAdapter, ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey,
        ExternalPluginAdapterUpdateInfo, HookableLifecycleEvent, PluginAuthority, UpdateAuthority,
    },
};
pub use setup::*;

use mpl_core::PluginRegistryV1Safe;
use solana_program::{pubkey::Pubkey, system_instruction, system_program};
use solana_program_test::tokio;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

const EXECUTE_PREFIX: &str = "mpl-core-execute";

/// Read the external plugin registry for an asset and assert the lifecycle check
/// flags for the first external plugin adapter's given event discriminant.
async fn assert_external_plugin_lifecycle_flags(
    context: &mut solana_program_test::ProgramTestContext,
    asset_pubkey: Pubkey,
    event_discriminant: u8,
    expected_flags: u32,
) {
    let account = context
        .banks_client
        .get_account(asset_pubkey)
        .await
        .unwrap()
        .unwrap();

    let asset = mpl_core::Asset::from_bytes(&account.data).unwrap();
    let header = asset.plugin_header.as_ref().unwrap();
    let registry =
        PluginRegistryV1Safe::from_bytes(&account.data[header.plugin_registry_offset as usize..])
            .unwrap();

    assert_eq!(registry.external_registry.len(), 1);
    let record = &registry.external_registry[0];
    let checks = record.lifecycle_checks.as_ref().unwrap();
    let check = checks
        .iter()
        .find(|(event, _)| *event == event_discriminant)
        .expect("Expected lifecycle event not found in registry");
    assert_eq!(
        check.1.flags, expected_flags,
        "lifecycle check flags mismatch: expected {expected_flags}, got {}",
        check.1.flags
    );
}

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
// Cannot create a collection with an AgentIdentity
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_cannot_create_collection_with_agent_identity() {
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
            external_plugin_adapters: vec![ExternalPluginAdapterInitInfo::AgentIdentity(
                AgentIdentityInitInfo {
                    uri: String::from("https://example.com/agent.json"),
                    init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                    lifecycle_checks: vec![(
                        HookableLifecycleEvent::Execute,
                        ExternalCheckResult { flags: 1 }, // CAN_LISTEN
                    )],
                },
            )],
        },
    )
    .await
    .unwrap_err();

    assert_custom_instruction_error!(0, error, MplCoreError::InvalidPluginAdapterTarget);
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

// ---------------------------------------------------------------------------
// Update AgentIdentity lifecycle checks
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_update_agent_identity_lifecycle_checks() {
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
                        ExternalCheckResult { flags: 1 }, // CAN_LISTEN
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

    // Assert initial flags: Execute(4) = CAN_LISTEN(1)
    assert_external_plugin_lifecycle_flags(&mut context, asset.pubkey(), 4, 1).await;

    // Update lifecycle checks from CAN_LISTEN to CAN_LISTEN | CAN_APPROVE
    let update_ix = UpdateExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::AgentIdentity)
        .update_info(ExternalPluginAdapterUpdateInfo::AgentIdentity(
            AgentIdentityUpdateInfo {
                uri: None,
                lifecycle_checks: Some(vec![(
                    HookableLifecycleEvent::Execute,
                    ExternalCheckResult { flags: 3 }, // CAN_LISTEN | CAN_APPROVE
                )]),
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

    // URI should remain unchanged
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

    // Assert updated flags: Execute(4) = CAN_LISTEN | CAN_APPROVE(3)
    assert_external_plugin_lifecycle_flags(&mut context, asset.pubkey(), 4, 3).await;
}

// ---------------------------------------------------------------------------
// Update AgentIdentity URI and lifecycle checks simultaneously
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_update_agent_identity_uri_and_lifecycle_checks() {
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
                        ExternalCheckResult { flags: 1 }, // CAN_LISTEN
                    )],
                },
            )],
        },
    )
    .await
    .unwrap();

    // Update both URI and lifecycle checks at once
    let update_ix = UpdateExternalPluginAdapterV1Builder::new()
        .asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .key(ExternalPluginAdapterKey::AgentIdentity)
        .update_info(ExternalPluginAdapterUpdateInfo::AgentIdentity(
            AgentIdentityUpdateInfo {
                uri: Some(String::from("https://example.com/agent-v3.json")),
                lifecycle_checks: Some(vec![(
                    HookableLifecycleEvent::Execute,
                    ExternalCheckResult { flags: 7 }, // CAN_LISTEN | CAN_APPROVE | CAN_REJECT
                )]),
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
                uri: String::from("https://example.com/agent-v3.json"),
            })],
        },
    )
    .await;

    // Assert updated flags: Execute(4) = CAN_LISTEN | CAN_APPROVE | CAN_REJECT(7)
    assert_external_plugin_lifecycle_flags(&mut context, asset.pubkey(), 4, 7).await;
}

// ---------------------------------------------------------------------------
// Execute on asset with AgentIdentity (CAN_LISTEN)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_execute_with_agent_identity() {
    let mut context = program_test().start_with_context().await;

    let payer_key = context.payer.pubkey();
    airdrop(&mut context, &payer_key, 2_000_000_000)
        .await
        .unwrap();

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
                        ExternalCheckResult { flags: 1 }, // CAN_LISTEN
                    )],
                },
            )],
        },
    )
    .await
    .unwrap();

    // Fund the asset signer PDA so the system transfer has something to send.
    let (asset_signer, _) = Pubkey::find_program_address(
        &[EXECUTE_PREFIX.as_bytes(), asset.pubkey().as_ref()],
        &mpl_core::ID,
    );

    airdrop(&mut context, &asset_signer, 1_000_000_000)
        .await
        .unwrap();

    let recipient = Keypair::new();

    // Build a system transfer instruction from the asset signer PDA to a recipient.
    let inner_transfer =
        system_instruction::transfer(&asset_signer, &recipient.pubkey(), 500_000_000);

    let execute_ix = ExecuteV1Builder::new()
        .asset(asset.pubkey())
        .collection(None)
        .asset_signer(asset_signer)
        .payer(payer_key)
        .authority(Some(payer_key))
        .system_program(system_program::ID)
        .program_id(system_program::ID)
        .instruction_data(inner_transfer.data.clone())
        .add_remaining_accounts(
            &inner_transfer
                .accounts
                .iter()
                .map(|a| solana_program::instruction::AccountMeta {
                    pubkey: a.pubkey,
                    is_signer: if a.pubkey == asset_signer {
                        false // signer is injected by CPI
                    } else {
                        a.is_signer
                    },
                    is_writable: a.is_writable,
                })
                .collect::<Vec<_>>(),
        )
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[execute_ix],
        Some(&payer_key),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify the recipient received funds.
    let recipient_balance = context
        .banks_client
        .get_balance(recipient.pubkey())
        .await
        .unwrap();
    assert_eq!(recipient_balance, 500_000_000);
}
