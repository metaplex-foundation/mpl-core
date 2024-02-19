use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplAssetError,
    instruction::accounts::TransferAccounts,
    plugins::{fetch_plugin, Plugin, PluginType},
    state::{Asset, Compressible, CompressionProof, DataBlob, HashedAsset, Key, SolanaAccount},
    utils::{assert_authority, load_key},
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

    match load_key(ctx.accounts.asset_address, 0)? {
        Key::HashedAsset => {
            let mut asset =
                Asset::verify_proof(ctx.accounts.asset_address, args.compression_proof)?;

            asset.owner = *ctx.accounts.new_owner.key;

            asset.wrap()?;

            // Make a new hashed asset with updated owner and save to account.
            HashedAsset::new(asset.hash()?).save(ctx.accounts.asset_address, 0)
        }
        Key::Asset => {
            let mut asset = Asset::load(ctx.accounts.asset_address, 0)?;

            let mut authority_check: Result<(), ProgramError> =
                Err(MplAssetError::InvalidAuthority.into());
            if asset.get_size() != ctx.accounts.asset_address.data_len() {
                solana_program::msg!("Fetch Plugin");
                let (authorities, plugin, _) =
                    fetch_plugin(ctx.accounts.asset_address, PluginType::Delegate)?;

                solana_program::msg!("Assert authority");
                authority_check = assert_authority(
                    ctx.accounts.asset_address,
                    ctx.accounts.authority,
                    &authorities,
                );

                if let Plugin::Delegate(delegate) = plugin {
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

            asset.owner = *ctx.accounts.new_owner.key;
            asset.save(ctx.accounts.asset_address, 0)
        }
        _ => Err(MplAssetError::IncorrectAccount.into()),
    }
}
