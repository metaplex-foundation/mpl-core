use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_memory::sol_memcpy, system_instruction, system_program,
};

use crate::{
    error::MplAssetError,
    instruction::accounts::TransferAccounts,
    state::{Asset, Compressible, DataState, HashedAsset, Key},
    utils::DataBlob,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum MigrationLevel {
    MigrateOnly,
    MigrateAndBurn,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct TransferArgs {}

// // danenbm working on
// #[account(0, writable, name="asset_address", desc = "The address of the asset")]
// #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
// #[account(2, signer, name="authority", desc = "The owner or delegate of the asset")]
// #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
// #[account(4, name="new_owner", desc = "The new owner to which to transfer the asset")]
// #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
// Transfer(TransferArgs),

pub(crate) fn transfer<'a>(accounts: &'a [AccountInfo<'a>], args: TransferArgs) -> ProgramResult {
    // Accounts.
    let ctx = TransferAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;

    if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
    }

    let asset = Asset::load(ctx.accounts.asset_address, 0)?;

    asset.owner = ctx.accounts.new_owner.key();

    // if *ctx.accounts.system_program.key != system_program::id() {
    //     return Err(MplAssetError::InvalidSystemProgram.into());
    // }

    // let updated_asset = Asset {
    //     key: Key::Asset,
    //     update_authority: *ctx
    //         .accounts
    //         .update_authority
    //         .unwrap_or(ctx.accounts.payer)
    //         .key,
    //     owner: *ctx
    //         .accounts
    //         .owner
    //         .unwrap_or(ctx.accounts.update_authority.unwrap_or(ctx.accounts.payer))
    //         .key,
    //     name: args.name,
    //     uri: args.uri,
    // };

    // let serialized_data = new_asset.try_to_vec()?;

    // let serialized_data = match args.data_state {
    //     DataState::AccountState => serialized_data,
    //     DataState::LedgerState => {
    //         invoke(&spl_noop::instruction(serialized_data.clone()), &[])?;

    //         let hashed_asset = HashedAsset {
    //             key: Key::HashedAsset,
    //             hash: new_asset.hash()?,
    //         };

    //         hashed_asset.try_to_vec()?
    //     }
    // };

    //let lamports = rent.minimum_balance(serialized_data.len());

    // CPI to the System Program.
    // invoke(
    //     &system_instruction::create_account(
    //         ctx.accounts.payer.key,
    //         ctx.accounts.asset_address.key,
    //         lamports,
    //         serialized_data.len() as u64,
    //         &crate::id(),
    //     ),
    //     &[
    //         ctx.accounts.payer.clone(),
    //         ctx.accounts.asset_address.clone(),
    //         ctx.accounts.system_program.clone(),
    //     ],
    // )?;

    // sol_memcpy(
    //     &mut ctx.accounts.asset_address.try_borrow_mut_data()?,
    //     &serialized_data,
    //     serialized_data.len(),
    // );

    Ok(())
}
