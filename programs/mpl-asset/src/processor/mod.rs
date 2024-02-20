use crate::instruction::MplAssetInstruction;
use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

mod create;
pub(crate) use create::*;

mod delegate;
pub(crate) use delegate::*;

mod revoke;
pub(crate) use revoke::*;

mod burn;
pub(crate) use burn::*;

mod transfer;
pub(crate) use transfer::*;

mod update;
pub(crate) use update::*;

mod freeze;
pub(crate) use freeze::*;

mod thaw;
pub(crate) use thaw::*;

mod compress;
pub(crate) use compress::*;

mod decompress;
pub(crate) use decompress::*;

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
        MplAssetInstruction::Delegate(args) => {
            msg!("Instruction: Delegate");
            delegate(accounts, args)
        }
        MplAssetInstruction::Revoke(args) => {
            msg!("Instruction: Revoke");
            revoke(accounts, args)
        }
        MplAssetInstruction::Burn(args) => {
            msg!("Instruction: Burn");
            burn(accounts, args)
        }
        MplAssetInstruction::Transfer(args) => {
            msg!("Instruction: Transfer");
            transfer(accounts, args)
        }
        MplAssetInstruction::Update(args) => {
            msg!("Instruction: Update");
            update(accounts, args)
        }
        MplAssetInstruction::Freeze(args) => {
            msg!("Instruction: Freeze");
            freeze(accounts, args)
        }
        MplAssetInstruction::Thaw(args) => {
            msg!("Instruction: Thaw");
            thaw(accounts, args)
        }
        MplAssetInstruction::Compress(args) => {
            msg!("Instruction: Compress");
            compress(accounts, args)
        }
        MplAssetInstruction::Decompress(args) => {
            msg!("Instruction: Decompress");
            decompress(accounts, args)
        }
    }
}
