// Execution Delegate Mollusk Tests
//
// These tests verify the execution delegate mechanism in the AgentIdentity
// plugin's `validate_execute`. When an ExecutionDelegateRecordV1 account is
// passed as the first remaining account (index 7 in the full accounts list),
// and the signing authority matches the record, the plugin approves execution
// for non-owner authorities.

#[allow(deprecated)]
use {
    borsh::BorshSerialize,
    mollusk_svm::Mollusk,
    mpl_core_program::{
        plugins::{
            AgentIdentity, ExternalCheckResult, ExternalPluginAdapter, ExternalPluginAdapterType,
            ExternalRegistryRecord, HookableLifecycleEvent, PluginHeaderV1, PluginRegistryV1,
        },
        state::{AssetV1, Authority, DataBlob, Key, UpdateAuthority},
        ID as MPL_CORE_ID,
    },
    solana_program::program_error::ProgramError,
    solana_sdk::{
        account::Account,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_program,
    },
};

/// The mpl-agent-tools program ID (owner of ExecutionDelegateRecordV1 accounts).
const MPL_AGENT_TOOLS_ID: Pubkey =
    solana_sdk::pubkey!("TLREGni9ZEyGC3vnPZtqUh95xQ8oPqJSvNjvB7FGK8S");

/// Minimum lamports for accounts to be rent-exempt-ish in tests.
const ACCOUNT_LAMPORTS: u64 = 1_000_000_000;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn core_mollusk() -> Mollusk {
    Mollusk::new(&MPL_CORE_ID, "mpl_core_program")
}

fn payer_account() -> Account {
    Account {
        lamports: ACCOUNT_LAMPORTS,
        data: vec![],
        owner: system_program::ID,
        executable: false,
        rent_epoch: 0,
    }
}

fn to_mollusk_accounts(accounts: Vec<(Pubkey, Account)>) -> Vec<(Pubkey, Account)> {
    let mut result = accounts;

    if let Some(pos) = result.iter().position(|(k, _)| *k == system_program::ID) {
        result.remove(pos);
    }

    let (sys_key, sys_account) = mollusk_svm::program::keyed_account_for_system_program();
    result.push((sys_key, sys_account));

    result
}

fn assert_failure(result: &mollusk_svm::result::InstructionResult, expected: ProgramError) {
    match &result.program_result {
        mollusk_svm::result::ProgramResult::Success => {
            panic!(
                "Expected instruction to fail with {:?}, but it succeeded",
                expected
            );
        }
        mollusk_svm::result::ProgramResult::Failure(err) => {
            assert_eq!(
                *err, expected,
                "Expected error {:?}, got {:?}",
                expected, err
            );
        }
        mollusk_svm::result::ProgramResult::UnknownError(err) => {
            panic!(
                "Expected ProgramError {:?}, got UnknownError({:?})",
                expected, err
            );
        }
    }
}

/// Derive the asset signer PDA for the Execute instruction.
fn asset_signer_pda(asset: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"mpl-core-execute", asset.as_ref()], &MPL_CORE_ID)
}

// ---------------------------------------------------------------------------
// Account data builders
// ---------------------------------------------------------------------------

