// Agent Identity External Plugin Mollusk Tests
//
// These tests verify the AgentIdentity external plugin adapter behavior using
// Mollusk, which allows us to force the agent identity PDA to be a signer
// without needing the mpl-agent-identity program deployed. Mollusk bypasses
// transaction-level signature verification, so we can set `signer: true` on
// the PDA's AccountMeta and the program's `assert_signer()` check will pass.

#[allow(deprecated)]
use {
    borsh::BorshSerialize,
    mollusk_svm::Mollusk,
    mpl_core_program::{
        plugins::{
            AgentIdentity, AgentIdentityInitInfo, AgentIdentityUpdateInfo, ExternalCheckResult,
            ExternalPluginAdapter, ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey,
            ExternalPluginAdapterType, ExternalPluginAdapterUpdateInfo, ExternalRegistryRecord,
            HookableLifecycleEvent, PluginHeaderV1, PluginRegistryV1,
        },
        state::{AssetV1, Authority, CollectionV1, DataBlob, Key, UpdateAuthority},
        ID as MPL_CORE_ID,
    },
    solana_sdk::{
        account::Account,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_program,
    },
};

// The mpl-agent-identity program ID used for PDA derivation.
const AGENT_IDENTITY_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("1DREGFgysWYxLnRnKQnwrxnJQeSMk2HmGaC6whw2B2p");

/// Minimum lamports for accounts to be rent-exempt-ish in tests.
const ACCOUNT_LAMPORTS: u64 = 1_000_000_000;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates a Mollusk instance for the mpl-core program.
fn core_mollusk() -> Mollusk {
    Mollusk::new(&MPL_CORE_ID, "mpl_core_program")
}

/// Creates a payer account with lamports.
fn payer_account() -> Account {
    Account {
        lamports: ACCOUNT_LAMPORTS,
        data: vec![],
        owner: system_program::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Normalizes accounts for Mollusk by replacing any system program entry with
/// the proper executable account.
fn to_mollusk_accounts(accounts: Vec<(Pubkey, Account)>) -> Vec<(Pubkey, Account)> {
    let mut result = accounts;

    // Remove duplicate system program if present.
    if let Some(pos) = result.iter().position(|(k, _)| *k == system_program::ID) {
        result.remove(pos);
    }

    let (sys_key, sys_account) = mollusk_svm::program::keyed_account_for_system_program();
    result.push((sys_key, sys_account));

    result
}

/// Derive the agent identity PDA for an asset.
fn agent_identity_pda(asset: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"agent_identity", asset.as_ref()],
        &AGENT_IDENTITY_PROGRAM_ID,
    )
}

fn assert_success(result: &mollusk_svm::result::InstructionResult) {
    assert!(
        matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Expected instruction to succeed, got: {:?}",
        result.program_result
    );
}

fn assert_failure(result: &mollusk_svm::result::InstructionResult) {
    assert!(
        !matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Expected instruction to fail, but it succeeded"
    );
}

// ---------------------------------------------------------------------------
// Account data builders
// ---------------------------------------------------------------------------

