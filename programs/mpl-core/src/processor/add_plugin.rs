use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{AddCollectionPluginAccounts, AddPluginAccounts},
    plugins::{create_meta_idempotent, initialize_plugin, Plugin, PluginType, ValidationResult},
    state::{Asset, Authority, Collection, DataBlob, Key, SolanaAccount},
    utils::{
        load_key, resolve_authority, validate_asset_permissions, validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddPluginArgs {
    plugin: Plugin,
    init_authority: Option<Authority>,
}

pub(crate) fn add_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddPluginArgs,
) -> ProgramResult {
    let ctx = AddPluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    if let Key::HashedAsset = load_key(ctx.accounts.asset, 0)? {
        msg!("Error: Adding plugin to compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    if Plugin::validate_add_plugin(
        &args.plugin,
        authority,
        None,
        &args.init_authority.unwrap_or(args.plugin.manager()),
        None,
    )? == ValidationResult::Rejected
    {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Validate asset permissions.
    let (mut asset, _, _) = validate_asset_permissions(
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        Some(&args.plugin),
        Asset::check_add_plugin,
        Collection::check_add_plugin,
        PluginType::check_add_plugin,
        Asset::validate_add_plugin,
        Collection::validate_add_plugin,
        Plugin::validate_add_plugin,
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    process_add_plugin::<Asset>(
        ctx.accounts.asset,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        &args.plugin,
        &args.init_authority.unwrap_or(args.plugin.manager()),
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddCollectionPluginArgs {
    plugin: Plugin,
    init_authority: Option<Authority>,
}

pub(crate) fn add_collection_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddCollectionPluginArgs,
) -> ProgramResult {
    let ctx = AddCollectionPluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    // Validate collection permissions.
    let _ = validate_collection_permissions(
        authority,
        ctx.accounts.collection,
        Some(&args.plugin),
        Collection::check_add_plugin,
        PluginType::check_add_plugin,
        Collection::validate_add_plugin,
        Plugin::validate_add_plugin,
    )?;

    process_add_plugin::<Collection>(
        ctx.accounts.collection,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        &args.plugin,
        &args.init_authority.unwrap_or(args.plugin.manager()),
    )
}

fn process_add_plugin<'a, T: DataBlob + SolanaAccount>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    plugin: &Plugin,
    authority: &Authority,
) -> ProgramResult {
    solana_program::msg!("Creating meta if it doesn't exist");
    create_meta_idempotent::<T>(account, payer, system_program)?;
    solana_program::msg!("Initializing plugin");
    initialize_plugin::<T>(plugin, authority, account, payer, system_program)?;
    solana_program::msg!("Plugin added successfully");
    Ok(())
}
