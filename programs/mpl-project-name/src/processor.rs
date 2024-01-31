use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke, pubkey::Pubkey,
    rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::error::MplProjectNameError;
use crate::instruction::accounts::CreateAccounts;
use crate::instruction::{CreateArgs, MplProjectNameInstruction};
use crate::state::{Key, MyAccount, MyData};

pub fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction: MplProjectNameInstruction =
        MplProjectNameInstruction::try_from_slice(instruction_data)?;
    match instruction {
        MplProjectNameInstruction::Create(args) => {
            msg!("Instruction: Create");
            create(accounts, args)
        }
    }
}

fn create<'a>(accounts: &'a [AccountInfo<'a>], args: CreateArgs) -> ProgramResult {
    // Accounts.
    let ctx = CreateAccounts::context(accounts)?;
    let rent = Rent::get()?;

    // Guards.
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplProjectNameError::InvalidSystemProgram.into());
    }

    // Fetch the space and minimum lamports required for rent exemption.
    let space: usize = MyAccount::LEN;
    let lamports: u64 = rent.minimum_balance(space);

    // CPI to the System Program.
    invoke(
        &system_instruction::create_account(
            ctx.accounts.payer.key,
            ctx.accounts.address.key,
            lamports,
            space as u64,
            &crate::id(),
        ),
        &[
            ctx.accounts.payer.clone(),
            ctx.accounts.address.clone(),
            ctx.accounts.system_program.clone(),
        ],
    )?;

    let my_account = MyAccount {
        key: Key::MyAccount,
        authority: *ctx.accounts.authority.key,
        data: MyData {
            field1: args.arg1,
            field2: args.arg2,
        },
    };

    my_account.save(ctx.accounts.address)
}