/// Serializes a bare AssetV1 (no plugins) owned by mpl-core.
fn valid_asset_account(owner: &Pubkey) -> Account {
    let asset = AssetV1::new(
        *owner,
        UpdateAuthority::Address(*owner),
        "Test Asset".to_string(),
        "https://example.com/test".to_string(),
    );
    let data = asset.try_to_vec().unwrap();
    Account {
        lamports: ACCOUNT_LAMPORTS,
        data,
        owner: MPL_CORE_ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Builds an Account containing an AssetV1 with an AgentIdentity external
/// plugin already initialized.
fn build_asset_with_agent_identity(
    owner: &Pubkey,
    uri: &str,
    plugin_authority: Authority,
    lifecycle_checks: Vec<(HookableLifecycleEvent, ExternalCheckResult)>,
) -> Account {
    let asset = AssetV1::new(
        *owner,
        UpdateAuthority::Address(*owner),
        "Test Asset".to_string(),
        "https://example.com/test".to_string(),
    );
    let asset_data = asset.try_to_vec().unwrap();
    let asset_len = asset.len();

    // Plugin header sits immediately after core data.
    let header_offset = asset_len;
    // PluginHeaderV1 is 9 bytes: 1 (Key) + 8 (usize).
    let plugin_data_start = header_offset + 9;

    // Serialize the external plugin adapter data.
    let plugin = ExternalPluginAdapter::AgentIdentity(AgentIdentity {
        uri: uri.to_string(),
    });
    let plugin_bytes = plugin.try_to_vec().unwrap();

    // The registry sits after the plugin data.
    let registry_offset = plugin_data_start + plugin_bytes.len();

    let header = PluginHeaderV1 {
        key: Key::PluginHeaderV1,
        plugin_registry_offset: registry_offset,
    };

    let external_record = ExternalRegistryRecord {
        plugin_type: ExternalPluginAdapterType::AgentIdentity,
        authority: plugin_authority,
        lifecycle_checks: Some(lifecycle_checks),
        offset: plugin_data_start,
        data_offset: None,
        data_len: None,
    };

    let registry = PluginRegistryV1 {
        key: Key::PluginRegistryV1,
        registry: vec![],
        external_registry: vec![external_record],
    };

    let header_bytes = header.try_to_vec().unwrap();
    let registry_bytes = registry.try_to_vec().unwrap();

    let mut data = Vec::new();
    data.extend_from_slice(&asset_data);
    data.extend_from_slice(&header_bytes);
    data.extend_from_slice(&plugin_bytes);
    data.extend_from_slice(&registry_bytes);

    Account {
        lamports: ACCOUNT_LAMPORTS,
        data,
        owner: MPL_CORE_ID,
        executable: false,
        rent_epoch: 0,
    }
}

// ---------------------------------------------------------------------------
// Instruction builders
// ---------------------------------------------------------------------------

/// Builds a CreateV2 instruction with an AgentIdentity external plugin.
///
/// Discriminator: 20
/// Account layout:
///   0: asset (writable, signer)
///   1: collection (optional) - use program ID as placeholder
///   2: authority (optional, signer) - use program ID as placeholder
///   3: payer (writable, signer)
///   4: owner (optional) - use program ID as placeholder
///   5: update_authority (optional) - use program ID as placeholder
///   6: system_program
///   7: log_wrapper (optional) - use program ID as placeholder
///   8: agent_identity_pda (signer) - remaining account
fn create_v2_with_agent_identity(
    asset: Pubkey,
    payer: Pubkey,
    agent_identity_pda: Pubkey,
    init_info: &AgentIdentityInitInfo,
) -> Instruction {
    // Serialize instruction data manually: discriminator + CreateV2Args fields.
    let mut data = vec![20u8]; // CreateV2 discriminator

    // DataState::AccountState = variant 0
    0u8.serialize(&mut data).unwrap();
    // name: String
    "Test Asset".to_string().serialize(&mut data).unwrap();
    // uri: String
    "https://example.com/test"
        .to_string()
        .serialize(&mut data)
        .unwrap();
    // plugins: Option<Vec<PluginAuthorityPair>> = None
    0u8.serialize(&mut data).unwrap();
    // external_plugin_adapters: Option<Vec<ExternalPluginAdapterInitInfo>> = Some(vec![...])
    let adapters = vec![ExternalPluginAdapterInitInfo::AgentIdentity(
        init_info.clone(),
    )];
    Some(adapters).serialize(&mut data).unwrap();

    Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(asset, true), // 0: asset (writable, signer)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 1: collection (optional)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 2: authority (optional)
            AccountMeta::new(payer, true), // 3: payer (writable, signer)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 4: owner (optional)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 5: update_authority (optional)
            AccountMeta::new_readonly(system_program::ID, false), // 6: system_program
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 7: log_wrapper (optional)
            AccountMeta::new_readonly(agent_identity_pda, true), // 8: PDA (signer!)
        ],
    )
}

