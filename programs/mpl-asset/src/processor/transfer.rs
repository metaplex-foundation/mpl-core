use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_memory::sol_memcpy,
};

use crate::{
    error::MplAssetError,
    instruction::accounts::TransferAccounts,
    state::{Asset, Compressible, CompressionProof, HashedAsset, Key},
    utils::{load_key, DataBlob},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct TransferArgs {
    compression_proof: CompressionProof,
}

pub(crate) fn transfer<'a>(accounts: &'a [AccountInfo<'a>], args: TransferArgs) -> ProgramResult {
    // Accounts.
    let ctx = TransferAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
    }

    let serialized_data = match load_key(ctx.accounts.asset_address, 0)? {
        Key::HashedAsset => {
            // Check that arguments passed in result in on-chain hash.
            let mut asset = Asset::from(args.compression_proof);
            let args_asset_hash = asset.hash()?;
            let current_account_hash = HashedAsset::load(ctx.accounts.asset_address, 0)?.hash;
            if args_asset_hash != current_account_hash {
                return Err(MplAssetError::IncorrectAssetHash.into());
            }

            // Update owner and send Noop instruction.
            asset.owner = *ctx.accounts.new_owner.key;
            let serialized_data = asset.try_to_vec()?;
            invoke(&spl_noop::instruction(serialized_data), &[])?;

            // Make a new hashed asset with updated owner.
            HashedAsset::new(asset.hash()?).try_to_vec()?
        }
        Key::Asset => {
            let mut asset = Asset::load(ctx.accounts.asset_address, 0)?;
            asset.owner = *ctx.accounts.new_owner.key;
            asset.try_to_vec()?
        }
        _ => return Err(MplAssetError::IncorrectAccount.into()),
    };

    sol_memcpy(
        &mut ctx.accounts.asset_address.try_borrow_mut_data()?,
        &serialized_data,
        serialized_data.len(),
    );

    Ok(())
}
