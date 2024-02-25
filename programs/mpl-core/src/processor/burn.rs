use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::{
    error::MplAssetError,
    instruction::accounts::BurnAccounts,
    plugins::{fetch_plugin, Plugin, PluginType},
    state::{Asset, Compressible, CompressionProof, DataBlob, Key, SolanaAccount},
    utils::{assert_authority, close_program_account, load_key},
};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct BurnArgs {
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

    match load_key(ctx.accounts.asset_address, 0)? {
        Key::HashedAsset => {
            let compression_proof = args
                .compression_proof
                .ok_or(MplAssetError::MissingCompressionProof)?;
            let asset = Asset::verify_proof(ctx.accounts.asset_address, compression_proof)?;

            if ctx.accounts.authority.key != &asset.owner {
                return Err(MplAssetError::InvalidAuthority.into());
            }

            // TODO: Check delegates in compressed case.

            asset.wrap()?;
        }
        Key::Asset => {
            let asset = Asset::load(ctx.accounts.asset_address, 0)?;

            let mut authority_check: Result<(), ProgramError> =
                Err(MplAssetError::InvalidAuthority.into());
            if asset.get_size() != ctx.accounts.asset_address.data_len() {
                solana_program::msg!("Fetch Plugin");
                let (authorities, plugin, _) =
                    fetch_plugin(ctx.accounts.asset_address, PluginType::Freeze)?;

                solana_program::msg!("Assert authority");
                authority_check = assert_authority(&asset, ctx.accounts.authority, &authorities);

                if let Plugin::Freeze(delegate) = plugin {
                    if delegate.frozen {
                        return Err(MplAssetError::AssetIsFrozen.into());
                    }
                }
            }

            match authority_check {
                Ok(_) => Ok::<(), ProgramError>(()),
                Err(_) => {
                    if ctx.accounts.authority.key != &asset.owner {
                        Err(MplAssetError::InvalidAuthority.into())
                    } else {
                        Ok(())
                    }
                }
            }?;
        }
        _ => return Err(MplAssetError::IncorrectAccount.into()),
    }

    close_program_account(ctx.accounts.asset_address, ctx.accounts.authority)
}
