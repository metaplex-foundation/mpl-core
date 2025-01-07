use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        ApproveCollectionPluginAuthorityV1Accounts, ApprovePluginAuthorityV1Accounts,
    },
    plugins::{approve_authority_on_plugin, fetch_wrapped_plugin, Plugin, PluginType},
    state::{AssetV1, Authority, CollectionV1, CoreAsset, DataBlob, Key, SolanaAccount},
    utils::{
        fetch_core_data, load_key, resolve_authority, validate_asset_permissions,
        validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct ApprovePluginAuthorityV1Args {
    pub plugin_type: PluginType,
    pub new_authority: Authority,
}

pub(crate) fn approve_plugin_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: ApprovePluginAuthorityV1Args,
) -> ProgramResult {
    let ctx = ApprovePluginAuthorityV1Accounts::context(accounts)?;

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

    if let Key::HashedAssetV1 = load_key(ctx.accounts.asset, 0)? {
        msg!("Error: Approve plugin authority for compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    let (plugin_authority, plugin) =
        fetch_wrapped_plugin::<AssetV1>(ctx.accounts.asset, None, args.plugin_type)?;

    // Validate asset permissions.
    let (mut asset, _, _) = validate_asset_permissions(
        accounts,
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        None,
        Some(&plugin),
        Some(&plugin_authority),
        None,
        None,
        AssetV1::check_approve_plugin_authority,
        CollectionV1::check_approve_plugin_authority,
        PluginType::check_approve_plugin_authority,
        AssetV1::validate_approve_plugin_authority,
        CollectionV1::validate_approve_plugin_authority,
        Plugin::validate_approve_plugin_authority,
        None,
        None,
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    process_approve_plugin_authority::<AssetV1>(
        ctx.accounts.asset,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        &args.plugin_type,
        &args.new_authority,
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct ApproveCollectionPluginAuthorityV1Args {
    pub plugin_type: PluginType,
    pub new_authority: Authority,
}

pub(crate) fn approve_collection_plugin_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: ApproveCollectionPluginAuthorityV1Args,
) -> ProgramResult {
    let ctx = ApproveCollectionPluginAuthorityV1Accounts::context(accounts)?;

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

    let (plugin_authority, plugin) =
        fetch_wrapped_plugin::<CollectionV1>(ctx.accounts.collection, None, args.plugin_type)?;

    // Validate collection permissions.
    let _ = validate_collection_permissions(
        accounts,
        authority,
        ctx.accounts.collection,
        None,
        Some(&plugin),
        Some(&plugin_authority),
        None,
        None,
        CollectionV1::check_approve_plugin_authority,
        PluginType::check_approve_plugin_authority,
        CollectionV1::validate_approve_plugin_authority,
        Plugin::validate_approve_plugin_authority,
        None,
        None,
    )?;

    process_approve_plugin_authority::<CollectionV1>(
        ctx.accounts.collection,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        &args.plugin_type,
        &args.new_authority,
    )
}

fn process_approve_plugin_authority<'a, T: CoreAsset + DataBlob + SolanaAccount>(
    core_info: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    plugin_type: &PluginType,
    new_authority: &Authority,
) -> ProgramResult {
    let (_, plugin_header, plugin_registry) = fetch_core_data::<T>(core_info)?;

    let plugin_header = match plugin_header {
        Some(header) => header,
        None => return Err(MplCoreError::PluginsNotInitialized.into()),
    };

    let mut plugin_registry = match plugin_registry {
        Some(registry) => registry,
        None => return Err(MplCoreError::PluginsNotInitialized.into()),
    };

    approve_authority_on_plugin::<T>(
        plugin_type,
        new_authority,
        core_info,
        &plugin_header,
        &mut plugin_registry,
        payer,
        system_program,
    )
}