/// Builds a CreateCollectionV2 instruction with an AgentIdentity external plugin.
///
/// Discriminator: 21
/// Account layout:
///   0: collection (writable, signer)
///   1: update_authority (optional)
///   2: payer (writable, signer)
///   3: system_program
fn create_collection_v2_with_agent_identity(
    collection: Pubkey,
    payer: Pubkey,
    init_info: &AgentIdentityInitInfo,
) -> Instruction {
    let mut data = vec![21u8]; // CreateCollectionV2 discriminator

    // name: String
    "Test Collection".to_string().serialize(&mut data).unwrap();
    // uri: String
    "https://example.com/collection"
        .to_string()
        .serialize(&mut data)
        .unwrap();
    // plugins: Option<Vec<PluginAuthorityPair>> = None
    0u8.serialize(&mut data).unwrap();
    // external_plugin_adapters: Option<Vec<ExternalPluginAdapterInitInfo>> = Some(vec![...])
    let adapters = vec![ExternalPluginAdapterInitInfo::AgentIdentity(
        init_info.clone(),
    )];
    Some(adapters).serialize(&mut data).unwrap();

    Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(collection, true), // 0: collection (writable, signer)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 1: update_authority (optional)
            AccountMeta::new(payer, true),      // 2: payer (writable, signer)
            AccountMeta::new_readonly(system_program::ID, false), // 3: system_program
        ],
    )
}

/// Builds an AddExternalPluginAdapterV1 instruction for AgentIdentity.
///
/// Discriminator: 22
/// Account layout:
///   0: asset (writable)
///   1: collection (optional)
///   2: payer (writable, signer)
///   3: authority (optional, signer)
///   4: system_program
///   5: log_wrapper (optional)
///   6: agent_identity_pda (signer!) - remaining account
fn add_agent_identity_instruction(
    asset: Pubkey,
    payer: Pubkey,
    agent_identity_pda: Pubkey,
    init_info: &AgentIdentityInitInfo,
) -> Instruction {
    let mut data = vec![22u8]; // AddExternalPluginAdapterV1 discriminator

    // init_info: ExternalPluginAdapterInitInfo
    ExternalPluginAdapterInitInfo::AgentIdentity(init_info.clone())
        .serialize(&mut data)
        .unwrap();

    Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(asset, false),                // 0: asset (writable)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 1: collection (optional)
            AccountMeta::new(payer, true),                 // 2: payer (writable, signer)
            AccountMeta::new_readonly(payer, true),        // 3: authority (signer)
            AccountMeta::new_readonly(system_program::ID, false), // 4: system_program
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 5: log_wrapper (optional)
            AccountMeta::new_readonly(agent_identity_pda, true), // 6: PDA (signer!)
        ],
    )
}

/// Builds an AddCollectionExternalPluginAdapterV1 instruction for AgentIdentity.
///
/// Discriminator: 23
/// Account layout:
///   0: collection (writable)
///   1: payer (writable, signer)
///   2: authority (optional, signer)
///   3: system_program
///   4: log_wrapper (optional)
fn add_collection_agent_identity_instruction(
    collection: Pubkey,
    payer: Pubkey,
    init_info: &AgentIdentityInitInfo,
) -> Instruction {
    let mut data = vec![23u8]; // AddCollectionExternalPluginAdapterV1 discriminator

    ExternalPluginAdapterInitInfo::AgentIdentity(init_info.clone())
        .serialize(&mut data)
        .unwrap();

    Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(collection, false), // 0: collection (writable)
            AccountMeta::new(payer, true),       // 1: payer (writable, signer)
            AccountMeta::new_readonly(payer, true), // 2: authority (signer)
            AccountMeta::new_readonly(system_program::ID, false), // 3: system_program
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 4: log_wrapper (optional)
        ],
    )
}

