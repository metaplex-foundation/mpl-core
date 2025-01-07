use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::TransferV1Accounts,
    plugins::{ExternalPluginAdapter, HookableLifecycleEvent, Plugin, PluginType},
    state::{AssetV1, Authority, CollectionV1, CompressionProof, Key, SolanaAccount, Wrappable},
    utils::{
        compress_into_account_space, load_key, rebuild_account_state_from_proof_data,
        resolve_authority, validate_asset_permissions, verify_proof,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct TransferV1Args {
    compression_proof: Option<CompressionProof>,
}

pub(crate) fn transfer<'a>(accounts: &'a [AccountInfo<'a>], args: TransferV1Args) -> ProgramResult {
    // Accounts.
    let ctx = TransferV1Accounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    if let Some(system_program) = ctx.accounts.system_program {
        if system_program.key != &solana_program::system_program::ID {
            return Err(MplCoreError::InvalidSystemProgram.into());
        }
    }

    if let Some(log_wrapper) = ctx.accounts.log_wrapper {
        if log_wrapper.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    let key = load_key(ctx.accounts.asset, 0)?;

    match key {
        Key::HashedAssetV1 => {
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
                ctx.accounts.payer,
                system_program,
            )?;

            // TODO Enable compressed transfer.
            msg!("Error: Transferring compressed is currently not available");
            return Err(MplCoreError::NotAvailable.into());
        }
        Key::AssetV1 => (),
        _ => return Err(MplCoreError::IncorrectAccount.into()),
    }

    // Validate asset permissions.
    let (mut asset, plugin_header, plugin_registry) = validate_asset_permissions(
        accounts,
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        Some(ctx.accounts.new_owner),
        None,
        None,
        None,
        None,
        None,
        AssetV1::check_transfer,
        CollectionV1::check_transfer,
        PluginType::check_transfer,
        AssetV1::validate_transfer,
        CollectionV1::validate_transfer,
        Plugin::validate_transfer,
        Some(ExternalPluginAdapter::validate_transfer),
        Some(HookableLifecycleEvent::Transfer),
    )?;

    // Reset every owner-managed plugin in the registry.
    if let (Some(plugin_header), Some(mut plugin_registry)) =
        (plugin_header, plugin_registry.clone())
    {
        plugin_registry.registry.iter_mut().for_each(|record| {
            if record.plugin_type.manager() == Authority::Owner {
                record.authority = Authority::Owner;
            }
        });

        // Save the plugin registry.
        plugin_registry.save(ctx.accounts.asset, plugin_header.plugin_registry_offset)?;
    }

    // Set the new owner.
    asset.owner = *ctx.accounts.new_owner.key;

    // Reserialize the account into correct format.
    match key {
        Key::HashedAssetV1 => {
            let system_program = ctx
                .accounts
                .system_program
                .ok_or(MplCoreError::MissingSystemProgram)?;

            // Compress the asset and plugin registry into account space.
            let compression_proof = compress_into_account_space(
                asset,
                plugin_registry,
                ctx.accounts.asset,
                ctx.accounts.payer,
                system_program,
            )?;

            // Send the spl-noop event for indexing the compressed asset.
            compression_proof.wrap()
        }
        Key::AssetV1 => {
            // Increment sequence number only if it is `Some(_)`.
            asset.seq = asset.seq.map(|seq| seq.saturating_add(1));
            asset.save(ctx.accounts.asset, 0)
        }
        _ => unreachable!(),
    }
}
