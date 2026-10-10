//! Programs registered next to mpl-core in the Mollusk harness, and the
//! executable accounts that represent them.
//!
//! Two host-function builtins are added to every [`super::core_mollusk`]
//! instance in both native and SBF modes:
//!
//! - an SPL Noop stand-in at [`SPL_NOOP_ID`], so `Wrappable::wrap` CPIs
//!   succeed without loading the real ELF, and
//! - a `recorder` at [`RECORDER_ID`] that remembers every instruction it
//!   receives (see [`take_recorded`]) and returns a configurable result (see
//!   [`set_recorder_result`]), for `ExecuteV1` tests that need to assert what
//!   the program CPIed into.

use {
    super::{core_program_account, PROGRAM_NAME},
    mollusk_svm::{
        program::{
            create_keyed_account_for_builtin_program, keyed_account_for_system_program, Builtin,
        },
        Mollusk,
    },
    mpl_core_program::ID as MPL_CORE_ID,
    solana_account::Account,
    solana_program::{instruction::InstructionError, pubkey::Pubkey},
    solana_program_runtime::declare_process_instruction,
    std::cell::RefCell,
};

pub use mpl_core_program::SPL_NOOP_ID;

/// Name under which the noop builtin is registered.
pub const SPL_NOOP_NAME: &str = "spl_noop";

/// Program id of the recording builtin.
pub const RECORDER_ID: Pubkey = Pubkey::new_from_array([0xBB; 32]);

/// Name under which the recorder builtin is registered.
pub const RECORDER_NAME: &str = "recorder";

/// Compute units charged per builtin invocation.
const BUILTIN_COMPUTE_UNITS: u64 = 1;

/// One account as the recorder saw it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedAccount {
    /// The account key.
    pub pubkey: Pubkey,
    /// Whether the account was passed as a signer.
    pub is_signer: bool,
    /// Whether the account was passed as writable.
    pub is_writable: bool,
}

/// One invocation of the recorder builtin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedInvocation {
    /// The program id the runtime invoked (always [`RECORDER_ID`]).
    pub program_id: Pubkey,
    /// The instruction accounts in order, with their flags.
    pub accounts: Vec<RecordedAccount>,
    /// The instruction data.
    pub data: Vec<u8>,
}

thread_local! {
    static RECORDED: RefCell<Vec<RecordedInvocation>> = const { RefCell::new(Vec::new()) };
    static RECORDER_RESULT: RefCell<Result<(), InstructionError>> = const { RefCell::new(Ok(())) };
}

declare_process_instruction!(NoopEntrypoint, BUILTIN_COMPUTE_UNITS, |_invoke_context| {
    Ok(())
});

declare_process_instruction!(
    RecorderEntrypoint,
    BUILTIN_COMPUTE_UNITS,
    |invoke_context| {
        let instruction_context = invoke_context
            .transaction_context
            .get_current_instruction_context()?;
        let program_id = *instruction_context.get_program_key()?;
        let mut accounts = Vec::new();
        for index in 0..instruction_context.get_number_of_instruction_accounts() {
            accounts.push(RecordedAccount {
                pubkey: *instruction_context.get_key_of_instruction_account(index)?,
                is_signer: instruction_context.is_instruction_account_signer(index)?,
                is_writable: instruction_context.is_instruction_account_writable(index)?,
            });
        }
        let data = instruction_context.get_instruction_data().to_vec();
        RECORDED.with(|recorded| {
            recorded.borrow_mut().push(RecordedInvocation {
                program_id,
                accounts,
                data,
            })
        });
        RECORDER_RESULT.with(|result| result.borrow().clone())
    }
);

/// The SPL Noop builtin: accepts anything and returns `Ok(())`.
pub fn noop_builtin() -> Builtin {
    Builtin {
        program_id: SPL_NOOP_ID,
        name: SPL_NOOP_NAME,
        entrypoint: NoopEntrypoint::vm,
    }
}

/// The recorder builtin: logs what it receives into a thread local.
pub fn recorder_builtin() -> Builtin {
    Builtin {
        program_id: RECORDER_ID,
        name: RECORDER_NAME,
        entrypoint: RecorderEntrypoint::vm,
    }
}

/// Registers the auxiliary builtins on a Mollusk instance.
pub fn register_builtins(mollusk: &mut Mollusk) {
    mollusk.program_cache.add_builtin(noop_builtin());
    mollusk.program_cache.add_builtin(recorder_builtin());
}

/// Returns and clears the invocations the recorder saw on this thread.
pub fn take_recorded() -> Vec<RecordedInvocation> {
    RECORDED.with(|recorded| std::mem::take(&mut *recorded.borrow_mut()))
}

/// Discards the invocations the recorder saw on this thread.
pub fn clear_recorded() {
    RECORDED.with(|recorded| recorded.borrow_mut().clear());
}

/// Sets what the recorder returns on this thread from now on (default `Ok`).
pub fn set_recorder_result(result: Result<(), InstructionError>) {
    RECORDER_RESULT.with(|slot| *slot.borrow_mut() = result);
}

/// The executable accounts of every program the harness knows about, in the
/// form the current mode (native or SBF) expects: mpl-core, the system
/// program, spl-noop and the recorder.
pub fn keyed_program_accounts() -> Vec<(Pubkey, Account)> {
    vec![
        (MPL_CORE_ID, core_program_account()),
        keyed_account_for_system_program(),
        create_keyed_account_for_builtin_program(&SPL_NOOP_ID, SPL_NOOP_NAME),
        create_keyed_account_for_builtin_program(&RECORDER_ID, RECORDER_NAME),
    ]
}

/// Replaces any entry for a known program id with the correct executable
/// account and appends the ones that are missing.
///
/// Use this on the account list handed to `Mollusk::process_instruction`;
/// `Fixture` (a `MolluskContext`) hydrates program accounts by itself.
pub fn with_program_accounts(accounts: Vec<(Pubkey, Account)>) -> Vec<(Pubkey, Account)> {
    let programs = keyed_program_accounts();
    let mut result: Vec<(Pubkey, Account)> = accounts
        .into_iter()
        .filter(|(key, _)| !programs.iter().any(|(program, _)| program == key))
        .collect();
    result.extend(programs);
    result
}

/// The ELF name mpl-core is registered under (re-exported for callers that
/// build their own program account).
pub const CORE_PROGRAM_NAME: &str = PROGRAM_NAME;
