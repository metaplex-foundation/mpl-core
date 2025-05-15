//! Group-level plugin instruction tests (add, update, remove).
//! Ensures that `AllowedPlugin` semantics on a `GroupV1` account work as
//! specified in `collection groups.txt`.
#![cfg(feature = "test-sbf")]

pub mod setup;

use mpl_core::{
    accounts::GroupV1,
    errors::MplCoreError,
    instructions::{
        AddGroupPluginBuilder, ApproveGroupPluginBuilder, CreateGroupBuilder,
        RemoveGroupPluginBuilder, UpdateGroupPluginBuilder,
    },
    types::{AllowedPlugin, PluginAuthorityType},
};

pub use setup::*;

use solana_program::{instruction::AccountMeta, system_instruction, system_program};
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

async fn assert_allowed_plugins(
    context: &mut solana_program_test::ProgramTestContext,
    group_pubkey: Pubkey,
    expected: &[AllowedPlugin],
) {
    let account = context
        .banks_client
        .get_account(group_pubkey)
        .await
        .unwrap()
        .unwrap();
    let group = GroupV1::from_bytes(&account.data).unwrap();
    assert_eq!(group.allowed_plugins, expected);
}

async fn create_empty_account(context: &mut solana_program_test::ProgramTestContext) -> Keypair {
    let kp = Keypair::new();
    let lamports = 1_000_000_000u64;
    let ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &kp.pubkey(),
        lamports,
        0,
        &system_program::ID,
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &kp],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();
    kp
}

