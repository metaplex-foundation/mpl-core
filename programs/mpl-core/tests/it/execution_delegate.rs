//! Execution delegates: `ExecuteV1` by a non-owner authority.
//!
//! The AgentIdentity adapter's `validate_execute` approves a non-owner
//! authority when the first remaining account (index 7 of the instruction)
//! is an `ExecutionDelegateRecordV1` owned by mpl-agent-tools whose
//! `authority` and `agent_asset` match the signer and the asset. Any other
//! account there makes the plugin abstain, and with nothing else approving
//! the instruction fails with `NoApprovals`. Once validated, `ExecuteV1`
//! strips the record before forwarding the remaining accounts to the target
//! program and marks the asset signer PDA as a signer.
//!
//! Wherever the legacy test only cared that validation passed, the recorder
//! builtin is the CPI target: the instruction then has to succeed, and the
//! recorded invocation shows exactly what the target program received.

use {
    crate::common::*,
    mollusk_svm::{result::InstructionResult, Mollusk},
    mpl_core_program::{
        error::MplCoreError,
        plugins::{ExternalCheckResult, HookableLifecycleEvent},
    },
    solana_account::Account,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    solana_system_interface::{instruction as system_instruction, program as system_program},
};

const AGENT_URI: &str = "https://example.com/agent.json";

/// `CAN_LISTEN | CAN_APPROVE`.
const LISTEN_AND_APPROVE: ExternalCheckResult = ExternalCheckResult { flags: 0x3 };

/// Instruction data handed to `ExecuteV1`; the target must receive it verbatim.
const FORWARDED_DATA: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];

/// An asset owned by `owner` whose AgentIdentity adapter can approve `Execute`.
fn agent_asset(owner: Pubkey) -> Account {
    AssetSpec::new(owner)
        .adapter(ExternalAdapterSpec::agent_identity(
            AGENT_URI,
            vec![(HookableLifecycleEvent::Execute, LISTEN_AND_APPROVE)],
        ))
        .build()
}

/// The optional delegate record as the leading remaining account.
fn record_meta(record: Option<Pubkey>) -> Vec<AccountMeta> {
    record
        .map(|key| AccountMeta::new_readonly(key, false))
        .into_iter()
        .collect()
}

/// `ExecuteV1` on `asset` targeting the recorder with `FORWARDED_DATA`. The
/// remaining accounts are `record` (when given) followed by the asset signer
/// PDA, passed unsigned so the test can see the program upgrade it to a
/// signer.
fn record_ix(
    asset: Pubkey,
    payer: Pubkey,
    authority: Option<Pubkey>,
    record: Option<Pubkey>,
) -> Instruction {
    let (asset_signer, _) = asset_signer_pda(&asset);
    let ix = ix::execute_v1(
        asset,
        None,
        asset_signer,
        payer,
        authority,
        RECORDER_ID,
        FORWARDED_DATA.to_vec(),
        &record_meta(record),
    );
    with_remaining(ix, [AccountMeta::new_readonly(asset_signer, false)])
}

/// `ExecuteV1` on `asset` that CPIs a one-lamport system transfer from the
/// asset signer PDA to `dest`, with `record` (when given) ahead of the CPI
/// accounts.
fn transfer_ix(
    asset: Pubkey,
    payer: Pubkey,
    authority: Option<Pubkey>,
    record: Option<Pubkey>,
    dest: Pubkey,
) -> Instruction {
    let (asset_signer, _) = asset_signer_pda(&asset);
    let transfer = system_instruction::transfer(&asset_signer, &dest, 1);
    let ix = ix::execute_v1(
        asset,
        None,
        asset_signer,
        payer,
        authority,
        system_program::ID,
        transfer.data,
        &record_meta(record),
    );
    with_remaining(
        ix,
        [
            AccountMeta::new(asset_signer, false),
            AccountMeta::new(dest, false),
        ],
    )
}

/// Runs `ix` with the program accounts filled in, clearing the recorder first.
fn run(mollusk: &Mollusk, ix: &Instruction, accounts: Vec<(Pubkey, Account)>) -> InstructionResult {
    clear_recorded();
    mollusk.process_instruction(ix, &with_program_accounts(accounts))
}

/// Asserts the recorder was invoked once, with the asset signer PDA as its
/// only account (so any delegate record was stripped), passed as a signer,
/// and with `FORWARDED_DATA` unchanged.
fn assert_forwarded(asset: &Pubkey) {
    let (asset_signer, _) = asset_signer_pda(asset);
    assert_eq!(
        take_recorded(),
        vec![RecordedInvocation {
            program_id: RECORDER_ID,
            accounts: vec![RecordedAccount {
                pubkey: asset_signer,
                is_signer: true,
                is_writable: false,
            }],
            data: FORWARDED_DATA.to_vec(),
        }]
    );
}

