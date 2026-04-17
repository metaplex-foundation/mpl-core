use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    instruction::accounts::UpdateCollectionInfoV1Accounts,
    state::{CollectionV1, SolanaAccount},
    utils::fetch_core_data,
};

pub const BUBBLEGUM_SIGNER: Pubkey =
    solana_program::pubkey!("CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk");

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) enum UpdateType {
    /// Update collection details due to minting assets to collection.
    Mint,
    /// Update collection details due to adding assets to collection.
    Add,
    /// Update collection details due to removing assets from collection.
    Remove,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateCollectionInfoV1Args {
    update_type: UpdateType,
    amount: u32,
}

pub(crate) fn update_collection_info<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdateCollectionInfoV1Args,
) -> ProgramResult {
    // Accounts.
    let ctx = UpdateCollectionInfoV1Accounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.bubblegum_signer)?;

    // This instruction can only be called by the Bubblegum program.
    if *ctx.accounts.bubblegum_signer.key != BUBBLEGUM_SIGNER {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    let (mut collection, _, _) = fetch_core_data::<CollectionV1>(ctx.accounts.collection)?;

    match args.update_type {
        UpdateType::Mint => {
            collection.num_minted = collection.num_minted.saturating_add(args.amount);
            collection.current_size = collection.current_size.saturating_add(args.amount);
        }
        UpdateType::Add => {
            collection.current_size = collection.current_size.saturating_add(args.amount);
        }
        UpdateType::Remove => {
            collection.current_size = collection.current_size.saturating_sub(args.amount);
        }
    }

    collection.save(ctx.accounts.collection, 0)?;

    Ok(())
}