/// Builds an Account containing an AssetV1 with an AgentIdentity external
/// plugin that has the given lifecycle checks.
fn build_asset_with_agent_identity(
    owner: &Pubkey,
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

    let header_offset = asset_len;
    // PluginHeaderV1 is 9 bytes: 1 (Key) + 8 (usize).
    let plugin_data_start = header_offset + 9;

    let plugin = ExternalPluginAdapter::AgentIdentity(AgentIdentity {
        uri: "https://example.com/agent.json".to_string(),
    });
    let plugin_bytes = plugin.try_to_vec().unwrap();

    let registry_offset = plugin_data_start + plugin_bytes.len();

    let header = PluginHeaderV1 {
        key: Key::PluginHeaderV1,
        plugin_registry_offset: registry_offset,
    };

    let external_record = ExternalRegistryRecord {
        plugin_type: ExternalPluginAdapterType::AgentIdentity,
        authority: Authority::UpdateAuthority,
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

/// Builds a bare AssetV1 (no plugins) owned by mpl-core.
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

/// Manually constructs a 104-byte ExecutionDelegateRecordV1 account.
///
/// Layout (104 bytes):
///   [0]     = 0x02 (Key::ExecutionDelegateRecordV1)
///   [1]     = bump (u8)
///   [2..8]  = padding [0u8; 6]
///   [8..40] = executive_profile (Pubkey)
///   [40..72] = authority (Pubkey)
///   [72..104] = agent_asset (Pubkey)
fn build_execution_delegate_record(
    executive_profile: &Pubkey,
    authority: &Pubkey,
    agent_asset: &Pubkey,
) -> Account {
    let mut data = vec![0u8; 104];
    data[0] = 0x02; // Key::ExecutionDelegateRecordV1
    data[1] = 255; // bump (arbitrary)
                   // [2..8] padding already zeroed
    data[8..40].copy_from_slice(executive_profile.as_ref());
    data[40..72].copy_from_slice(authority.as_ref());
    data[72..104].copy_from_slice(agent_asset.as_ref());

    Account {
        lamports: ACCOUNT_LAMPORTS,
        data,
        owner: MPL_AGENT_TOOLS_ID,
        executable: false,
        rent_epoch: 0,
    }
}

// ---------------------------------------------------------------------------
// Instruction builder
// ---------------------------------------------------------------------------

/// Builds an ExecuteV1 instruction (discriminator 31).
///
/// Account layout (7 fixed):
///   0: asset (writable)
///   1: collection (optional, writable) -- MPL_CORE_ID placeholder
///   2: asset_signer -- PDA from ["mpl-core-execute", asset]
///   3: payer (writable, signer)
///   4: authority (optional, signer) -- MPL_CORE_ID placeholder if same as payer
///   5: system_program
///   6: program_id -- target CPI program
///
/// Remaining accounts (index 7+): delegate record (if any), then CPI accounts.
fn execute_v1_instruction(
    asset: Pubkey,
    asset_signer: Pubkey,
    payer: Pubkey,
    authority: Option<Pubkey>,
    program_id: Pubkey,
    delegate_record: Option<(Pubkey, Account)>,
    cpi_remaining_accounts: Vec<AccountMeta>,
    instruction_data: Vec<u8>,
) -> Instruction {
    let mut data = vec![31u8]; // ExecuteV1 discriminator

    // ExecuteV1Args: { instruction_data: Vec<u8> }
    instruction_data.serialize(&mut data).unwrap();

    let authority_meta = match authority {
        Some(auth) => AccountMeta::new_readonly(auth, true),
        None => AccountMeta::new_readonly(MPL_CORE_ID, false), // placeholder
    };

    let mut accounts = vec![
        AccountMeta::new(asset, false),                       // 0: asset
        AccountMeta::new(MPL_CORE_ID, false),                 // 1: collection (optional)
        AccountMeta::new_readonly(asset_signer, false),       // 2: asset_signer
        AccountMeta::new(payer, true),                        // 3: payer
        authority_meta,                                       // 4: authority
        AccountMeta::new_readonly(system_program::ID, false), // 5: system_program
        AccountMeta::new_readonly(program_id, false),         // 6: program_id
    ];

    if let Some((delegate_key, _)) = &delegate_record {
        accounts.push(AccountMeta::new_readonly(*delegate_key, false)); // 7: delegate record
    }

    accounts.extend(cpi_remaining_accounts);

    Instruction::new_with_bytes(MPL_CORE_ID, &data, accounts)
}

// ===========================================================================
// Happy-path tests
// ===========================================================================

/// Owner calls execute -- no delegate needed, owner authority approves.
/// The CPI to spl_noop will fail (program not loaded) but validation passes.
/// We verify the error is NOT NoApprovals (Custom(26)).
#[test]
fn execute_as_owner() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset_key);

    // Asset with AgentIdentity plugin (Execute: CAN_LISTEN | CAN_APPROVE)
    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 }, // CAN_LISTEN | CAN_APPROVE
        )],
    );

    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        owner,
        None, // authority = payer (owner)
        spl_noop::ID,
        None,   // no delegate record
        vec![], // no CPI remaining accounts
        vec![], // instruction_data
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (owner, payer_account()),
        (asset_signer, payer_account()),
        (
            spl_noop::ID,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: solana_sdk::bpf_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        ),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    // Validation should pass (owner is authority). CPI may fail if spl_noop
    // binary isn't loaded, but we should NOT get NoApprovals.
    match &result.program_result {
        mollusk_svm::result::ProgramResult::Success => { /* great */ }
        mollusk_svm::result::ProgramResult::Failure(err) => {
            assert_ne!(
                *err,
                ProgramError::Custom(26), // NoApprovals
                "Owner execute should not fail with NoApprovals"
            );
        }
        mollusk_svm::result::ProgramResult::UnknownError(_) => { /* CPI failure is OK */ }
    }
}