/// Builds an UpdateExternalPluginAdapterV1 instruction.
///
/// Discriminator: 26
/// Account layout:
///   0: asset (writable)
///   1: collection (optional)
///   2: payer (writable, signer)
///   3: authority (optional, signer)
///   4: system_program
///   5: log_wrapper (optional)
fn update_agent_identity_instruction(
    asset: Pubkey,
    payer: Pubkey,
    update_info: &AgentIdentityUpdateInfo,
) -> Instruction {
    let mut data = vec![26u8]; // UpdateExternalPluginAdapterV1 discriminator

    // key: ExternalPluginAdapterKey
    ExternalPluginAdapterKey::AgentIdentity
        .serialize(&mut data)
        .unwrap();
    // update_info: ExternalPluginAdapterUpdateInfo
    ExternalPluginAdapterUpdateInfo::AgentIdentity(update_info.clone())
        .serialize(&mut data)
        .unwrap();

    Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(asset, false),                // 0: asset (writable)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 1: collection (optional)
            AccountMeta::new(payer, true),                 // 2: payer (writable, signer)
            AccountMeta::new_readonly(payer, true),        // 3: authority (signer)
            AccountMeta::new_readonly(system_program::ID, false), // 4: system_program
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 5: log_wrapper (optional)
        ],
    )
}

/// Builds a RemoveExternalPluginAdapterV1 instruction.
///
/// Discriminator: 24
/// Account layout:
///   0: asset (writable)
///   1: collection (optional)
///   2: payer (writable, signer)
///   3: authority (optional, signer)
///   4: system_program
///   5: log_wrapper (optional)
fn remove_agent_identity_instruction(asset: Pubkey, payer: Pubkey) -> Instruction {
    let mut data = vec![24u8]; // RemoveExternalPluginAdapterV1 discriminator

    // key: ExternalPluginAdapterKey
    ExternalPluginAdapterKey::AgentIdentity
        .serialize(&mut data)
        .unwrap();

    Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(asset, false),                // 0: asset (writable)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 1: collection (optional)
            AccountMeta::new(payer, true),                 // 2: payer (writable, signer)
            AccountMeta::new_readonly(payer, true),        // 3: authority (signer)
            AccountMeta::new_readonly(system_program::ID, false), // 4: system_program
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 5: log_wrapper (optional)
        ],
    )
}

// ---------------------------------------------------------------------------
// Default init/update info helpers
// ---------------------------------------------------------------------------

fn default_agent_identity_init_info() -> AgentIdentityInitInfo {
    AgentIdentityInitInfo {
        uri: "https://example.com/agent.json".to_string(),
        init_plugin_authority: None, // defaults to UpdateAuthority
        lifecycle_checks: vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x1 }, // CAN_LISTEN
        )],
    }
}

/// An empty Account (system-program owned, zero data) for a new asset key.
fn empty_asset_account() -> Account {
    Account {
        lamports: 0,
        data: vec![],
        owner: system_program::ID,
        executable: false,
        rent_epoch: 0,
    }
}

// ===========================================================================
// Happy-path tests
// ===========================================================================

#[test]
fn create_asset_with_agent_identity() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (pda, _) = agent_identity_pda(&asset);

    let instruction =
        create_v2_with_agent_identity(asset, payer, pda, &default_agent_identity_init_info());

    let accounts = to_mollusk_accounts(vec![
        (asset, empty_asset_account()),
        (payer, payer_account()),
        (pda, payer_account()), // PDA account needs lamports to exist
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_success(&result);
}

#[test]
fn add_agent_identity_to_existing_asset() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (pda, _) = agent_identity_pda(&asset_key);

    let instruction =
        add_agent_identity_instruction(asset_key, payer, pda, &default_agent_identity_init_info());

    let accounts = to_mollusk_accounts(vec![
        (asset_key, valid_asset_account(&payer)),
        (payer, payer_account()),
        (pda, payer_account()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_success(&result);
}

#[test]
fn update_agent_identity_uri() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    let asset_account = build_asset_with_agent_identity(
        &payer,
        "https://example.com/agent.json",
        Authority::UpdateAuthority,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x1 },
        )],
    );

    let instruction = update_agent_identity_instruction(
        asset_key,
        payer,
        &AgentIdentityUpdateInfo {
            uri: Some("https://example.com/updated-agent.json".to_string()),
            lifecycle_checks: None,
        },
    );

    let accounts = to_mollusk_accounts(vec![(asset_key, asset_account), (payer, payer_account())]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_success(&result);
}