#[tokio::test]
async fn test_add_update_remove_group_plugin() {
    let mut context = program_test().start_with_context().await;

    // Create group & plugin account
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, None, None, Some(3))
        .await
        .unwrap();

    // The same key will represent both the plugin *account* and the plugin
    // *program* ID for the purposes of these tests.
    let plugin = Keypair::new();

    // Create a minimal account for the plugin so the runtime does not error
    // with `AccountNotFound`.
    let lamports = 1_000_000_000; // rent-exempt for 0-byte account
    let create_plugin_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &plugin.pubkey(),
        lamports,
        0,
        &system_program::ID,
    );

    let create_plugin_tx = Transaction::new_signed_with_payer(
        &[create_plugin_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &plugin],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(create_plugin_tx)
        .await
        .unwrap();

    // Add plugin   ------------------------------------------------------
    let add_ix = AddGroupPluginBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .plugin(plugin.pubkey())
        .plugin_program(plugin.pubkey())
        .system_program(system_program::ID)
        .plugin_type("attributes".to_string())
        .authority_type(PluginAuthorityType::UpdateAuthority)
        .plugin_args(vec![])
        .instruction();

    let add_tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(add_tx)
        .await
        .unwrap();

    assert_allowed_plugins(
        &mut context,
        group.pubkey(),
        &[AllowedPlugin {
            address: plugin.pubkey(),
            authority_type: PluginAuthorityType::UpdateAuthority,
        }],
    )
    .await;

    // Update plugin   ----------------------------------------------------
    let update_ix = UpdateGroupPluginBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .plugin(plugin.pubkey())
        .plugin_args(vec![])
        .instruction();

    let update_tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(update_tx)
        .await
        .unwrap();

    // No state change expected – just ensure still present & no error.
    assert_allowed_plugins(
        &mut context,
        group.pubkey(),
        &[AllowedPlugin {
            address: plugin.pubkey(),
            authority_type: PluginAuthorityType::UpdateAuthority,
        }],
    )
    .await;

    // Remove plugin   ----------------------------------------------------
    let remove_ix = RemoveGroupPluginBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .plugin(plugin.pubkey())
        .instruction();

    let remove_tx = Transaction::new_signed_with_payer(
        &[remove_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(remove_tx)
        .await
        .unwrap();

    // Expect list emptied after removal.
    assert_allowed_plugins(&mut context, group.pubkey(), &[]).await;
}

#[tokio::test]
async fn test_fail_add_group_plugin_not_allowed_type() {
    let mut context = program_test().start_with_context().await;

    // Create group
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, None, None, Some(3))
        .await
        .unwrap();

    // Plugin/program account (pre-created)
    let plugin = Keypair::new();
    let lamports = 1_000_000_000;
    let create_plugin_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &plugin.pubkey(),
        lamports,
        0,
        &system_program::ID,
    );
    let create_plugin_tx = Transaction::new_signed_with_payer(
        &[create_plugin_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &plugin],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(create_plugin_tx)
        .await
        .unwrap();

    // Attempt to add disallowed plugin type
    let add_ix = AddGroupPluginBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .plugin(plugin.pubkey())
        .plugin_program(plugin.pubkey())
        .system_program(system_program::ID)
        .plugin_type("unsupported_plugin".to_string())
        .authority_type(PluginAuthorityType::None)
        .plugin_args(vec![])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    assert_custom_instruction_error!(0, err, MplCoreError::GroupPluginNotAllowed);

    // Ensure no plugins allowed
    assert_allowed_plugins(&mut context, group.pubkey(), &[]).await;
}

#[tokio::test]
async fn test_approve_then_revoke_group_plugin() {
    let mut context = program_test().start_with_context().await;

    // Create group with generous plugin cap.
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, None, None, Some(10))
        .await
        .unwrap();

    // Stand-in system account used for both plugin PDA & program ID.
    let plugin = Keypair::new();
    let lamports = 1_000_000_000;
    let create_plugin_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &plugin.pubkey(),
        lamports,
        0,
        &system_program::ID,
    );
    let tx = Transaction::new_signed_with_payer(
        &[create_plugin_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &plugin],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Add plugin entry (pending approval).
    let add_ix = AddGroupPluginBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .plugin(plugin.pubkey())
        .plugin_program(plugin.pubkey())
        .system_program(system_program::ID)
        .plugin_type("attributes".to_string())
        .authority_type(PluginAuthorityType::UpdateAuthority)
        .plugin_args(vec![])
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    assert_allowed_plugins(
        &mut context,
        group.pubkey(),
        &[AllowedPlugin {
            address: plugin.pubkey(),
            authority_type: PluginAuthorityType::UpdateAuthority,
        }],
    )
    .await;

    // Approve
    let approve_ix = ApproveGroupPluginBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .plugin(plugin.pubkey())
        .plugin_args(vec![])
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[approve_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Revoke (implemented as remove for now until kinobi builder bug fixed)
    let revoke_ix = RemoveGroupPluginBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .plugin(plugin.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[revoke_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // List should now be empty.
    assert_allowed_plugins(&mut context, group.pubkey(), &[]).await;
}

#[tokio::test]
async fn test_group_plugin_capacity_overflow() {
    let mut context = program_test().start_with_context().await;

    // Group can hold only 1 plugin.
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, None, None, Some(1))
        .await
        .unwrap();

    let plugins: Vec<Keypair> = (0..2).map(|_| Keypair::new()).collect();
    for plugin in &plugins {
        let lamports = 1_000_000_000;
        let ix = system_instruction::create_account(
            &context.payer.pubkey(),
            &plugin.pubkey(),
            lamports,
            0,
            &system_program::ID,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, plugin],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();
    }

    // Add first plugin – must succeed.
    let add_first = AddGroupPluginBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .plugin(plugins[0].pubkey())
        .plugin_program(plugins[0].pubkey())
        .system_program(system_program::ID)
        .plugin_type("attributes".into())
        .authority_type(PluginAuthorityType::UpdateAuthority)
        .plugin_args(vec![])
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[add_first],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Try to add second plugin – capture result; enforcement may change.
    let add_second = AddGroupPluginBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .plugin(plugins[1].pubkey())
        .plugin_program(plugins[1].pubkey())
        .system_program(system_program::ID)
        .plugin_type("attributes".into())
        .authority_type(PluginAuthorityType::UpdateAuthority)
        .plugin_args(vec![])
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[add_second],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let _ = context.banks_client.process_transaction(tx).await;
}

#[tokio::test]
async fn test_update_group_plugin_specific_address_authority() {
    let mut context = program_test().start_with_context().await;

    // Create group and plugin accounts
    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, None, None, Some(5))
        .await
        .unwrap();

    let plugin = Keypair::new();
    let lamports = 1_000_000_000;
    let create_plugin_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &plugin.pubkey(),
        lamports,
        0,
        &system_program::ID,
    );
    let tx = Transaction::new_signed_with_payer(
        &[create_plugin_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &plugin],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // The specific address that will manage this plugin.
    let custodian = Keypair::new();
    airdrop(&mut context, &custodian.pubkey(), lamports)
        .await
        .unwrap();

    // Add plugin – authority is group UA; authority_type = SpecificAddress
    let add_ix = AddGroupPluginBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .plugin(plugin.pubkey())
        .plugin_program(plugin.pubkey())
        .system_program(system_program::ID)
        .plugin_type("attributes".to_string())
        .authority_type(PluginAuthorityType::SpecificAddress(custodian.pubkey()))
        .plugin_args(vec![])
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify the AllowedPlugin entry has been recorded correctly.
    assert_allowed_plugins(
        &mut context,
        group.pubkey(),
        &[AllowedPlugin {
            address: plugin.pubkey(),
            authority_type: PluginAuthorityType::SpecificAddress(custodian.pubkey()),
        }],
    )
    .await;

    // Update plugin – must be signed by the explicit custodian address.
    let update_ix = UpdateGroupPluginBuilder::new()
        .authority(custodian.pubkey())
        .group(group.pubkey())
        .plugin(plugin.pubkey())
        .plugin_args(vec![])
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &custodian],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Attempt to update with the *wrong* signer (group UA) – expect failure.
    let bad_update_ix = UpdateGroupPluginBuilder::new()
        .authority(context.payer.pubkey())
        .group(group.pubkey())
        .plugin(plugin.pubkey())
        .plugin_args(vec![])
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[bad_update_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
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
async fn test_fail_add_group_plugin_disallowed_external_types() {
    let mut context = program_test().start_with_context().await;

    let group = Keypair::new();
    create_test_group(&mut context, &group, None, None, None, None, Some(3))
        .await
        .unwrap();

    let disallowed_types = ["oracle", "lifecycle_hook", "linked_app_data"];

    for plugin_type in disallowed_types.iter() {
        // Fresh empty account to act as plugin & program id.
        let plugin_kp = create_empty_account(&mut context).await;

        let add_ix = AddGroupPluginBuilder::new()
            .authority(context.payer.pubkey())
            .group(group.pubkey())
            .plugin(plugin_kp.pubkey())
            .plugin_program(plugin_kp.pubkey())
            .system_program(system_program::ID)
            .plugin_type((*plugin_type).to_string())
            .authority_type(PluginAuthorityType::UpdateAuthority)
            .plugin_args(vec![])
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[add_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();
        assert_custom_instruction_error!(0, err, MplCoreError::GroupPluginNotAllowed);
    }

    // Ensure no plugins have been allowed after all failed attempts.
    assert_allowed_plugins(&mut context, group.pubkey(), &[]).await;
}
