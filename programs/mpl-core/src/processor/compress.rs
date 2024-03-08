use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::CompressAccounts,
    plugins::{Plugin, PluginType},
    state::{Asset, Collection, Key, Wrappable},
    utils::{compress_into_account_space, fetch_core_data, load_key, validate_asset_permissions},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CompressArgs {}

pub(crate) fn compress<'a>(accounts: &'a [AccountInfo<'a>], _args: CompressArgs) -> ProgramResult {
    // Accounts.
    let ctx = CompressAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
        payer
    } else {
        ctx.accounts.authority
    };

    match load_key(ctx.accounts.asset, 0)? {
        Key::Asset => {
            let (asset, _, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

            // Validate asset permissions.
            let _ = validate_asset_permissions(
                ctx.accounts.authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                None,
                Asset::check_compress,
                Collection::check_compress,
                PluginType::check_compress,
                Asset::validate_compress,
                Collection::validate_compress,
                Plugin::validate_compress,
            )?;

            // Compress the asset and plugin registry into account space.
            let compression_proof = compress_into_account_space(
                asset,
                plugin_registry,
                ctx.accounts.asset,
                payer,
                ctx.accounts.system_program,
            )?;

            // Send the spl-noop event for indexing the compressed asset.
            compression_proof.wrap()?;

            // TODO Enable compression.
            msg!("Error: Compression currently not available");
            Err(MplCoreError::NotAvailable.into())
        }
        Key::HashedAsset => Err(MplCoreError::AlreadyCompressed.into()),
        _ => Err(MplCoreError::IncorrectAccount.into()),
    }
}
