use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    program_error::PrintProgramError, pubkey::Pubkey,
};

use crate::{error::MplCoreError, processor};

// START: Heap start
// LENGTH: Heap length
// MIN: Minimal allocation size
// PAGE_SIZE: Allocation page size
#[cfg(target_os = "solana")]
#[global_allocator]
static ALLOC: smalloc::Smalloc<
    { solana_program::entrypoint::HEAP_START_ADDRESS as usize },
    { solana_program::entrypoint::HEAP_LENGTH as usize },
    16,
    1024,
> = smalloc::Smalloc::new();

entrypoint!(process_instruction);

/// Entrypoint function
fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = processor::process_instruction(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<MplCoreError>();
        return Err(error);
    }
    Ok(())
}
