use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{BurnCollectionV1Accounts, BurnV1Accounts},
    plugins::{ExternalPluginAdapter, HookableLifecycleEvent, Plugin, PluginType},
    state::{AssetV1, CollectionV1, CompressionProof, Key, SolanaAccount, Wrappable},
    utils::{
        close_program_account, load_key, rebuild_account_state_from_proof_data, resolve_authority,
        validate_asset_permissions, verify_proof,
    },
};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct BurnV1Args {
    compression_proof: Option<CompressionProof>,
}

pub(crate) fn burn<'a>(accounts: &'a [AccountInfo<'a>], args: BurnV1Args) -> ProgramResult {
    // Accounts.
    let ctx = BurnV1Accounts::context(accounts)?;
    let collection = if let Some(collection) = ctx.accounts.collection {
        Some(CollectionV1::load(collection, 0)?)
    } else {
        None
    };

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

    match load_key(ctx.accounts.asset, 0)? {
        Key::HashedAssetV1 => {
            let mut compression_proof = args
                .compression_proof
                .ok_or(MplCoreError::MissingCompressionProof)?;

            let system_program = ctx
                .accounts
                .system_program
                .ok_or(MplCoreError::MissingSystemProgram)?;

            // Verify the proof and rebuild Asset struct in account space.
            let (asset, plugins) = verify_proof(ctx.accounts.asset, &compression_proof)?;

            // Use the data from the compression proof to rebuild the account.  Only needed for validation.
            rebuild_account_state_from_proof_data(
                asset,
                plugins,
                ctx.accounts.asset,
                ctx.accounts.payer,
                system_program,
            )?;

            // Increment sequence number for the spl-noop event.  Note we don't care about the
            // sequence number in account state because we are closing the account later in this
            // instruction.
            compression_proof.seq = compression_proof.seq.saturating_add(1);

            // Send the spl-noop event for indexing the compressed asset.
            compression_proof.wrap()?;

            // TODO Enable compressed burn.
            msg!("Error: Burning compressed is currently not available");
            return Err(MplCoreError::NotAvailable.into());
        }
        Key::AssetV1 => (),
        _ => return Err(MplCoreError::IncorrectAccount.into()),
    }

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
        AssetV1::check_burn,
        CollectionV1::check_burn,
        PluginType::check_burn,
        AssetV1::validate_burn,
        CollectionV1::validate_burn,
        Plugin::validate_burn,
        Some(ExternalPluginAdapter::validate_burn),
        Some(HookableLifecycleEvent::Burn),
    )?;

    process_burn(ctx.accounts.asset, ctx.accounts.payer)?;
    if let Some(mut collection) = collection {
        collection.decrement_size()?;
        collection.save(ctx.accounts.collection.unwrap(), 0)?;
    };
    Ok(())
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct BurnCollectionV1Args {
    compression_proof: Option<CompressionProof>,
}

pub(crate) fn burn_collection<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: BurnCollectionV1Args,
) -> ProgramResult {
    // Accounts.
    let ctx = BurnCollectionV1Accounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    if let Some(log_wrapper) = ctx.accounts.log_wrapper {
        if log_wrapper.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    let collection = CollectionV1::load(ctx.accounts.collection, 0)?;
    if collection.current_size > 0 {
        return Err(MplCoreError::CollectionMustBeEmpty.into());
    }

    // If the update authority is the one burning the collection, and the collection is empty, then it can be burned.
    if authority.key != &collection.update_authority {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    process_burn(ctx.accounts.collection, ctx.accounts.payer)
}

fn process_burn<'a>(core_info: &AccountInfo<'a>, authority: &AccountInfo<'a>) -> ProgramResult {
    close_program_account(core_info, authority)
}
