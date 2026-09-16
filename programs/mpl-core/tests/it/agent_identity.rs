//! The `AgentIdentity` external plugin adapter.
//!
//! The adapter may only live on an asset (never a collection), at most once,
//! and adding it requires the asset's agent identity PDA under
//! mpl-agent-identity to sign. Mollusk performs no signature verification, so
//! the PDA is passed with `is_signer = true` and the program's
//! `assert_signer` / `assert_derivation` checks see exactly what they would
//! on-chain after a CPI from mpl-agent-identity. There is no JS equivalent of
//! these tests.

use {
    crate::common::{
        accounts::{DEFAULT_ASSET_NAME, DEFAULT_COLLECTION_NAME, DEFAULT_URI},
        *,
    },
    mpl_core::types as client,
    mpl_core_program::{
        error::MplCoreError,
        plugins::{
            AgentIdentity, AgentIdentityInitInfo, AgentIdentityUpdateInfo, ExternalCheckResult,
            ExternalPluginAdapter, ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey,
            ExternalPluginAdapterType, ExternalPluginAdapterUpdateInfo, HookableLifecycleEvent,
        },
        state::{Authority, UpdateAuthority},
    },
    solana_account::Account,
    solana_program::{instruction::AccountMeta, program_error::ProgramError, pubkey::Pubkey},
};

/// URI the adapter is created with.
const AGENT_URI: &str = "https://example.com/agent.json";

/// `ExternalCheckResult` flag bits.
const CAN_LISTEN: u32 = 0x1;
const CAN_APPROVE: u32 = 0x2;
const CAN_REJECT: u32 = 0x4;

/// A lifecycle check on `Execute` with the given flags.
fn execute_check(flags: u32) -> (HookableLifecycleEvent, ExternalCheckResult) {
    (
        HookableLifecycleEvent::Execute,
        ExternalCheckResult { flags },
    )
}

/// The client-side init info for an `AgentIdentity` adapter.
fn init_info(
    uri: &str,
    init_plugin_authority: Option<Authority>,
    lifecycle_checks: Vec<(HookableLifecycleEvent, ExternalCheckResult)>,
) -> client::ExternalPluginAdapterInitInfo {
    convert(&ExternalPluginAdapterInitInfo::AgentIdentity(
        AgentIdentityInitInfo {
            uri: uri.to_string(),
            init_plugin_authority,
            lifecycle_checks,
        },
    ))
}

/// Init info with [`AGENT_URI`], the default (update) authority and a single
/// `Execute` check with `CAN_LISTEN`.
fn default_init_info() -> client::ExternalPluginAdapterInitInfo {
    init_info(AGENT_URI, None, vec![execute_check(CAN_LISTEN)])
}

/// The client-side update info for an `AgentIdentity` adapter.
fn update_info(
    uri: Option<&str>,
    lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
) -> client::ExternalPluginAdapterUpdateInfo {
    convert(&ExternalPluginAdapterUpdateInfo::AgentIdentity(
        AgentIdentityUpdateInfo {
            uri: uri.map(str::to_string),
            lifecycle_checks,
        },
    ))
}

/// An asset owned by `owner` that already carries the default adapter (see
/// [`default_init_info`]).
fn asset_with_agent_identity(owner: Pubkey) -> Account {
    AssetSpec::new(owner)
        .adapter(ExternalAdapterSpec::agent_identity(
            AGENT_URI,
            vec![execute_check(CAN_LISTEN)],
        ))
        .build()
}

/// The remaining account that carries the agent identity PDA as a signer.
fn pda_signer(pda: Pubkey) -> AccountMeta {
    AccountMeta::new_readonly(pda, true)
}

/// Asserts the account holds exactly one `AgentIdentity` adapter with the
/// given URI, authority and lifecycle checks, and a consistent registry.
fn assert_agent_identity(
    account: &Account,
    uri: &str,
    authority: Authority,
    lifecycle_checks: &[(HookableLifecycleEvent, ExternalCheckResult)],
) {
    assert_registry_consistent(account);
    let parsed = parse_asset(&account.data).1;
    assert_eq!(
        parsed.adapters.len(),
        1,
        "expected exactly one external adapter"
    );
    let (record, adapter, data) = read_adapter(account, &ExternalPluginAdapterKey::AgentIdentity)
        .expect("the AgentIdentity adapter should be in the registry");
    assert_eq!(record.plugin_type, ExternalPluginAdapterType::AgentIdentity);
    assert_eq!(record.authority, authority);
    assert_eq!(record.lifecycle_checks.as_deref(), Some(lifecycle_checks));
    assert_eq!(
        adapter,
        ExternalPluginAdapter::AgentIdentity(AgentIdentity {
            uri: uri.to_string()
        })
    );
    assert!(data.is_none(), "AgentIdentity carries no data");
}