// ===========================================================================
// Execution delegate happy-path tests
// ===========================================================================

/// Non-owner authority with a valid ExecutionDelegateRecordV1 that matches
/// (authority + agent_asset). The AgentIdentity plugin has Execute with
/// CAN_APPROVE, so validation should approve.
#[test]
fn execute_with_valid_delegate_record() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate_authority = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset_key);
    let executive_profile = Pubkey::new_unique();

    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 }, // CAN_LISTEN | CAN_APPROVE
        )],
    );

    let delegate_record_key = Pubkey::new_unique();
    let delegate_record_account =
        build_execution_delegate_record(&executive_profile, &delegate_authority, &asset_key);

    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        delegate_authority,
        None, // authority = payer
        spl_noop::ID,
        Some((delegate_record_key, delegate_record_account.clone())),
        vec![],
        vec![],
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (delegate_authority, payer_account()),
        (asset_signer, payer_account()),
        (
            spl_noop::ID,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: solana_sdk::bpf_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        ),
        (delegate_record_key, delegate_record_account),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    // Validation should pass. CPI may fail but NOT NoApprovals.
    match &result.program_result {
        mollusk_svm::result::ProgramResult::Success => { /* great */ }
        mollusk_svm::result::ProgramResult::Failure(err) => {
            assert_ne!(
                *err,
                ProgramError::Custom(26), // NoApprovals
                "Valid delegate should not fail with NoApprovals"
            );
        }
        mollusk_svm::result::ProgramResult::UnknownError(_) => { /* CPI failure is OK */ }
    }
}

/// Delegate is also the payer (authority == payer), with separate authority account.
#[test]
fn execute_with_delegate_as_separate_authority() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate_authority = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset_key);
    let executive_profile = Pubkey::new_unique();
    let payer = Pubkey::new_unique();

    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 },
        )],
    );

    let delegate_record_key = Pubkey::new_unique();
    let delegate_record_account =
        build_execution_delegate_record(&executive_profile, &delegate_authority, &asset_key);

    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        payer,
        Some(delegate_authority), // separate authority
        spl_noop::ID,
        Some((delegate_record_key, delegate_record_account.clone())),
        vec![],
        vec![],
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (payer, payer_account()),
        (delegate_authority, payer_account()),
        (asset_signer, payer_account()),
        (
            spl_noop::ID,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: solana_sdk::bpf_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        ),
        (delegate_record_key, delegate_record_account),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    match &result.program_result {
        mollusk_svm::result::ProgramResult::Success => { /* great */ }
        mollusk_svm::result::ProgramResult::Failure(err) => {
            assert_ne!(
                *err,
                ProgramError::Custom(26),
                "Valid delegate with separate authority should not fail with NoApprovals"
            );
        }
        mollusk_svm::result::ProgramResult::UnknownError(_) => { /* CPI failure is OK */ }
    }
}

// ===========================================================================
// Negative / security tests
// ===========================================================================

/// Non-owner, no remaining accounts (<=7 total) -- plugin abstains -- NoApprovals.
#[test]
fn execute_non_owner_without_remaining_accounts() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let non_owner = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset_key);

    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 },
        )],
    );

    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        non_owner,
        None,
        spl_noop::ID,
        None, // no delegate record
        vec![],
        vec![],
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (non_owner, payer_account()),
        (asset_signer, payer_account()),
        (
            spl_noop::ID,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: solana_sdk::bpf_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        ),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result, ProgramError::Custom(26)); // NoApprovals
}

/// Wrong PDA for asset_signer -- InvalidExecutePda (Custom(49)).
#[test]
fn execute_with_invalid_asset_signer_pda() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let wrong_signer = Pubkey::new_unique(); // not a valid PDA

    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 },
        )],
    );

    let instruction = execute_v1_instruction(
        asset_key,
        wrong_signer,
        owner,
        None,
        spl_noop::ID,
        None,
        vec![],
        vec![],
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (owner, payer_account()),
        (wrong_signer, payer_account()),
        (
            spl_noop::ID,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: solana_sdk::bpf_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        ),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result, ProgramError::Custom(49)); // InvalidExecutePda
}

