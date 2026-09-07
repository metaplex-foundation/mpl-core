//! Shared Mollusk harness for the mpl-core integration tests.
//!
//! By default the program under test is the SBF ELF (`mpl_core_program.so`)
//! that Mollusk loads from `SBF_OUT_DIR`, exactly as it runs on-chain.
//!
//! When the tests are compiled by `cargo llvm-cov` (which sets
//! `cfg(coverage)`), or when `MPL_CORE_NATIVE_PROGRAM=1` is set at runtime,
//! the program is instead registered with Mollusk as a native builtin that
//! calls the host-compiled entrypoint. LLVM source-based coverage
//! instrumentation only observes host code, so this is what lets the
//! coverage report attribute every line the Mollusk tests exercise back to
//! `programs/mpl-core/src`. The account serialization, CPI, sysvar and
//! return-data plumbing mirrors what `solana-program-test` does for its
//! `processor!` builtins, so the program sees the same ABI in both modes.
//!
//! Every test module includes this file with `mod common;` and gets a
//! `Mollusk` from [`core_mollusk`].
#![allow(dead_code)]

use {mollusk_svm::Mollusk, mpl_core_program::ID as MPL_CORE_ID, solana_account::Account};

/// Name of the program ELF (without extension) that Mollusk loads in SBF mode.
pub const PROGRAM_NAME: &str = "mpl_core_program";

/// Environment variable that forces the native (host-compiled) program.
pub const NATIVE_PROGRAM_ENV: &str = "MPL_CORE_NATIVE_PROGRAM";

/// Whether the tests should run the host-compiled program instead of the
/// SBF ELF.
pub fn native_program_enabled() -> bool {
    if cfg!(coverage) {
        return true;
    }
    match std::env::var(NATIVE_PROGRAM_ENV) {
        Ok(value) => !matches!(value.trim(), "" | "0" | "false" | "no" | "off"),
        Err(_) => false,
    }
}

/// Creates a Mollusk instance with the mpl-core program registered.
pub fn core_mollusk() -> Mollusk {
    if native_program_enabled() {
        native::mollusk()
    } else {
        Mollusk::new(&MPL_CORE_ID, PROGRAM_NAME)
    }
}

/// Creates the executable account for the mpl-core program itself, for
/// tests that pass the program ID as an optional-account sentinel.
///
/// The account has to match how the program is registered with Mollusk: a
/// BPF upgradeable loader program account in SBF mode, and a native-loader
/// builtin account in native mode (otherwise the runtime would route the
/// invocation through the BPF loader, which cannot execute a builtin).
pub fn core_program_account() -> Account {
    if native_program_enabled() {
        mollusk_svm::program::create_keyed_account_for_builtin_program(&MPL_CORE_ID, PROGRAM_NAME).1
    } else {
        mollusk_svm::program::create_program_account_loader_v3(&MPL_CORE_ID)
    }
}

/// Runs the host-compiled program inside Mollusk as a native builtin.
pub mod native {
    use {
        super::{MPL_CORE_ID, PROGRAM_NAME},
        mollusk_svm::{program::Builtin, Mollusk},
        solana_program::{
            account_info::AccountInfo,
            entrypoint::{deserialize, ProgramResult},
            instruction::{Instruction, InstructionError},
            program_error::{ProgramError, UNSUPPORTED_SYSVAR},
            pubkey::Pubkey,
        },
        solana_program_runtime::{
            declare_process_instruction,
            invoke_context::InvokeContext,
            serialization::{deserialize_parameters, serialize_parameters},
            stable_log,
        },
        solana_svm_timings::ExecuteTimings,
        solana_sysvar::{
            program_stubs::{set_syscall_stubs, SyscallStubs},
            SysvarSerialize,
        },
        std::{cell::RefCell, mem::transmute, panic::AssertUnwindSafe, sync::Once},
    };

    /// Compute units charged per native invocation. Native execution is not
    /// metered, so this is only what Mollusk reports as consumed.
    const NATIVE_COMPUTE_UNITS: u64 = 1;