// ===========================================================================
// Happy paths
// ===========================================================================

#[test]
fn create_asset_with_agent_identity() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (pda, _) = agent_identity_pda(&asset);

    let ix = ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        Some(vec![default_init_info()]),
        &[pda_signer(pda)],
    );
    let accounts = with_program_accounts(vec![
        (asset, empty_account()),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
        (pda, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    assert_ok(&result);

    let created = account_of(&result, &asset);
    let core = read_asset(created);
    assert_eq!(core.owner, payer);
    assert_eq!(core.update_authority, UpdateAuthority::Address(payer));
    assert_agent_identity(
        created,
        AGENT_URI,
        Authority::UpdateAuthority,
        &[execute_check(CAN_LISTEN)],
    );
}

#[test]
fn add_agent_identity_to_existing_asset() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (pda, _) = agent_identity_pda(&asset);

    let ix = with_remaining(
        ix::add_external_plugin_adapter_v1(
            asset,
            None,
            payer,
            Some(payer),
            None,
            default_init_info(),
        ),
        [pda_signer(pda)],
    );
    let accounts = with_program_accounts(vec![
        (asset, AssetSpec::new(payer).build()),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
        (pda, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    assert_ok(&result);
    assert_agent_identity(
        account_of(&result, &asset),
        AGENT_URI,
        Authority::UpdateAuthority,
        &[execute_check(CAN_LISTEN)],
    );
}

#[test]
fn update_agent_identity_uri() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_uri = "https://example.com/updated-agent.json";

    let ix = ix::update_external_plugin_adapter_v1(
        asset,
        None,
        payer,
        Some(payer),
        None,
        client::ExternalPluginAdapterKey::AgentIdentity,
        update_info(Some(new_uri), None),
    );
    let accounts = with_program_accounts(vec![
        (asset, asset_with_agent_identity(payer)),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    assert_ok(&result);
    // The URI changes; the lifecycle checks are untouched.
    assert_agent_identity(
        account_of(&result, &asset),
        new_uri,
        Authority::UpdateAuthority,
        &[execute_check(CAN_LISTEN)],
    );
}

#[test]
fn update_agent_identity_lifecycle_checks() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_checks = vec![execute_check(CAN_LISTEN | CAN_APPROVE)];

    let ix = ix::update_external_plugin_adapter_v1(
        asset,
        None,
        payer,
        Some(payer),
        None,
        client::ExternalPluginAdapterKey::AgentIdentity,
        update_info(None, Some(new_checks.clone())),
    );
    let accounts = with_program_accounts(vec![
        (asset, asset_with_agent_identity(payer)),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    assert_ok(&result);
    // The checks change; the URI is untouched.
    assert_agent_identity(
        account_of(&result, &asset),
        AGENT_URI,
        Authority::UpdateAuthority,
        &new_checks,
    );
}

#[test]
fn update_agent_identity_uri_and_lifecycle_checks() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_uri = "https://example.com/agent-v3.json";
    let new_checks = vec![execute_check(CAN_LISTEN | CAN_APPROVE | CAN_REJECT)];

    let ix = ix::update_external_plugin_adapter_v1(
        asset,
        None,
        payer,
        Some(payer),
        None,
        client::ExternalPluginAdapterKey::AgentIdentity,
        update_info(Some(new_uri), Some(new_checks.clone())),
    );
    let accounts = with_program_accounts(vec![
        (asset, asset_with_agent_identity(payer)),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    assert_ok(&result);
    assert_agent_identity(
        account_of(&result, &asset),
        new_uri,
        Authority::UpdateAuthority,
        &new_checks,
    );
}

#[test]
fn remove_agent_identity() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();

    let ix = ix::remove_external_plugin_adapter_v1(
        asset,
        None,
        payer,
        Some(payer),
        None,
        client::ExternalPluginAdapterKey::AgentIdentity,
    );
    let accounts = with_program_accounts(vec![
        (asset, asset_with_agent_identity(payer)),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    assert_ok(&result);

    let removed = account_of(&result, &asset);
    assert_registry_consistent(removed);
    assert!(
        read_adapter(removed, &ExternalPluginAdapterKey::AgentIdentity).is_none(),
        "the adapter should be gone from the registry"
    );
    let parsed = parse_asset(&removed.data).1;
    assert!(parsed.has_meta(), "the (empty) header and registry remain");
    assert!(parsed.plugins.is_empty() && parsed.adapters.is_empty());
}

#[test]
fn create_asset_with_agent_identity_multiple_lifecycle_checks() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (pda, _) = agent_identity_pda(&asset);
    let checks = vec![execute_check(CAN_LISTEN | CAN_APPROVE)];

    let ix = ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        Some(vec![init_info(AGENT_URI, None, checks.clone())]),
        &[pda_signer(pda)],
    );
    let accounts = with_program_accounts(vec![
        (asset, empty_account()),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
        (pda, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    assert_ok(&result);
    assert_agent_identity(
        account_of(&result, &asset),
        AGENT_URI,
        Authority::UpdateAuthority,
        &checks,
    );
}

#[test]
fn create_asset_with_agent_identity_address_authority() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (pda, _) = agent_identity_pda(&asset);
    let plugin_authority = Authority::Address {
        address: Pubkey::new_unique(),
    };

    let ix = ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        Some(vec![init_info(
            AGENT_URI,
            Some(plugin_authority),
            vec![execute_check(CAN_LISTEN)],
        )]),
        &[pda_signer(pda)],
    );
    let accounts = with_program_accounts(vec![
        (asset, empty_account()),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
        (pda, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    assert_ok(&result);
    assert_agent_identity(
        account_of(&result, &asset),
        AGENT_URI,
        plugin_authority,
        &[execute_check(CAN_LISTEN)],
    );
}

// ===========================================================================
// Negative / security cases
// ===========================================================================

#[test]
fn cannot_create_collection_with_agent_identity() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let collection = Pubkey::new_unique();

    let ix = ix::create_collection_v2(
        collection,
        None,
        payer,
        DEFAULT_COLLECTION_NAME,
        DEFAULT_URI,
        None,
        Some(vec![default_init_info()]),
        &[],
    );
    let accounts = with_program_accounts(vec![
        (collection, empty_account()),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    // `validate_create` rejects the adapter when there is no asset.
    assert_core_err(&result, MplCoreError::InvalidPluginAdapterTarget);
}

#[test]
fn cannot_add_agent_identity_to_collection() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let collection = Pubkey::new_unique();

    let ix = ix::add_collection_external_plugin_adapter_v1(
        collection,
        payer,
        Some(payer),
        None,
        default_init_info(),
    );
    let accounts = with_program_accounts(vec![
        (collection, CollectionSpec::new(payer).build()),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    // `validate_add_external_plugin_adapter` rejects the adapter on a collection.
    assert_core_err(&result, MplCoreError::InvalidPluginAdapterTarget);
}

#[test]
fn cannot_add_agent_identity_without_pda_remaining_account() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();

    // No remaining account: the last account is the log wrapper placeholder,
    // which is not a signer.
    let ix = ix::add_external_plugin_adapter_v1(
        asset,
        None,
        payer,
        Some(payer),
        None,
        default_init_info(),
    );
    let accounts = with_program_accounts(vec![
        (asset, AssetSpec::new(payer).build()),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    assert_program_err(&result, ProgramError::MissingRequiredSignature);
}

#[test]
fn cannot_add_agent_identity_with_wrong_pda() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    // A PDA derived for a different asset.
    let (wrong_pda, _) = agent_identity_pda(&Pubkey::new_unique());

    let ix = with_remaining(
        ix::add_external_plugin_adapter_v1(
            asset,
            None,
            payer,
            Some(payer),
            None,
            default_init_info(),
        ),
        [pda_signer(wrong_pda)],
    );
    let accounts = with_program_accounts(vec![
        (asset, AssetSpec::new(payer).build()),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
        (wrong_pda, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    // The signer check passes, the derivation check does not.
    assert_core_err(&result, MplCoreError::AgentIdentityMustSign);
}

#[test]
fn cannot_add_agent_identity_with_unsigned_pda() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (pda, _) = agent_identity_pda(&asset);

    // The right PDA, but not marked as a signer.
    let ix = with_remaining(
        ix::add_external_plugin_adapter_v1(
            asset,
            None,
            payer,
            Some(payer),
            None,
            default_init_info(),
        ),
        [AccountMeta::new_readonly(pda, false)],
    );
    let accounts = with_program_accounts(vec![
        (asset, AssetSpec::new(payer).build()),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
        (pda, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    assert_program_err(&result, ProgramError::MissingRequiredSignature);
}

#[test]
fn cannot_add_duplicate_agent_identity() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (pda, _) = agent_identity_pda(&asset);

    let ix = with_remaining(
        ix::add_external_plugin_adapter_v1(
            asset,
            None,
            payer,
            Some(payer),
            None,
            init_info(
                "https://example.com/agent2.json",
                None,
                vec![execute_check(CAN_LISTEN)],
            ),
        ),
        [pda_signer(pda)],
    );
    let accounts = with_program_accounts(vec![
        (asset, asset_with_agent_identity(payer)),
        (payer, payer_account(ACCOUNT_LAMPORTS)),
        (pda, payer_account(ACCOUNT_LAMPORTS)),
    ]);

    let result = mollusk.process_instruction(&ix, &accounts);
    assert_core_err(&result, MplCoreError::ExternalPluginAdapterAlreadyExists);
}
