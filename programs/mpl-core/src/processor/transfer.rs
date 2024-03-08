use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::TransferAccounts,
    plugins::{Plugin, PluginType},
    state::{Asset, Collection, Compressible, CompressionProof, HashedAsset, Key, SolanaAccount},
    utils::{load_key, validate_asset_permissions, verify_proof},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct TransferArgs {
    compression_proof: Option<CompressionProof>,
}

pub(crate) fn transfer<'a>(accounts: &'a [AccountInfo<'a>], args: TransferArgs) -> ProgramResult {
    // Accounts.
    let ctx = TransferAccounts::context(accounts)?;

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
            let (mut asset, _) = verify_proof(ctx.accounts.asset, &compression_proof)?;

            if ctx.accounts.authority.key != &asset.owner {
                return Err(MplCoreError::InvalidAuthority.into());
            }

            // TODO: Check delegates in compressed case.

            asset.owner = *ctx.accounts.new_owner.key;

            asset.wrap()?;

            // Make a new hashed asset with updated owner and save to account.
            HashedAsset::new(asset.hash()?).save(ctx.accounts.asset, 0)
        }
        Key::Asset => {
            let (mut asset, _, _) = validate_asset_permissions(
                ctx.accounts.authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                Some(ctx.accounts.new_owner),
                Asset::check_transfer,
                Collection::check_transfer,
                PluginType::check_transfer,
                Asset::validate_transfer,
                Collection::validate_transfer,
                Plugin::validate_transfer,
            )?;

            asset.owner = *ctx.accounts.new_owner.key;
            asset.save(ctx.accounts.asset, 0)
        }
        _ => Err(MplCoreError::IncorrectAccount.into()),
    }
}
