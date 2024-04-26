use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{AddCollectionExternalPluginV1Accounts, AddExternalPluginV1Accounts},
    plugins::{
        create_meta_idempotent, initialize_external_plugin, ExternalPlugin, ExternalPluginInitInfo,
        Plugin, PluginType, PluginValidationContext, ValidationResult,
    },
    state::{AssetV1, Authority, CollectionV1, DataBlob, Key, SolanaAccount},
    utils::{
        load_key, resolve_authority, validate_asset_permissions, validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddExternalPluginV1Args {
    /// External plugin initialization info.
    pub init_info: ExternalPluginInitInfo,
}

pub(crate) fn add_external_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddExternalPluginV1Args,
) -> ProgramResult {
    let ctx = AddExternalPluginV1Accounts::context(accounts)?;

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

    let validation_ctx = PluginValidationContext {
        accounts,
        asset_info: Some(ctx.accounts.asset),
        collection_info: ctx.accounts.collection,
        self_authority: &Authority::UpdateAuthority,
        authority_info: authority,
        resolved_authorities: None,
        new_owner: None,
        target_plugin: None,
    };

    if ExternalPlugin::validate_add_external_plugin(
        &ExternalPlugin::from(&args.init_info),
        &validation_ctx,
    )? == ValidationResult::Rejected
    {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    let external_plugin = ExternalPlugin::from(&args.init_info);

    // Validate asset permissions.
    let (mut asset, _, _) = validate_asset_permissions(
        accounts,
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        None,
        Some(&external_plugin),
        AssetV1::check_add_external_plugin,
        CollectionV1::check_add_external_plugin,
        PluginType::check_add_external_plugin,
        AssetV1::validate_add_external_plugin,
        CollectionV1::validate_add_external_plugin,
        Plugin::validate_add_external_plugin,
        None,
        None,
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    process_add_external_plugin::<AssetV1>(
        ctx.accounts.asset,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        &args.init_info,
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddCollectionExternalPluginV1Args {
    /// External plugin initialization info.
    pub init_info: ExternalPluginInitInfo,
}

pub(crate) fn add_collection_external_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddCollectionExternalPluginV1Args,
) -> ProgramResult {
    let ctx = AddCollectionExternalPluginV1Accounts::context(accounts)?;

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

    let validation_ctx = PluginValidationContext {
        accounts,
        asset_info: None,
        collection_info: Some(ctx.accounts.collection),
        self_authority: &Authority::UpdateAuthority,
        authority_info: authority,
        resolved_authorities: None,
        new_owner: None,
        target_plugin: None,
    };

    if ExternalPlugin::validate_add_external_plugin(
        &ExternalPlugin::from(&args.init_info),
        &validation_ctx,
    )? == ValidationResult::Rejected
    {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    let external_plugin = ExternalPlugin::from(&args.init_info);

    // Validate collection permissions.
    let _ = validate_collection_permissions(
        accounts,
        authority,
        ctx.accounts.collection,
        None,
        Some(&external_plugin),
        CollectionV1::check_add_external_plugin,
        PluginType::check_add_external_plugin,
        CollectionV1::validate_add_external_plugin,
        Plugin::validate_add_external_plugin,
        None,
        None,
    )?;

    process_add_external_plugin::<CollectionV1>(
        ctx.accounts.collection,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        &args.init_info,
    )
}

fn process_add_external_plugin<'a, T: DataBlob + SolanaAccount>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    init_info: &ExternalPluginInitInfo,
) -> ProgramResult {
    let (_, mut plugin_header, mut plugin_registry) =
        create_meta_idempotent::<T>(account, payer, system_program)?;
    initialize_external_plugin::<T>(
        init_info,
        &mut plugin_header,
        &mut plugin_registry,
        account,
        payer,
        system_program,
    )?;
    Ok(())
}
