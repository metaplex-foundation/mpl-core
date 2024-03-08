use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memcpy,
    system_program,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::DecompressAccounts,
    plugins::{create_meta_idempotent, initialize_plugin, Plugin, PluginType},
    state::{Asset, Collection, CompressionProof, Key},
    utils::{load_key, resize_or_reallocate_account, validate_asset_permissions, verify_proof},
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
            // Verify the proof and rebuild Asset struct in account.
            let (asset, plugins) = verify_proof(ctx.accounts.asset, &args.compression_proof)?;

            let serialized_data = asset.try_to_vec()?;
            resize_or_reallocate_account(
                ctx.accounts.asset,
                payer,
                ctx.accounts.system_program,
                serialized_data.len(),
            )?;

            sol_memcpy(
                &mut ctx.accounts.asset.try_borrow_mut_data()?,
                &serialized_data,
                serialized_data.len(),
            );

            // Add the plugins.
            if !plugins.is_empty() {
                create_meta_idempotent(ctx.accounts.asset, payer, ctx.accounts.system_program)?;

                for plugin in plugins {
                    initialize_plugin::<Asset>(
                        &plugin.plugin,
                        &plugin.authority,
                        ctx.accounts.asset,
                        payer,
                        ctx.accounts.system_program,
                    )?;
                }
            }

            // Validate permissions.
            //let (asset, _, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

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
        }
        Key::Asset => return Err(MplCoreError::AlreadyDecompressed.into()),
        _ => return Err(MplCoreError::IncorrectAccount.into()),
    }

    Ok(())
}