    /// Creates a Mollusk instance that executes the host-compiled program.
    pub fn mollusk() -> Mollusk {
        install_syscall_stubs();
        let mut mollusk = Mollusk::default();
        mollusk.program_cache.add_builtin(Builtin {
            program_id: MPL_CORE_ID,
            name: PROGRAM_NAME,
            entrypoint: Entrypoint::vm,
        });
        mollusk
    }

    /// Installs the syscall stubs once per process.
    fn install_syscall_stubs() {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            set_syscall_stubs(Box::new(NativeSyscallStubs));
        });
    }

    // The syscall stubs are free functions with no access to the runtime, so
    // the active `InvokeContext` is stashed in a thread local for the
    // duration of the native invocation. This is the same approach that
    // `solana-program-test` uses.
    thread_local! {
        static INVOKE_CONTEXT: RefCell<Option<usize>> = const { RefCell::new(None) };
    }

    /// Records the invoke context of the invocation that is about to run.
    fn set_invoke_context(new: &mut InvokeContext) {
        INVOKE_CONTEXT.with(|invoke_context| unsafe {
            invoke_context.replace(Some(transmute::<&mut InvokeContext, usize>(new)))
        });
    }

    /// Returns the invoke context recorded by [`set_invoke_context`].
    fn get_invoke_context<'a, 'b, 'c>() -> &'a mut InvokeContext<'b, 'c> {
        let ptr = INVOKE_CONTEXT.with(|invoke_context| match *invoke_context.borrow() {
            Some(val) => val,
            None => panic!("Invoke context not set!"),
        });
        unsafe { transmute::<usize, &mut InvokeContext>(ptr) }
    }

    declare_process_instruction!(Entrypoint, NATIVE_COMPUTE_UNITS, |invoke_context| {
        process_native_instruction(invoke_context)
    });

    /// Serializes the instruction accounts with the SBF ABI, runs the
    /// program's Rust entrypoint on them and commits the account changes
    /// back into the transaction context, like the BPF loader does after a
    /// VM execution.
    fn process_native_instruction(
        invoke_context: &mut InvokeContext,
    ) -> Result<(), InstructionError> {
        set_invoke_context(invoke_context);

        let mask_out_rent_epoch_in_vm_serialization = invoke_context
            .get_feature_set()
            .mask_out_rent_epoch_in_vm_serialization;

        let (mut parameter_bytes, _regions, accounts_metadata, _instruction_data_offset) = {
            let instruction_context = invoke_context
                .transaction_context
                .get_current_instruction_context()?;
            serialize_parameters(
                &instruction_context,
                /* stricter_abi_and_runtime_constraints */ false,
                /* account_data_direct_mapping */ false,
                mask_out_rent_epoch_in_vm_serialization,
            )?
        };

        let result = {
            // SAFETY: `parameter_bytes` was produced by `serialize_parameters`
            // in the aligned SBF ABI layout and outlives the returned views.
            let (program_id, account_infos, instruction_data) =
                unsafe { deserialize(parameter_bytes.as_slice_mut().as_mut_ptr()) };

            std::panic::catch_unwind(AssertUnwindSafe(|| {
                mpl_core_program::entrypoint::process_instruction(
                    program_id,
                    &account_infos,
                    instruction_data,
                )
            }))
        };

        match result {
            Ok(Ok(())) => {}
            Ok(Err(program_error)) => {
                return Err(InstructionError::from(u64::from(program_error)));
            }
            Err(_panic) => {
                return Err(InstructionError::ProgramFailedToComplete);
            }
        }

        // The syscall stubs may have replaced the invoke context reference
        // during a CPI, so re-fetch the instruction context before committing.
        let instruction_context = invoke_context
            .transaction_context
            .get_current_instruction_context()?;
        deserialize_parameters(
            &instruction_context,
            /* stricter_abi_and_runtime_constraints */ false,
            /* account_data_direct_mapping */ false,
            parameter_bytes.as_slice(),
            &accounts_metadata,
        )
    }

    /// Copies a sysvar from the runtime's sysvar cache into the program's
    /// buffer, charging the same compute cost as the real syscall.
    fn get_sysvar<T: Default + SysvarSerialize + Sized + Clone>(
        sysvar: Result<std::sync::Arc<T>, InstructionError>,
        var_addr: *mut u8,
    ) -> u64 {
        let invoke_context = get_invoke_context();
        if invoke_context
            .consume_checked(
                invoke_context.get_execution_cost().sysvar_base_cost + T::size_of() as u64,
            )
            .is_err()
        {
            panic!("Exceeded compute budget");
        }

        match sysvar {
            Ok(sysvar_data) => unsafe {
                *(var_addr as *mut _ as *mut T) = T::clone(&sysvar_data);
                solana_program::entrypoint::SUCCESS
            },
            Err(_) => UNSUPPORTED_SYSVAR,
        }
    }

    /// Routes the host `solana_program` syscall shims into the active
    /// Mollusk `InvokeContext`.
    struct NativeSyscallStubs;

    impl SyscallStubs for NativeSyscallStubs {
        fn sol_log(&self, message: &str) {
            let invoke_context = get_invoke_context();
            stable_log::program_log(&invoke_context.get_log_collector(), message);
        }

        fn sol_invoke_signed(
            &self,
            instruction: &Instruction,
            account_infos: &[AccountInfo],
            signers_seeds: &[&[&[u8]]],
        ) -> ProgramResult {
            let invoke_context = get_invoke_context();
            let transaction_context = &invoke_context.transaction_context;
            let instruction_context = transaction_context
                .get_current_instruction_context()
                .unwrap();
            let caller = instruction_context.get_program_key().unwrap();

            let signers = signers_seeds
                .iter()
                .map(|seeds| Pubkey::create_program_address(seeds, caller).unwrap())
                .collect::<Vec<_>>();

            invoke_context
                .prepare_next_instruction(instruction.clone(), &signers)
                .unwrap();

            // Copy the caller's `AccountInfo` modifications into the
            // transaction context before the callee runs.
            let transaction_context = &invoke_context.transaction_context;
            let instruction_context = transaction_context
                .get_current_instruction_context()
                .unwrap();
            let next_instruction_context =
                transaction_context.get_next_instruction_context().unwrap();
            let next_instruction_accounts = next_instruction_context.instruction_accounts();
            let mut account_indices = Vec::with_capacity(next_instruction_accounts.len());
            for instruction_account in next_instruction_accounts.iter() {
                let account_key = transaction_context
                    .get_key_of_account_at_index(instruction_account.index_in_transaction)
                    .unwrap();
                let account_info_index = account_infos
                    .iter()
                    .position(|account_info| account_info.unsigned_key() == account_key)
                    .ok_or(InstructionError::MissingAccount)
                    .unwrap();
                let account_info = &account_infos[account_info_index];
                let index_in_caller = instruction_context
                    .get_index_of_account_in_instruction(instruction_account.index_in_transaction)
                    .unwrap();
                let mut borrowed_account = instruction_context
                    .try_borrow_instruction_account(index_in_caller)
                    .unwrap();
                if borrowed_account.get_lamports() != account_info.lamports() {
                    borrowed_account
                        .set_lamports(account_info.lamports())
                        .unwrap();
                }
                let account_info_data = account_info.try_borrow_data().unwrap();
                // The redundant check avoids the expensive data comparison
                // when the data can simply be written.
                match borrowed_account.can_data_be_resized(account_info_data.len()) {
                    Ok(()) => borrowed_account
                        .set_data_from_slice(&account_info_data)
                        .unwrap(),
                    Err(err) if borrowed_account.get_data() != *account_info_data => {
                        panic!("{err:?}");
                    }
                    _ => {}
                }
                // Change the owner last so the lamport and data writes above
                // are still permitted.
                if borrowed_account.get_owner() != account_info.owner {
                    borrowed_account
                        .set_owner(account_info.owner.as_ref())
                        .unwrap();
                }
                if instruction_account.is_writable() {
                    account_indices
                        .push((instruction_account.index_in_transaction, account_info_index));
                }
            }

            let mut compute_units_consumed = 0;
            invoke_context
                .process_instruction(&mut compute_units_consumed, &mut ExecuteTimings::default())
                .map_err(|err| {
                    ProgramError::try_from(err).unwrap_or_else(|err| panic!("{}", err))
                })?;

            // Copy the callee's modifications back into the caller's
            // `AccountInfo`s.
            let transaction_context = &invoke_context.transaction_context;
            let instruction_context = transaction_context
                .get_current_instruction_context()
                .unwrap();
            for (index_in_transaction, account_info_index) in account_indices.into_iter() {
                let index_in_caller = instruction_context
                    .get_index_of_account_in_instruction(index_in_transaction)
                    .unwrap();
                let borrowed_account = instruction_context
                    .try_borrow_instruction_account(index_in_caller)
                    .unwrap();
                let account_info = &account_infos[account_info_index];
                **account_info.try_borrow_mut_lamports().unwrap() = borrowed_account.get_lamports();
                if account_info.owner != borrowed_account.get_owner() {
                    // `AccountInfo::owner` is an immutable reference into the
                    // serialized parameter buffer; the callee (e.g. the system
                    // program assigning an account) legitimately changes it.
                    #[allow(clippy::transmute_ptr_to_ptr)]
                    #[allow(mutable_transmutes)]
                    let account_info_mut =
                        unsafe { transmute::<&Pubkey, &mut Pubkey>(account_info.owner) };
                    *account_info_mut = *borrowed_account.get_owner();
                }

                let new_data = borrowed_account.get_data();
                let new_len = new_data.len();

                if account_info.data_len() != new_len {
                    account_info.resize(new_len)?;
                }

                let mut data = account_info.try_borrow_mut_data()?;
                data.clone_from_slice(new_data);
            }

            Ok(())
        }

        fn sol_get_clock_sysvar(&self, var_addr: *mut u8) -> u64 {
            get_sysvar(
                get_invoke_context().get_sysvar_cache().get_clock(),
                var_addr,
            )
        }

        fn sol_get_epoch_schedule_sysvar(&self, var_addr: *mut u8) -> u64 {
            get_sysvar(
                get_invoke_context().get_sysvar_cache().get_epoch_schedule(),
                var_addr,
            )
        }

        fn sol_get_epoch_rewards_sysvar(&self, var_addr: *mut u8) -> u64 {
            get_sysvar(
                get_invoke_context().get_sysvar_cache().get_epoch_rewards(),
                var_addr,
            )
        }

        #[allow(deprecated)]
        fn sol_get_fees_sysvar(&self, var_addr: *mut u8) -> u64 {
            get_sysvar(get_invoke_context().get_sysvar_cache().get_fees(), var_addr)
        }

        fn sol_get_rent_sysvar(&self, var_addr: *mut u8) -> u64 {
            get_sysvar(get_invoke_context().get_sysvar_cache().get_rent(), var_addr)
        }

        fn sol_get_last_restart_slot(&self, var_addr: *mut u8) -> u64 {
            get_sysvar(
                get_invoke_context()
                    .get_sysvar_cache()
                    .get_last_restart_slot(),
                var_addr,
            )
        }

        fn sol_get_return_data(&self) -> Option<(Pubkey, Vec<u8>)> {
            let (program_id, data) = get_invoke_context().transaction_context.get_return_data();
            // The syscall reports "no return data" as a zero length, which
            // `get_return_data` surfaces as `None`; mirror that here.
            if data.is_empty() {
                None
            } else {
                Some((*program_id, data.to_vec()))
            }
        }

        fn sol_set_return_data(&self, data: &[u8]) {
            let invoke_context = get_invoke_context();
            let transaction_context = &mut invoke_context.transaction_context;
            let instruction_context = transaction_context
                .get_current_instruction_context()
                .unwrap();
            let caller = *instruction_context.get_program_key().unwrap();
            transaction_context
                .set_return_data(caller, data.to_vec())
                .unwrap();
        }

        fn sol_get_stack_height(&self) -> u64 {
            let invoke_context = get_invoke_context();
            invoke_context.get_stack_height().try_into().unwrap()
        }
    }
}