/// Asserts the target program was never reached.
fn assert_no_cpi() {
    assert!(
        take_recorded().is_empty(),
        "validation failed, so the target program must not have been invoked"
    );
}

/// Asserts the instruction succeeded, the execute fee moved from `payer` to
/// `asset` (both funded with `ACCOUNT_LAMPORTS`) and the asset layout is
/// intact.
fn assert_executed(result: &InstructionResult, asset: &Pubkey, payer: &Pubkey) {
    assert_ok(result);
    let fee = ACCOUNT_LAMPORTS - lamports_of(result, payer);
    assert!(fee > 0, "the execute fee must be charged to the payer");
    assert_eq!(
        lamports_of(result, asset),
        ACCOUNT_LAMPORTS + fee,
        "the execute fee must be credited to the asset"
    );
    assert_registry_consistent(account_of(result, asset));
}

/// Asserts the one-lamport CPI transfer from the asset signer PDA to `dest`
/// went through.
fn assert_transferred(result: &InstructionResult, asset: &Pubkey, dest: &Pubkey) {
    let (asset_signer, _) = asset_signer_pda(asset);
    assert_eq!(
        lamports_of(result, dest),
        1,
        "dest must receive the lamport"
    );
    assert_eq!(
        lamports_of(result, &asset_signer),
        ACCOUNT_LAMPORTS - 1,
        "the asset signer PDA must fund the transfer"
    );
}

// ===========================================================================
// Happy paths
// ===========================================================================

/// Owner calls execute: no delegate needed, the owner authority approves and
/// the target receives the asset signer PDA as a signer.
#[test]
fn execute_as_owner() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset);

    let result = run(
        &mollusk,
        &record_ix(asset, owner, None, None),
        vec![
            (asset, agent_asset(owner)),
            (owner, payer_account(ACCOUNT_LAMPORTS)),
            (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
        ],
    );
    assert_executed(&result, &asset, &owner);
    assert_forwarded(&asset);
}

/// Non-owner authority with an `ExecutionDelegateRecordV1` matching both the
/// authority and the asset: the AgentIdentity plugin approves, and the record
/// is stripped from what the target receives.
#[test]
fn execute_with_valid_delegate_record() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset);
    let record = Pubkey::new_unique();

    let result = run(
        &mollusk,
        &record_ix(asset, delegate, None, Some(record)),
        vec![
            (asset, agent_asset(owner)),
            (delegate, payer_account(ACCOUNT_LAMPORTS)),
            (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
            (
                record,
                execution_delegate_record(Pubkey::new_unique(), delegate, asset),
            ),
        ],
    );
    assert_executed(&result, &asset, &delegate);
    assert_forwarded(&asset);
}

/// The delegate signs as a separate authority account while another account
/// pays.
#[test]
fn execute_with_delegate_as_separate_authority() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset);
    let record = Pubkey::new_unique();

    let result = run(
        &mollusk,
        &record_ix(asset, payer, Some(delegate), Some(record)),
        vec![
            (asset, agent_asset(owner)),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (delegate, payer_account(ACCOUNT_LAMPORTS)),
            (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
            (
                record,
                execution_delegate_record(Pubkey::new_unique(), delegate, asset),
            ),
        ],
    );
    assert_executed(&result, &asset, &payer);
    assert_eq!(
        lamports_of(&result, &delegate),
        ACCOUNT_LAMPORTS,
        "the authority must not be charged when a separate payer is given"
    );
    assert_forwarded(&asset);
}

// ===========================================================================
// Negative / security
// ===========================================================================

/// Non-owner with no remaining accounts (7 accounts total): the plugin
/// abstains and nothing approves.
#[test]
fn execute_non_owner_without_remaining_accounts() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let non_owner = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset);

    let ix = ix::execute_v1(
        asset,
        None,
        asset_signer,
        non_owner,
        None,
        RECORDER_ID,
        FORWARDED_DATA.to_vec(),
        &[],
    );
    let result = run(
        &mollusk,
        &ix,
        vec![
            (asset, agent_asset(owner)),
            (non_owner, payer_account(ACCOUNT_LAMPORTS)),
            (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
        ],
    );
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_no_cpi();
}

