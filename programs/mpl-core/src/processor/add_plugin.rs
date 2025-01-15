use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{AddCollectionPluginV1Accounts, AddPluginV1Accounts},
    plugins::{
        create_meta_idempotent, initialize_plugin, Plugin, PluginType, PluginValidationContext,
        ValidationResult,
    },
    state::{AssetV1, Authority, CollectionV1, DataBlob, Key, SolanaAccount},
    utils::{
        load_key, resolve_authority, validate_asset_permissions, validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddPluginV1Args {
    plugin: Plugin,
    init_authority: Option<Authority>,
}

pub(crate) fn add_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddPluginV1Args,
) -> ProgramResult {
    let ctx = AddPluginV1Accounts::context(accounts)?;

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
        msg!("Error: Adding plugin to compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    // TODO move into plugin validation when asset/collection is part of validation context
    let plugin_type = PluginType::from(&args.plugin);
    if plugin_type == PluginType::MasterEdition {
        return Err(MplCoreError::InvalidPlugin.into());
    }

    let target_plugin_authority = args.init_authority.unwrap_or(args.plugin.manager());

    // TODO: Seed with Rejected
    // TODO: refactor to allow add_plugin to approve additions
    let validation_ctx = PluginValidationContext {
        accounts,
        asset_info: Some(ctx.accounts.asset),
        collection_info: ctx.accounts.collection,
        self_authority: &target_plugin_authority,
        authority_info: authority,
        resolved_authorities: None,
        new_owner: None,
        new_asset_authority: None,
        new_collection_authority: None,
        target_plugin: Some(&args.plugin),
        target_plugin_authority: Some(&target_plugin_authority),
        target_external_plugin: None,
        target_external_plugin_authority: None,
    };
    if Plugin::validate_add_plugin(&args.plugin, &validation_ctx)? == ValidationResult::Rejected {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Validate asset permissions.
    let (mut asset, _, _) = validate_asset_permissions(
        accounts,
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        None,
        Some(&args.plugin),
        Some(&target_plugin_authority),
        None,
        None,
        AssetV1::check_add_plugin,
        CollectionV1::check_add_plugin,
        PluginType::check_add_plugin,
        AssetV1::validate_add_plugin,
        CollectionV1::validate_add_plugin,
        Plugin::validate_add_plugin,
        None,
        None,
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    process_add_plugin::<AssetV1>(
        ctx.accounts.asset,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        &args.plugin,
        &args.init_authority.unwrap_or(args.plugin.manager()),
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddCollectionPluginV1Args {
    plugin: Plugin,
    init_authority: Option<Authority>,
}

pub(crate) fn add_collection_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddCollectionPluginV1Args,
) -> ProgramResult {
    let ctx = AddCollectionPluginV1Accounts::context(accounts)?;

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

    let target_plugin_authority = args.init_authority.unwrap_or(args.plugin.manager());
    let validation_ctx = PluginValidationContext {
        accounts,
        asset_info: None,
        collection_info: Some(ctx.accounts.collection),
        self_authority: &target_plugin_authority,
        authority_info: authority,
        resolved_authorities: None,
        new_owner: None,
        new_asset_authority: None,
        new_collection_authority: None,
        target_plugin: Some(&args.plugin),
        target_plugin_authority: Some(&target_plugin_authority),
        target_external_plugin: None,
        target_external_plugin_authority: None,
    };
    if Plugin::validate_add_plugin(&args.plugin, &validation_ctx)? == ValidationResult::Rejected {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Cannot add owner-managed plugins to collection.
    if args.plugin.manager() == Authority::Owner {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Validate collection permissions.
    let _ = validate_collection_permissions(
        accounts,
        authority,
        ctx.accounts.collection,
        None,
        Some(&args.plugin),
        Some(&target_plugin_authority),
        None,
        None,
        CollectionV1::check_add_plugin,
        PluginType::check_add_plugin,
        CollectionV1::validate_add_plugin,
        Plugin::validate_add_plugin,
        None,
        None,
    )?;

    process_add_plugin::<CollectionV1>(
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
    let (_, header_offset, mut plugin_header, mut plugin_registry) =
        create_meta_idempotent::<T>(account, payer, system_program)?;
    initialize_plugin::<T>(
        plugin,
        authority,
        header_offset,
        &mut plugin_header,
        &mut plugin_registry,
        account,
        payer,
        system_program,
    )?;
    Ok(())
}