#[test]
fn update_agent_identity_lifecycle_checks() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    let asset_account = build_asset_with_agent_identity(
        &payer,
        "https://example.com/agent.json",
        Authority::UpdateAuthority,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x1 },
        )],
    );

    let instruction = update_agent_identity_instruction(
        asset_key,
        payer,
        &AgentIdentityUpdateInfo {
            uri: None,
            lifecycle_checks: Some(vec![(
                HookableLifecycleEvent::Execute,
                ExternalCheckResult { flags: 0x3 }, // CAN_LISTEN | CAN_APPROVE
            )]),
        },
    );

    let accounts = to_mollusk_accounts(vec![(asset_key, asset_account), (payer, payer_account())]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_success(&result);
}

#[test]
fn update_agent_identity_uri_and_lifecycle_checks() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    let asset_account = build_asset_with_agent_identity(
        &payer,
        "https://example.com/agent.json",
        Authority::UpdateAuthority,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x1 },
        )],
    );

    let instruction = update_agent_identity_instruction(
        asset_key,
        payer,
        &AgentIdentityUpdateInfo {
            uri: Some("https://example.com/agent-v3.json".to_string()),
            lifecycle_checks: Some(vec![(
                HookableLifecycleEvent::Execute,
                ExternalCheckResult { flags: 0x7 }, // CAN_LISTEN | CAN_APPROVE | CAN_REJECT
            )]),
        },
    );

    let accounts = to_mollusk_accounts(vec![(asset_key, asset_account), (payer, payer_account())]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_success(&result);
}

#[test]
fn remove_agent_identity() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    let asset_account = build_asset_with_agent_identity(
        &payer,
        "https://example.com/agent.json",
        Authority::UpdateAuthority,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x1 },
        )],
    );

    let instruction = remove_agent_identity_instruction(asset_key, payer);

    let accounts = to_mollusk_accounts(vec![(asset_key, asset_account), (payer, payer_account())]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_success(&result);
}