/// An `asset_signer` that is not the PDA derived from the asset.
#[test]
fn execute_with_invalid_asset_signer_pda() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let wrong_signer = Pubkey::new_unique();

    let ix = ix::execute_v1(
        asset,
        None,
        wrong_signer,
        owner,
        None,
        RECORDER_ID,
        FORWARDED_DATA.to_vec(),
        &[AccountMeta::new_readonly(wrong_signer, false)],
    );
    let result = run(
        &mollusk,
        &ix,
        vec![
            (asset, agent_asset(owner)),
            (owner, payer_account(ACCOUNT_LAMPORTS)),
            (wrong_signer, payer_account(ACCOUNT_LAMPORTS)),
        ],
    );
    assert_core_err(&result, MplCoreError::InvalidExecutePda);
    assert_no_cpi();
}

/// A matching delegate record cannot help when the asset has no AgentIdentity
/// plugin to evaluate it.
#[test]
fn execute_non_owner_without_agent_identity_plugin() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let non_owner = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset);
    let record = Pubkey::new_unique();

    let result = run(
        &mollusk,
        &record_ix(asset, non_owner, None, Some(record)),
        vec![
            (asset, AssetSpec::new(owner).build()),
            (non_owner, payer_account(ACCOUNT_LAMPORTS)),
            (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
            (
                record,
                execution_delegate_record(Pubkey::new_unique(), non_owner, asset),
            ),
        ],
    );
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_no_cpi();
}

/// The record names a different authority than the signer.
#[test]
fn execute_with_wrong_authority_delegate() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let signer = Pubkey::new_unique();
    let other_authority = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset);
    let record = Pubkey::new_unique();

    let result = run(
        &mollusk,
        &record_ix(asset, signer, None, Some(record)),
        vec![
            (asset, agent_asset(owner)),
            (signer, payer_account(ACCOUNT_LAMPORTS)),
            (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
            (
                record,
                execution_delegate_record(Pubkey::new_unique(), other_authority, asset),
            ),
        ],
    );
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_no_cpi();
}

/// The record names a different agent asset.
#[test]
fn execute_with_wrong_asset_delegate() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let other_asset = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset);
    let record = Pubkey::new_unique();

    let result = run(
        &mollusk,
        &record_ix(asset, delegate, None, Some(record)),
        vec![
            (asset, agent_asset(owner)),
            (delegate, payer_account(ACCOUNT_LAMPORTS)),
            (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
            (
                record,
                execution_delegate_record(Pubkey::new_unique(), delegate, other_asset),
            ),
        ],
    );
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_no_cpi();
}

/// A well-formed record that is not owned by mpl-agent-tools is ignored.
#[test]
fn execute_with_wrong_program_owner_delegate() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset);
    let record = Pubkey::new_unique();
    let mut record_account = execution_delegate_record(Pubkey::new_unique(), delegate, asset);
    record_account.owner = system_program::ID;

    let result = run(
        &mollusk,
        &record_ix(asset, delegate, None, Some(record)),
        vec![
            (asset, agent_asset(owner)),
            (delegate, payer_account(ACCOUNT_LAMPORTS)),
            (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
            (record, record_account),
        ],
    );
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_no_cpi();
}

/// A record whose discriminator is not `Key::ExecutionDelegateRecordV1` is
/// ignored.
#[test]
fn execute_with_invalid_discriminator_delegate() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset);
    let record = Pubkey::new_unique();
    let mut record_account = execution_delegate_record(Pubkey::new_unique(), delegate, asset);
    record_account.data[0] = 0xFF;

    let result = run(
        &mollusk,
        &record_ix(asset, delegate, None, Some(record)),
        vec![
            (asset, agent_asset(owner)),
            (delegate, payer_account(ACCOUNT_LAMPORTS)),
            (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
            (record, record_account),
        ],
    );
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_no_cpi();
}

// ===========================================================================
// CPI account forwarding (the delegate-record strip must not shift accounts)
// ===========================================================================

/// Baseline: the owner runs a system transfer from the asset signer PDA with
/// no delegate record in the remaining accounts.
#[test]
fn execute_system_transfer_owner_no_delegate() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset);
    let dest = Pubkey::new_unique();

    let result = run(
        &mollusk,
        &transfer_ix(asset, owner, None, None, dest),
        vec![
            (asset, agent_asset(owner)),
            (owner, payer_account(ACCOUNT_LAMPORTS)),
            (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
            (dest, empty_account()),
        ],
    );
    assert_executed(&result, &asset, &owner);
    assert_transferred(&result, &asset, &dest);
}

