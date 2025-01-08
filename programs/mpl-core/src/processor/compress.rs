use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::CompressV1Accounts,
    plugins::{Plugin, PluginType},
    state::{AssetV1, CollectionV1, Key, Wrappable},
    utils::{
        compress_into_account_space, fetch_core_data, load_key, resolve_authority,
        validate_asset_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CompressV1Args {}

pub(crate) fn compress<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: CompressV1Args,
) -> ProgramResult {
    // Accounts.
    let ctx = CompressV1Accounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    if ctx.accounts.system_program.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if let Some(log_wrapper) = ctx.accounts.log_wrapper {
        if log_wrapper.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    match load_key(ctx.accounts.asset, 0)? {
        Key::AssetV1 => {
            let (asset, _, plugin_registry) = fetch_core_data::<AssetV1>(ctx.accounts.asset)?;

            // Validate asset permissions.
            let _ = validate_asset_permissions(
                accounts,
                authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                None,
                None,
                None,
                None,
                None,
                None,
                AssetV1::check_compress,
                CollectionV1::check_compress,
                PluginType::check_compress,
                AssetV1::validate_compress,
                CollectionV1::validate_compress,
                Plugin::validate_compress,
                None,
                None,
            )?;

            // Compress the asset and plugin registry into account space.
            let compression_proof = compress_into_account_space(
                asset,
                plugin_registry,
                ctx.accounts.asset,
                ctx.accounts.payer,
                ctx.accounts.system_program,
            )?;

            // Send the spl-noop event for indexing the compressed asset.
            compression_proof.wrap()?;

            // TODO Enable compression.
            msg!("Error: Compression currently not available");
            Err(MplCoreError::NotAvailable.into())
        }
        Key::HashedAssetV1 => Err(MplCoreError::AlreadyCompressed.into()),
        _ => Err(MplCoreError::IncorrectAccount.into()),
    }
}
