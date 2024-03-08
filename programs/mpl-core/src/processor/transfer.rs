use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::TransferAccounts,
    plugins::{Plugin, PluginType},
    state::{Asset, Collection, CompressionProof, Key, SolanaAccount, Wrappable},
    utils::{
        compress_into_account_space, load_key, rebuild_account_state_from_proof_data,
        validate_asset_permissions, verify_proof,
    },
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
    let payer = if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
        payer
    } else {
        ctx.accounts.authority
    };

    match load_key(ctx.accounts.asset, 0)? {
        Key::HashedAsset => {
            let compression_proof = args
                .compression_proof
                .ok_or(MplCoreError::MissingCompressionProof)?;

            let system_program = ctx
                .accounts
                .system_program
                .ok_or(MplCoreError::MissingSystemProgram)?;

            // Verify the proof and rebuild Asset struct in account space.
            let (mut asset, plugins) = verify_proof(ctx.accounts.asset, &compression_proof)?;

            // Set the new owner.
            asset.owner = *ctx.accounts.new_owner.key;

            // Use the data from the compression proof to rebuild the account.
            rebuild_account_state_from_proof_data(
                asset,
                plugins,
                ctx.accounts.asset,
                payer,
                system_program,
            )?;

            let (asset, _, plugin_registry) = validate_asset_permissions(
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

            let compression_proof = compress_into_account_space(
                asset,
                plugin_registry,
                ctx.accounts.asset,
                payer,
                system_program,
            )?;

            compression_proof.wrap()
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

            // Set the new owner.
            asset.owner = *ctx.accounts.new_owner.key;
            asset.save(ctx.accounts.asset, 0)
        }
        _ => Err(MplCoreError::IncorrectAccount.into()),
    }
}
