use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::TransferAccounts,
    plugins::{Plugin, PluginType},
    state::{Asset, Collection, CompressionProof, Key, SolanaAccount, Wrappable},
    utils::{
        compress_into_account_space, load_key, rebuild_account_state_from_proof_data,
        resolve_payer, validate_asset_permissions, verify_proof,
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
    let payer = resolve_payer(ctx.accounts.authority, ctx.accounts.payer)?;

    let key = load_key(ctx.accounts.asset, 0)?;

    match key {
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

            // TODO Enable compressed transfer.
            msg!("Error: Transferring compressed is currently not available");
            return Err(MplCoreError::NotAvailable.into());
        }
        Key::Asset => (),
        _ => return Err(MplCoreError::IncorrectAccount.into()),
    }

    // Validate asset permissions.
    let (mut asset, _, plugin_registry) = validate_asset_permissions(
        ctx.accounts.authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        Some(ctx.accounts.new_owner),
        None,
        Asset::check_transfer,
        Collection::check_transfer,
        PluginType::check_transfer,
        Asset::validate_transfer,
        Collection::validate_transfer,
        Plugin::validate_transfer,
    )?;

    // Set the new owner.
    asset.owner = *ctx.accounts.new_owner.key;

    // Reserialize the account into correct format.
    match key {
        Key::HashedAsset => {
            let system_program = ctx
                .accounts
                .system_program
                .ok_or(MplCoreError::MissingSystemProgram)?;

            // Compress the asset and plugin registry into account space.
            let compression_proof = compress_into_account_space(
                asset,
                plugin_registry,
                ctx.accounts.asset,
                payer,
                system_program,
            )?;

            // Send the spl-noop event for indexing the compressed asset.
            compression_proof.wrap()
        }
        Key::Asset => {
            // Increment sequence number only if it is `Some(_)`.
            asset.seq = asset.seq.map(|seq| seq.saturating_add(1));
            asset.save(ctx.accounts.asset, 0)
        }
        _ => unreachable!(),
    }
}
