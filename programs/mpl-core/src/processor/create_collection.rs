use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_memory::sol_memcpy, rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::CreateCollectionV1Accounts,
    plugins::{create_meta_idempotent, initialize_plugin, PluginAuthorityPair},
    state::{CollectionV1, Key},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct CreateCollectionV1Args {
    pub(crate) name: String,
    pub(crate) uri: String,
    pub(crate) plugins: Option<Vec<PluginAuthorityPair>>,
}

pub(crate) fn create_collection<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: CreateCollectionV1Args,
) -> ProgramResult {
    // Accounts.
    let ctx = CreateCollectionV1Accounts::context(accounts)?;
    let rent = Rent::get()?;

    // Guards.
    assert_signer(ctx.accounts.collection)?;
    assert_signer(ctx.accounts.payer)?;

    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    let new_collection = CollectionV1 {
        key: Key::CollectionV1,
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

    create_meta_idempotent::<CollectionV1>(
        ctx.accounts.collection,
        ctx.accounts.payer,
        ctx.accounts.system_program,
    )?;

    for plugin in args.plugins.unwrap_or_default() {
        initialize_plugin::<CollectionV1>(
            &plugin.plugin,
            &plugin.authority.unwrap_or(plugin.plugin.manager()),
            ctx.accounts.collection,
            ctx.accounts.payer,
            ctx.accounts.system_program,
        )?;
    }

    Ok(())
}
