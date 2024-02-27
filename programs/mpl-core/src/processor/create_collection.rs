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
    state::{CollectionData, Key},
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
    assert_signer(ctx.accounts.collection_address)?;
    assert_signer(ctx.accounts.payer)?;

    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    let new_collection = CollectionData {
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
            ctx.accounts.collection_address.key,
            lamports,
            serialized_data.len() as u64,
            &crate::id(),
        ),
        &[
            ctx.accounts.payer.clone(),
            ctx.accounts.collection_address.clone(),
            ctx.accounts.system_program.clone(),
        ],
    )?;

    sol_memcpy(
        &mut ctx.accounts.collection_address.try_borrow_mut_data()?,
        &serialized_data,
        serialized_data.len(),
    );

    drop(serialized_data);

    solana_program::msg!("Collection created.");
    create_meta_idempotent(
        ctx.accounts.collection_address,
        ctx.accounts.payer,
        ctx.accounts.system_program,
    )?;

    solana_program::msg!("Meta created.");

    for plugin in args.plugins {
        initialize_plugin(
            &plugin,
            &plugin.default_authority()?,
            ctx.accounts.collection_address,
            ctx.accounts.payer,
            ctx.accounts.system_program,
        )?;
    }

    solana_program::msg!("Plugins initialized.");

    Ok(())
}
