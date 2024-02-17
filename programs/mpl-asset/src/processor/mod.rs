mod create;

use create::*;

use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::instruction::MplAssetInstruction;

pub fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction: MplAssetInstruction = MplAssetInstruction::try_from_slice(instruction_data)?;
    match instruction {
        MplAssetInstruction::Create(args) => {
            msg!("Instruction: Create");
            create(accounts, args)
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
