use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        RevokeCollectionPluginAuthorityAccounts, RevokePluginAuthorityAccounts,
    },
    plugins::{
        fetch_wrapped_plugin, revoke_authority_on_plugin, Plugin, PluginHeader, PluginRegistry,
        PluginType,
    },
    state::{Asset, Authority, Collection, Key},
    utils::{
        fetch_core_data, load_key, resolve_payer, resolve_to_authority, validate_asset_permissions,
        validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RevokePluginAuthorityArgs {
    pub plugin_type: PluginType,
}

pub(crate) fn revoke_plugin_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RevokePluginAuthorityArgs,
) -> ProgramResult {
    let ctx = RevokePluginAuthorityAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = resolve_payer(ctx.accounts.authority, ctx.accounts.payer)?;

    if let Key::HashedAsset = load_key(ctx.accounts.asset, 0)? {
        msg!("Error: Revoke plugin authority for compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    let (mut asset, plugin_header, mut plugin_registry) =
        fetch_core_data::<Asset>(ctx.accounts.asset)?;

    //TODO: Make this better.
    let authority_type =
        resolve_to_authority(ctx.accounts.authority, ctx.accounts.collection, &asset)?;

    let (_, plugin) = fetch_wrapped_plugin::<Asset>(ctx.accounts.asset, args.plugin_type)?;

    // Validate asset permissions.
    let _ = validate_asset_permissions(
        ctx.accounts.authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        Some(&plugin),
        Asset::check_revoke_plugin_authority,
        Collection::check_revoke_plugin_authority,
        PluginType::check_revoke_plugin_authority,
        Asset::validate_revoke_plugin_authority,
        Collection::validate_revoke_plugin_authority,
        Plugin::validate_revoke_plugin_authority,
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    process_revoke_plugin_authority(
        ctx.accounts.asset,
        payer,
        ctx.accounts.system_program,
        &authority_type,
        &args.plugin_type,
        plugin_header.as_ref(),
        plugin_registry.as_mut(),
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RevokeCollectionPluginAuthorityArgs {
    pub plugin_type: PluginType,
}

pub(crate) fn revoke_collection_plugin_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RevokeCollectionPluginAuthorityArgs,
) -> ProgramResult {
    let ctx = RevokeCollectionPluginAuthorityAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = resolve_payer(ctx.accounts.authority, ctx.accounts.payer)?;

    let (_, plugin_header, mut plugin_registry) =
        fetch_core_data::<Collection>(ctx.accounts.collection)?;

    let (_, plugin) =
        fetch_wrapped_plugin::<Collection>(ctx.accounts.collection, args.plugin_type)?;

    // Validate collection permissions.
    let _ = validate_collection_permissions(
        ctx.accounts.authority,
        ctx.accounts.collection,
        Some(&plugin),
        Collection::check_revoke_plugin_authority,
        PluginType::check_revoke_plugin_authority,
        Collection::validate_revoke_plugin_authority,
        Plugin::validate_revoke_plugin_authority,
    )?;

    process_revoke_plugin_authority(
        ctx.accounts.collection,
        payer,
        ctx.accounts.system_program,
        &Authority::UpdateAuthority,
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
    authority_type: &Authority,
    plugin_type: &PluginType,
    plugin_header: Option<&PluginHeader>,
    plugin_registry: Option<&mut PluginRegistry>,
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
        authority_type,
        core_info,
        plugin_header,
        plugin_registry,
        payer,
        system_program,
    )
}
