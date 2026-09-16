//! Result assertions with descriptive failure messages.
//!
//! The helpers that run instructions never panic on a program error; these
//! do, and say what was expected and what actually happened.

use {
    mollusk_svm::result::{InstructionResult, ProgramResult},
    mpl_core_program::{error::MplCoreError, state::Key, ID as MPL_CORE_ID},
    num_traits::FromPrimitive,
    solana_account::Account,
    solana_program::{instruction::InstructionError, program_error::ProgramError, pubkey::Pubkey},
};

/// Describes a program result, decoding `Custom` codes into `MplCoreError`
/// names where possible.
pub fn describe(result: &ProgramResult) -> String {
    match result {
        ProgramResult::Success => "Success".to_string(),
        ProgramResult::Failure(err) => format!("Failure({})", describe_program_error(err)),
        ProgramResult::UnknownError(err) => format!("UnknownError({err:?})"),
    }
}

fn describe_program_error(err: &ProgramError) -> String {
    match err {
        ProgramError::Custom(code) => match MplCoreError::from_u32(*code) {
            Some(core_err) => format!("Custom({code}) = MplCoreError::{core_err:?}"),
            None => format!("Custom({code})"),
        },
        other => format!("{other:?}"),
    }
}

/// Asserts the instruction succeeded.
pub fn assert_ok(result: &InstructionResult) {
    assert!(
        result.program_result.is_ok(),
        "expected the instruction to succeed, got {}",
        describe(&result.program_result)
    );
}

/// Asserts the instruction failed with the given `MplCoreError`
/// (`ProgramError::Custom(err as u32)`).
pub fn assert_core_err(result: &InstructionResult, expected: MplCoreError) {
    let expected_code = expected.clone() as u32;
    match &result.program_result {
        ProgramResult::Failure(ProgramError::Custom(code)) if *code == expected_code => {}
        other => panic!(
            "expected Failure(Custom({expected_code}) = MplCoreError::{expected:?}), got {}",
            describe(other)
        ),
    }
}

/// Asserts the instruction failed with the given `ProgramError`.
pub fn assert_program_err(result: &InstructionResult, expected: ProgramError) {
    match &result.program_result {
        ProgramResult::Failure(err) if *err == expected => {}
        other => panic!(
            "expected Failure({}), got {}",
            describe_program_error(&expected),
            describe(other)
        ),
    }
}

/// Asserts the instruction failed with an `InstructionError` that has no
/// `ProgramError` equivalent (a panic or `unreachable!()` in the program
/// surfaces as `ProgramFailedToComplete`).
pub fn assert_instruction_err(result: &InstructionResult, expected: InstructionError) {
    match &result.program_result {
        ProgramResult::UnknownError(err) if *err == expected => {}
        other => panic!(
            "expected UnknownError({expected:?}), got {}",
            describe(other)
        ),
    }
}

/// The resulting account for `key`; panics if the instruction did not touch it.
pub fn account_of<'a>(result: &'a InstructionResult, key: &Pubkey) -> &'a Account {
    result
        .get_account(key)
        .unwrap_or_else(|| panic!("account {key} is not among the resulting accounts"))
}

/// The resulting lamports of `key`.
pub fn lamports_of(result: &InstructionResult, key: &Pubkey) -> u64 {
    account_of(result, key).lamports
}

/// Asserts `account` is what `close_program_account` leaves behind: exactly
/// one byte of data equal to `Key::Uninitialized`, still owned by mpl-core.
/// (Mollusk's `Check::closed()` expects `Account::default()` and does not
/// match this.)
pub fn assert_account_burned(key: &Pubkey, account: &Account) {
    assert_eq!(
        account.data,
        vec![Key::Uninitialized as u8],
        "account {key} should hold exactly one Uninitialized byte after burning, has {} bytes: {:?}",
        account.data.len(),
        account.data
    );
    assert_eq!(
        account.owner, MPL_CORE_ID,
        "burned account {key} should still be owned by mpl-core"
    );
}

/// Asserts the resulting account for `key` was burned (see
/// [`assert_account_burned`]).
pub fn assert_burned(result: &InstructionResult, key: &Pubkey) {
    assert_account_burned(key, account_of(result, key));
}
