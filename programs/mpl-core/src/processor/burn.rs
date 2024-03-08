use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::{BurnAccounts, BurnCollectionAccounts},
    plugins::{Plugin, PluginType},
    state::{Asset, Collection, Compressible, CompressionProof, Key},
    utils::{
        close_program_account, load_key, validate_asset_permissions,
        validate_collection_permissions, verify_proof,
    },
};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct BurnArgs {
    compression_proof: Option<CompressionProof>,
}

pub(crate) fn burn<'a>(accounts: &'a [AccountInfo<'a>], args: BurnArgs) -> ProgramResult {
    // Accounts.
    let ctx = BurnAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
    }

    match load_key(ctx.accounts.asset, 0)? {
        Key::HashedAsset => {
            let compression_proof = args
                .compression_proof
                .ok_or(MplCoreError::MissingCompressionProof)?;
            let (asset, _) = verify_proof(ctx.accounts.asset, &compression_proof)?;

            if ctx.accounts.authority.key != &asset.owner {
                return Err(MplCoreError::InvalidAuthority.into());
            }

            // TODO: Check delegates in compressed case.

            asset.wrap()?;
        }
        Key::Asset => {
            let _ = validate_asset_permissions(
                ctx.accounts.authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                None,
                Asset::check_burn,
                Collection::check_burn,
                PluginType::check_burn,
                Asset::validate_burn,
                Collection::validate_burn,
                Plugin::validate_burn,
            )?;
        }
        _ => return Err(MplCoreError::IncorrectAccount.into()),
    }

    process_burn(ctx.accounts.asset, ctx.accounts.authority)
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct BurnCollectionArgs {
    compression_proof: Option<CompressionProof>,
}

pub(crate) fn burn_collection<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: BurnCollectionArgs,
) -> ProgramResult {
    // Accounts.
    let ctx = BurnCollectionAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
    }

    let _ = validate_collection_permissions(
        ctx.accounts.authority,
        ctx.accounts.collection,
        Collection::check_burn,
        PluginType::check_burn,
        Collection::validate_burn,
        Plugin::validate_burn,
    )?;

    process_burn(ctx.accounts.collection, ctx.accounts.authority)
}

fn process_burn<'a>(core_info: &AccountInfo<'a>, authority: &AccountInfo<'a>) -> ProgramResult {
    close_program_account(core_info, authority)
}
