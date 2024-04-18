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
    utils::{load_key, resolve_authority, validate_asset_permissions},
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

    // let (asset, plugin_header, plugin_registry) = fetch_core_data::<AssetV1>(ctx.accounts.asset)?;

    let validation_ctx = PluginValidationContext {
        self_authority: &Authority::UpdateAuthority,
        authority_info: authority,
        resolved_authorities: None,
        new_owner: None,
        target_plugin: None,
    };

    if ExternalPlugin::validate_add_external_plugin(&args.init_info, &validation_ctx)?
        == ValidationResult::Rejected
    {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Validate asset permissions.
    let (mut asset, _, _) = validate_asset_permissions(
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        None,
        Some(&args.init_info),
        AssetV1::check_add_external_plugin,
        CollectionV1::check_add_external_plugin,
        PluginType::check_add_external_plugin,
        AssetV1::validate_add_external_plugin,
        CollectionV1::validate_add_external_plugin,
        Plugin::validate_add_external_plugin,
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
    _args: AddCollectionExternalPluginV1Args,
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

    Err(MplCoreError::NotAvailable.into())
}

fn process_add_external_plugin<'a, T: DataBlob + SolanaAccount>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    init_info: &ExternalPluginInitInfo,
) -> ProgramResult {
    // Convert the ExternalPluginInitInfo into an ExternalPlugin.
    // let mut external_plugin = ExternalPlugin::from(init_info);

    // If the plugin is a LifecycleHook or DataStore, then we need to set the data offset and length.
    // match &mut external_plugin {
    //     ExternalPlugin::LifecycleHook(hook) => {
    //         hook.data_offset = 0;
    //         hook.data_len = 0;
    //     }
    //     ExternalPlugin::DataStore(data_store) => {
    //         data_store.data_offset = 0;
    //         data_store.data_len = 0;
    //     }
    //     _ => {}
    // };

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
