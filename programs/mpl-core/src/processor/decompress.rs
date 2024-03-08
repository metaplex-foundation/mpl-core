use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, system_program};

use crate::{
    error::MplCoreError,
    instruction::accounts::DecompressAccounts,
    plugins::{Plugin, PluginType},
    state::{Asset, Collection, CompressionProof, Key},
    utils::{
        load_key, rebuild_account_state_from_proof_data, validate_asset_permissions, verify_proof,
    },
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
    assert_signer(ctx.accounts.authority)?;
    let payer = if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
        payer
    } else {
        ctx.accounts.authority
    };

    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    match load_key(ctx.accounts.asset, 0)? {
        Key::HashedAsset => {
            // Verify the proof and rebuild Asset struct in account space.
            let (asset, plugins) = verify_proof(ctx.accounts.asset, &args.compression_proof)?;

            // Use the data from the compression proof to rebuild the account.
            rebuild_account_state_from_proof_data(
                asset,
                plugins,
                ctx.accounts.asset,
                payer,
                ctx.accounts.system_program,
            )?;

            // Validate asset permissions.
            let _ = validate_asset_permissions(
                ctx.accounts.authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                None,
                Asset::check_decompress,
                Collection::check_decompress,
                PluginType::check_decompress,
                Asset::validate_decompress,
                Collection::validate_decompress,
                Plugin::validate_decompress,
            )?;

            // TODO Enable compression.
            msg!("Error: Decompression currently not available");
            Err(MplCoreError::NotAvailable.into())
        }
        Key::Asset => Err(MplCoreError::AlreadyDecompressed.into()),
        _ => Err(MplCoreError::IncorrectAccount.into()),
    }
}
