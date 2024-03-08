use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_memory::sol_memcpy, rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::CreateCollectionAccounts,
    plugins::{create_meta_idempotent, initialize_plugin, Plugin},
    state::{Collection, Key},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CreateCollectionArgs {
    pub name: String,
    pub uri: String,
    pub plugins: Vec<Plugin>,
}

pub(crate) fn create_collection<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: CreateCollectionArgs,
) -> ProgramResult {
    // Accounts.
    let ctx = CreateCollectionAccounts::context(accounts)?;
    let rent = Rent::get()?;

    // Guards.
    assert_signer(ctx.accounts.collection)?;
    assert_signer(ctx.accounts.payer)?;

    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    let new_collection = Collection {
        key: Key::Collection,
        update_authority: *ctx
            .accounts
            .update_authority
            .unwrap_or(ctx.accounts.payer)
            .key,
        name: args.name,
        uri: args.uri,
        num_minted: 0,
        current_size: 0,
    };

    let serialized_data = new_collection.try_to_vec()?;

    let lamports = rent.minimum_balance(serialized_data.len());

    // CPI to the System Program.
    invoke(
        &system_instruction::create_account(
            ctx.accounts.payer.key,
            ctx.accounts.collection.key,
            lamports,
            serialized_data.len() as u64,
            &crate::id(),
        ),
        &[
            ctx.accounts.payer.clone(),
            ctx.accounts.collection.clone(),
            ctx.accounts.system_program.clone(),
        ],
    )?;

    sol_memcpy(
        &mut ctx.accounts.collection.try_borrow_mut_data()?,
        &serialized_data,
        serialized_data.len(),
    );

    drop(serialized_data);

    create_meta_idempotent::<Collection>(
        ctx.accounts.collection,
        ctx.accounts.payer,
        ctx.accounts.system_program,
    )?;

    for plugin in args.plugins {
        initialize_plugin::<Collection>(
            &plugin,
            &plugin.manager(),
            ctx.accounts.collection,
            ctx.accounts.payer,
            ctx.accounts.system_program,
        )?;
    }

    Ok(())
}