/// Asset has no AgentIdentity plugin, non-owner tries execute -- NoApprovals.
#[test]
fn execute_non_owner_without_agent_identity_plugin() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let non_owner = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset_key);

    // Bare asset, no plugins.
    let asset_account = valid_asset_account(&owner);

    let delegate_record_key = Pubkey::new_unique();
    let delegate_record_account =
        build_execution_delegate_record(&Pubkey::new_unique(), &non_owner, &asset_key);

    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        non_owner,
        None,
        spl_noop::ID,
        Some((delegate_record_key, delegate_record_account.clone())),
        vec![],
        vec![],
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (non_owner, payer_account()),
        (asset_signer, payer_account()),
        (
            spl_noop::ID,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: solana_sdk::bpf_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        ),
        (delegate_record_key, delegate_record_account),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result, ProgramError::Custom(26)); // NoApprovals
}

/// Delegate record has different authority than signer -- Abstain -- NoApprovals.
#[test]
fn execute_with_wrong_authority_delegate() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let actual_signer = Pubkey::new_unique();
    let different_authority = Pubkey::new_unique(); // doesn't match signer
    let asset_key = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset_key);

    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 },
        )],
    );

    let delegate_record_key = Pubkey::new_unique();
    // Record authority is different_authority, but signer is actual_signer
    let delegate_record_account =
        build_execution_delegate_record(&Pubkey::new_unique(), &different_authority, &asset_key);

    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        actual_signer,
        None,
        spl_noop::ID,
        Some((delegate_record_key, delegate_record_account.clone())),
        vec![],
        vec![],
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (actual_signer, payer_account()),
        (asset_signer, payer_account()),
        (
            spl_noop::ID,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: solana_sdk::bpf_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        ),
        (delegate_record_key, delegate_record_account),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result, ProgramError::Custom(26)); // NoApprovals
}

/// Delegate record has different agent_asset -- Abstain -- NoApprovals.
#[test]
fn execute_with_wrong_asset_delegate() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate_authority = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let wrong_asset = Pubkey::new_unique(); // doesn't match asset_key
    let (asset_signer, _) = asset_signer_pda(&asset_key);

    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 },
        )],
    );

    let delegate_record_key = Pubkey::new_unique();
    // Record agent_asset is wrong_asset, not asset_key
    let delegate_record_account =
        build_execution_delegate_record(&Pubkey::new_unique(), &delegate_authority, &wrong_asset);

    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        delegate_authority,
        None,
        spl_noop::ID,
        Some((delegate_record_key, delegate_record_account.clone())),
        vec![],
        vec![],
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (delegate_authority, payer_account()),
        (asset_signer, payer_account()),
        (
            spl_noop::ID,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: solana_sdk::bpf_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        ),
        (delegate_record_key, delegate_record_account),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result, ProgramError::Custom(26)); // NoApprovals
}

/// Delegate record account NOT owned by mpl_agent_tools::ID -- Abstain -- NoApprovals.
#[test]
fn execute_with_wrong_program_owner_delegate() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate_authority = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset_key);

    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 },
        )],
    );

    let delegate_record_key = Pubkey::new_unique();
    let mut delegate_record_account =
        build_execution_delegate_record(&Pubkey::new_unique(), &delegate_authority, &asset_key);
    // Override owner to system_program (wrong owner)
    delegate_record_account.owner = system_program::ID;

    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        delegate_authority,
        None,
        spl_noop::ID,
        Some((delegate_record_key, delegate_record_account.clone())),
        vec![],
        vec![],
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (delegate_authority, payer_account()),
        (asset_signer, payer_account()),
        (
            spl_noop::ID,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: solana_sdk::bpf_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        ),
        (delegate_record_key, delegate_record_account),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result, ProgramError::Custom(26)); // NoApprovals
}

/// Delegate record with Key byte != 0x02 -- Abstain -- NoApprovals.
#[test]
fn execute_with_invalid_discriminator_delegate() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate_authority = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset_key);

    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 },
        )],
    );

    let delegate_record_key = Pubkey::new_unique();
    let mut delegate_record_account =
        build_execution_delegate_record(&Pubkey::new_unique(), &delegate_authority, &asset_key);
    // Override discriminator byte to invalid value
    delegate_record_account.data[0] = 0xFF;

    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        delegate_authority,
        None,
        spl_noop::ID,
        Some((delegate_record_key, delegate_record_account.clone())),
        vec![],
        vec![],
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (delegate_authority, payer_account()),
        (asset_signer, payer_account()),
        (
            spl_noop::ID,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: solana_sdk::bpf_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        ),
        (delegate_record_key, delegate_record_account),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert_failure(&result, ProgramError::Custom(26)); // NoApprovals
}

// ===========================================================================
// CPI account shift bug tests
// ===========================================================================