/// With a delegate record first in the remaining accounts, the record is
/// stripped before the CPI so the system program sees only the transfer's
/// two accounts.
#[test]
fn execute_system_transfer_with_delegate_record_stripped() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset);
    let record = Pubkey::new_unique();
    let dest = Pubkey::new_unique();

    let result = run(
        &mollusk,
        &transfer_ix(asset, delegate, None, Some(record), dest),
        vec![
            (asset, agent_asset(owner)),
            (delegate, payer_account(ACCOUNT_LAMPORTS)),
            (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
            (dest, empty_account()),
            (
                record,
                execution_delegate_record(Pubkey::new_unique(), delegate, asset),
            ),
        ],
    );
    assert_executed(&result, &asset, &delegate);
    assert_transferred(&result, &asset, &dest);
}

/// Same as above with the delegate signing as a separate authority.
#[test]
fn execute_system_transfer_delegate_separate_authority() {
    let mollusk = core_mollusk();
    let owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let (asset_signer, _) = asset_signer_pda(&asset);
    let record = Pubkey::new_unique();
    let dest = Pubkey::new_unique();

    let result = run(
        &mollusk,
        &transfer_ix(asset, payer, Some(delegate), Some(record), dest),
        vec![
            (asset, agent_asset(owner)),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (delegate, payer_account(ACCOUNT_LAMPORTS)),
            (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
            (dest, empty_account()),
            (
                record,
                execution_delegate_record(Pubkey::new_unique(), delegate, asset),
            ),
        ],
    );
    assert_executed(&result, &asset, &payer);
    assert_transferred(&result, &asset, &dest);
}

/// A leading remaining account that is not an `ExecutionDelegateRecordV1` is
/// forwarded to the target untouched. The strip in `processor/execute.rs`
/// (lines 108-118) only fires when all three of its conditions hold, so each
/// case here fails exactly one of them and must survive.
///
/// The recorder is the target rather than the system program: forwarded
/// accounts are positional, so a preserved leading account would make a
/// system transfer read its source from the wrong slot. The recorder simply
/// reports what it received, which is the property under test.
///
/// The owner pays, so the base asset path approves and execution reaches the
/// strip. (The delegate-record tests above stop at `NoApprovals` instead,
/// which is why none of them reaches this branch.)
#[test]
fn execute_forwards_non_record_leading_account() {
    for (case, account) in [
        // Owned by mpl-agent-tools and long enough, but the discriminator is
        // not ExecutionDelegateRecordV1.
        ("wrong discriminator", {
            let mut a = execution_delegate_record(
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                Pubkey::new_unique(),
            );
            a.data[0] = 0xFF;
            a
        }),
        // Owned by mpl-agent-tools but empty, so the discriminator cannot be
        // read at all (`data_len() > 0` is false).
        ("empty agent-tools account", {
            let mut a = buffer_account(vec![]);
            a.owner = mpl_agent_tools::ID;
            a
        }),
        // Carries a valid record discriminator but belongs to another program.
        ("wrong program owner", {
            let mut a = execution_delegate_record(
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                Pubkey::new_unique(),
            );
            a.owner = system_program::ID;
            a
        }),
    ] {
        let mollusk = core_mollusk();
        let owner = Pubkey::new_unique();
        let asset = Pubkey::new_unique();
        let (asset_signer, _) = asset_signer_pda(&asset);
        let not_a_record = Pubkey::new_unique();

        let result = run(
            &mollusk,
            &record_ix(asset, owner, None, Some(not_a_record)),
            vec![
                (asset, agent_asset(owner)),
                (owner, payer_account(ACCOUNT_LAMPORTS)),
                (asset_signer, payer_account(ACCOUNT_LAMPORTS)),
                (not_a_record, account),
            ],
        );
        assert_executed(&result, &asset, &owner);

        // Both accounts reach the target, in order and unshifted: the leading
        // account was inspected and kept, and the asset signer is still
        // upgraded to a signer behind it.
        assert_eq!(
            take_recorded(),
            vec![RecordedInvocation {
                program_id: RECORDER_ID,
                accounts: vec![
                    RecordedAccount {
                        pubkey: not_a_record,
                        is_signer: false,
                        is_writable: false,
                    },
                    RecordedAccount {
                        pubkey: asset_signer,
                        is_signer: true,
                        is_writable: false,
                    },
                ],
                data: FORWARDED_DATA.to_vec(),
            }],
            "{case}: the leading account must be forwarded, not stripped"
        );
    }
}
