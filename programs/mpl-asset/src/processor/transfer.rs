use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, program::invoke};

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

    match load_key(ctx.accounts.asset_address, 0)? {
        Key::HashedAsset => {
            // TODO: Needs to be in helper.
            // Check that arguments passed in result in on-chain hash.
            let mut asset = Asset::from(args.compression_proof);
            let args_asset_hash = asset.hash()?;
            let current_account_hash = HashedAsset::load(ctx.accounts.asset_address, 0)?.hash;
            if args_asset_hash != current_account_hash {
                return Err(MplAssetError::IncorrectAssetHash.into());
            }

            // TODO: Needs to be in helper.
            // Update owner and send Noop instruction.
            asset.owner = *ctx.accounts.new_owner.key;
            let serialized_data = asset.try_to_vec()?;
            invoke(&spl_noop::instruction(serialized_data), &[])?;

            // Make a new hashed asset with updated owner.
            HashedAsset::new(asset.hash()?).save(ctx.accounts.asset_address, 0)
        }
        Key::Asset => {
            let mut asset = Asset::load(ctx.accounts.asset_address, 0)?;
            asset.owner = *ctx.accounts.new_owner.key;
            asset.save(ctx.accounts.asset_address, 0)
        }
        _ => Err(MplAssetError::IncorrectAccount.into()),
    }
}
