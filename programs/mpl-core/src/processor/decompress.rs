use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use mpl_utils::resize_or_reallocate_account_raw;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::DecompressAccounts,
    state::{Asset, CompressionProof, Key},
    utils::load_key,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct DecompressArgs {
    compression_proof: CompressionProof,
}

pub(crate) fn decompress<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: DecompressArgs,
) -> ProgramResult {
    // Accounts.
    let ctx = DecompressAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.owner)?;
    let payer = if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
        payer
    } else {
        ctx.accounts.owner
    };

    match load_key(ctx.accounts.asset_address, 0)? {
        Key::HashedAsset => {
            // TODO: Decompress assets with plugins.  Right now this would fail for anything that's
            // not just a plain asset.
            let asset = Asset::verify_proof(ctx.accounts.asset_address, args.compression_proof)?;

            if ctx.accounts.owner.key != &asset.owner {
                return Err(MplCoreError::InvalidAuthority.into());
            }

            // TODO: Delegated compress/decompress authority.

            let serialized_data = asset.try_to_vec()?;

            resize_or_reallocate_account_raw(
                ctx.accounts.asset_address,
                payer,
                ctx.accounts.system_program,
                serialized_data.len(),
            )?;

            sol_memcpy(
                &mut ctx.accounts.asset_address.try_borrow_mut_data()?,
                &serialized_data,
                serialized_data.len(),
            );
        }
        Key::Asset => return Err(MplCoreError::AlreadyDecompressed.into()),
        _ => return Err(MplCoreError::IncorrectAccount.into()),
    }

    Ok(())
}
