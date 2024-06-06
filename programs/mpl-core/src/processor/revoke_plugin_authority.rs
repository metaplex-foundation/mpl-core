use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        RevokeCollectionPluginAuthorityV1Accounts, RevokePluginAuthorityV1Accounts,
    },
    plugins::{
        fetch_wrapped_plugin, revoke_authority_on_plugin, Plugin, PluginHeaderV1, PluginRegistryV1,
        PluginType,
    },
    state::{AssetV1, CollectionV1, Key},
    utils::{
        fetch_core_data, load_key, resolve_authority, resolve_pubkey_to_authorities,
        resolve_pubkey_to_authorities_collection, validate_asset_permissions,
        validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RevokePluginAuthorityV1Args {
    pub plugin_type: PluginType,
}

pub(crate) fn revoke_plugin_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RevokePluginAuthorityV1Args,
) -> ProgramResult {
    let ctx = RevokePluginAuthorityV1Accounts::context(accounts)?;

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
        msg!("Error: Revoke plugin authority for compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    let (mut asset, plugin_header, mut plugin_registry) =
        fetch_core_data::<AssetV1>(ctx.accounts.asset)?;

    let (_, plugin) =
        fetch_wrapped_plugin::<AssetV1>(ctx.accounts.asset, Some(&asset), args.plugin_type)?;

    // Validate asset permissions.
    let _ = validate_asset_permissions(
        accounts,
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        None,
        Some(&plugin),
        None,
        AssetV1::check_revoke_plugin_authority,
        CollectionV1::check_revoke_plugin_authority,
        PluginType::check_revoke_plugin_authority,
        AssetV1::validate_revoke_plugin_authority,
        CollectionV1::validate_revoke_plugin_authority,
        Plugin::validate_revoke_plugin_authority,
        None,
        None,
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    let resolved_authorities =
        resolve_pubkey_to_authorities(authority, ctx.accounts.collection, &asset)?;
    let payer = if resolved_authorities.contains(&plugin.manager()) {
        ctx.accounts.payer
    } else {
        ctx.accounts.asset
    };

    process_revoke_plugin_authority(
        ctx.accounts.asset,
        payer,
        ctx.accounts.system_program,
        &args.plugin_type,
        plugin_header.as_ref(),
        plugin_registry.as_mut(),
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RevokeCollectionPluginAuthorityV1Args {
    pub plugin_type: PluginType,
}

pub(crate) fn revoke_collection_plugin_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RevokeCollectionPluginAuthorityV1Args,
) -> ProgramResult {
    let ctx = RevokeCollectionPluginAuthorityV1Accounts::context(accounts)?;

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

    let (collection, plugin_header, mut plugin_registry) =
        fetch_core_data::<CollectionV1>(ctx.accounts.collection)?;

    let (_, plugin) = fetch_wrapped_plugin::<CollectionV1>(
        ctx.accounts.collection,
        Some(&collection),
        args.plugin_type,
    )?;

    // Validate collection permissions.
    let _ = validate_collection_permissions(
        accounts,
        authority,
        ctx.accounts.collection,
        None,
        Some(&plugin),
        None,
        CollectionV1::check_revoke_plugin_authority,
        PluginType::check_revoke_plugin_authority,
        CollectionV1::validate_revoke_plugin_authority,
        Plugin::validate_revoke_plugin_authority,
        None,
        None,
    )?;

    let resolved_authorities =
        resolve_pubkey_to_authorities_collection(authority, ctx.accounts.collection)?;
    let payer = if resolved_authorities.contains(&plugin.manager()) {
        ctx.accounts.payer
    } else {
        ctx.accounts.collection
    };

    process_revoke_plugin_authority(
        ctx.accounts.collection,
        payer,
        ctx.accounts.system_program,
        &args.plugin_type,
        plugin_header.as_ref(),
        plugin_registry.as_mut(),
    )
}

#[allow(clippy::too_many_arguments)]
fn process_revoke_plugin_authority<'a>(
    core_info: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    plugin_type: &PluginType,
    plugin_header: Option<&PluginHeaderV1>,
    plugin_registry: Option<&mut PluginRegistryV1>,
) -> ProgramResult {
    let plugin_header = match plugin_header {
        Some(header) => header,
        None => return Err(MplCoreError::PluginsNotInitialized.into()),
    };

    let plugin_registry = match plugin_registry {
        Some(registry) => registry,
        None => return Err(MplCoreError::PluginsNotInitialized.into()),
    };

    revoke_authority_on_plugin(
        plugin_type,
        core_info,
        plugin_header,
        plugin_registry,
        payer,
        system_program,
    )
}