/// Baseline: owner executes a system transfer via CPI with no delegate record.
/// The asset_signer PDA is the source. This should succeed end-to-end.
#[test]
fn execute_system_transfer_owner_no_delegate() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset_key);
    let dest = Pubkey::new_unique();

    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 },
        )],
    );

    // Build a system transfer instruction: asset_signer -> dest, 1 lamport.
    let transfer_ix = solana_sdk::system_instruction::transfer(&asset_signer, &dest, 1);

    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        owner,
        None,
        system_program::ID, // CPI target = system program
        None,               // no delegate record
        vec![
            AccountMeta::new(asset_signer, false), // CPI account 0: source
            AccountMeta::new(dest, false),         // CPI account 1: dest
        ],
        transfer_ix.data,
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (owner, payer_account()),
        (
            asset_signer,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            dest,
            Account {
                lamports: 0,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        ),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert!(
        matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Owner system transfer CPI should succeed, got: {:?}",
        result.program_result
    );
}

/// Delegate execute with system transfer CPI. The delegate record at
/// remaining_accounts[0] must be stripped before CPI so the target program
/// receives only the real CPI accounts.
#[test]
fn execute_system_transfer_with_delegate_record_stripped() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate_authority = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset_key);
    let executive_profile = Pubkey::new_unique();
    let dest = Pubkey::new_unique();

    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 },
        )],
    );

    let delegate_record_key = Pubkey::new_unique();
    let delegate_record_account =
        build_execution_delegate_record(&executive_profile, &delegate_authority, &asset_key);

    let transfer_ix = solana_sdk::system_instruction::transfer(&asset_signer, &dest, 1);

    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        delegate_authority,
        None,
        system_program::ID,
        Some((delegate_record_key, delegate_record_account.clone())),
        vec![
            AccountMeta::new(asset_signer, false),
            AccountMeta::new(dest, false),
        ],
        transfer_ix.data,
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (delegate_authority, payer_account()),
        (
            asset_signer,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            dest,
            Account {
                lamports: 0,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (delegate_record_key, delegate_record_account),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert!(
        matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Delegate system transfer CPI should succeed after stripping delegate record, got: {:?}",
        result.program_result
    );
}

/// Delegate execute with separate authority and system transfer CPI.
#[test]
fn execute_system_transfer_delegate_separate_authority() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate_authority = Pubkey::new_unique();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset_key);
    let executive_profile = Pubkey::new_unique();
    let dest = Pubkey::new_unique();

    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 },
        )],
    );

    let delegate_record_key = Pubkey::new_unique();
    let delegate_record_account =
        build_execution_delegate_record(&executive_profile, &delegate_authority, &asset_key);

    let transfer_ix = solana_sdk::system_instruction::transfer(&asset_signer, &dest, 1);

    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        payer,
        Some(delegate_authority),
        system_program::ID,
        Some((delegate_record_key, delegate_record_account.clone())),
        vec![
            AccountMeta::new(asset_signer, false),
            AccountMeta::new(dest, false),
        ],
        transfer_ix.data,
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (payer, payer_account()),
        (delegate_authority, payer_account()),
        (
            asset_signer,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            dest,
            Account {
                lamports: 0,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (delegate_record_key, delegate_record_account),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert!(
        matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Delegate (separate authority) system transfer should succeed, got: {:?}",
        result.program_result
    );
}

/// Non-delegate remaining account (not owned by mpl_agent_tools) should NOT be
/// stripped -- it should be passed through to the CPI as-is.
#[test]
fn execute_system_transfer_non_delegate_remaining_account_preserved() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset_key);
    let dest = Pubkey::new_unique();

    let asset_account = build_asset_with_agent_identity(
        &owner,
        vec![(
            HookableLifecycleEvent::Execute,
            ExternalCheckResult { flags: 0x3 },
        )],
    );

    let transfer_ix = solana_sdk::system_instruction::transfer(&asset_signer, &dest, 1);

    // No delegate record -- just CPI accounts in remaining.
    let instruction = execute_v1_instruction(
        asset_key,
        asset_signer,
        owner,
        None,
        system_program::ID,
        None,
        vec![
            AccountMeta::new(asset_signer, false),
            AccountMeta::new(dest, false),
        ],
        transfer_ix.data,
    );

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (owner, payer_account()),
        (
            asset_signer,
            Account {
                lamports: ACCOUNT_LAMPORTS,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            dest,
            Account {
                lamports: 0,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        ),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert!(
        matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Owner CPI without delegate should pass all remaining accounts through, got: {:?}",
        result.program_result
    );
}