#[test]
fn create_asset_with_agent_identity_multiple_lifecycle_checks() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (pda, _) = agent_identity_pda(&asset);

    let init_info = AgentIdentityInitInfo {
        uri: "https://example.com/agent.json".to_string(),
        init_plugin_authority: None,
        lifecycle_checks: vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 }, // CAN_LISTEN | CAN_APPROVE
        )],
    };

    let instruction = create_v2_with_agent_identity(asset, payer, pda, &init_info);

    let accounts = to_mollusk_accounts(vec![
        (asset, empty_asset_account()),
        (payer, payer_account()),
        (pda, payer_account()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_success(&result);
}

#[test]
fn create_asset_with_agent_identity_address_authority() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (pda, _) = agent_identity_pda(&asset);
    let plugin_authority = Pubkey::new_unique();

    let init_info = AgentIdentityInitInfo {
        uri: "https://example.com/agent.json".to_string(),
        init_plugin_authority: Some(Authority::Address {
            address: plugin_authority,
        }),
        lifecycle_checks: vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x1 },
        )],
    };

    let instruction = create_v2_with_agent_identity(asset, payer, pda, &init_info);

    let accounts = to_mollusk_accounts(vec![
        (asset, empty_asset_account()),
        (payer, payer_account()),
        (pda, payer_account()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_success(&result);
}

// ===========================================================================
// Negative / security tests
// ===========================================================================

#[test]
fn cannot_create_collection_with_agent_identity() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let collection = Pubkey::new_unique();

    let instruction = create_collection_v2_with_agent_identity(
        collection,
        payer,
        &default_agent_identity_init_info(),
    );

    let accounts = to_mollusk_accounts(vec![
        (collection, empty_asset_account()),
        (payer, payer_account()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result);
}

#[test]
fn cannot_add_agent_identity_to_collection() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let collection_key = Pubkey::new_unique();

    // Build a valid collection account.
    let collection = CollectionV1::new(
        payer,
        "Test".to_string(),
        "https://example.com".to_string(),
        0,
        0,
    );
    let data = collection.try_to_vec().unwrap();
    let collection_account = Account {
        lamports: ACCOUNT_LAMPORTS,
        data,
        owner: MPL_CORE_ID,
        executable: false,
        rent_epoch: 0,
    };

    let instruction = add_collection_agent_identity_instruction(
        collection_key,
        payer,
        &default_agent_identity_init_info(),
    );

    let accounts = to_mollusk_accounts(vec![
        (collection_key, collection_account),
        (payer, payer_account()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result);
}

#[test]
fn cannot_add_agent_identity_without_pda_remaining_account() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    // Build the instruction WITHOUT the PDA remaining account.
    let mut data = vec![22u8]; // AddExternalPluginAdapterV1 discriminator
    ExternalPluginAdapterInitInfo::AgentIdentity(default_agent_identity_init_info())
        .serialize(&mut data)
        .unwrap();

    let instruction = Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(asset_key, false),
            AccountMeta::new_readonly(MPL_CORE_ID, false), // collection (optional)
            AccountMeta::new(payer, true),                 // payer
            AccountMeta::new_readonly(payer, true),        // authority
            AccountMeta::new_readonly(system_program::ID, false), // system_program
            AccountMeta::new_readonly(MPL_CORE_ID, false), // log_wrapper (optional)
                                                           // NO PDA remaining account!
        ],
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, valid_asset_account(&payer)),
        (payer, payer_account()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result);
}

#[test]
fn cannot_add_agent_identity_with_wrong_pda() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    // Use a completely wrong PDA (derived from a different asset).
    let wrong_asset = Pubkey::new_unique();
    let (wrong_pda, _) = agent_identity_pda(&wrong_asset);

    let instruction = add_agent_identity_instruction(
        asset_key,
        payer,
        wrong_pda,
        &default_agent_identity_init_info(),
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, valid_asset_account(&payer)),
        (payer, payer_account()),
        (wrong_pda, payer_account()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result);
}

#[test]
fn cannot_add_agent_identity_with_unsigned_pda() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (pda, _) = agent_identity_pda(&asset_key);

    // Build instruction with PDA present but NOT marked as signer.
    let mut data = vec![21u8];
    ExternalPluginAdapterInitInfo::AgentIdentity(default_agent_identity_init_info())
        .serialize(&mut data)
        .unwrap();

    let instruction = Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(asset_key, false),
            AccountMeta::new_readonly(MPL_CORE_ID, false),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(MPL_CORE_ID, false),
            AccountMeta::new_readonly(pda, false), // PDA present but NOT signer
        ],
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, valid_asset_account(&payer)),
        (payer, payer_account()),
        (pda, payer_account()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result);
}

#[test]
fn cannot_add_duplicate_agent_identity() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (pda, _) = agent_identity_pda(&asset_key);

    // Asset already has an agent identity plugin.
    let asset_account = build_asset_with_agent_identity(
        &payer,
        "https://example.com/agent.json",
        Authority::UpdateAuthority,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x1 },
        )],
    );

    let instruction = add_agent_identity_instruction(
        asset_key,
        payer,
        pda,
        &AgentIdentityInitInfo {
            uri: "https://example.com/agent2.json".to_string(),
            init_plugin_authority: None,
            lifecycle_checks: vec![(
                HookableLifecycleEvent::Execute,
                ExternalCheckResult { flags: 0x1 },
            )],
        },
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (payer, payer_account()),
        (pda, payer_account()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result);
}
