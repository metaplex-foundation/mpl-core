use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        AddCollectionExternalPluginAdapterV1Accounts, AddExternalPluginAdapterV1Accounts,
    },
    plugins::{
        create_meta_idempotent, initialize_external_plugin_adapter, ExternalPluginAdapter,
        ExternalPluginAdapterInitInfo, Plugin, PluginType, PluginValidationContext,
        ValidationResult,
    },
    state::{AssetV1, Authority, CollectionV1, DataBlob, Key, SolanaAccount},
    utils::{
        load_key, resolve_authority, validate_asset_permissions, validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddExternalPluginAdapterV1Args {
    /// External plugin adapter initialization info.
    pub init_info: ExternalPluginAdapterInitInfo,
}

pub(crate) fn add_external_plugin_adapter<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddExternalPluginAdapterV1Args,
) -> ProgramResult {
    let ctx = AddExternalPluginAdapterV1Accounts::context(accounts)?;

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

    // TODO: This should be handled in the validate call.
    match args.init_info {
        ExternalPluginAdapterInitInfo::LinkedLifecycleHook(_)
        | ExternalPluginAdapterInitInfo::LinkedAppData(_) => {
            return Err(MplCoreError::InvalidPluginAdapterTarget.into())
        }
        ExternalPluginAdapterInitInfo::DataSection(_) => {
            return Err(MplCoreError::CannotAddDataSection.into())
        }
        _ => (),
    }

    let external_plugin_adapter = ExternalPluginAdapter::from(&args.init_info);
    let external_plugin_adapter_authority = match &args.init_info {
        ExternalPluginAdapterInitInfo::LifecycleHook(lifecycle_hook) => {
            lifecycle_hook.init_plugin_authority
        }
        ExternalPluginAdapterInitInfo::Oracle(oracle) => oracle.init_plugin_authority,
        ExternalPluginAdapterInitInfo::AppData(app_data) => app_data.init_plugin_authority,
        ExternalPluginAdapterInitInfo::LinkedLifecycleHook(lifecycle_hook) => {
            lifecycle_hook.init_plugin_authority
        }
        ExternalPluginAdapterInitInfo::LinkedAppData(app_data) => app_data.init_plugin_authority,
        ExternalPluginAdapterInitInfo::DataSection(_) => unreachable!(),
    }
    .unwrap_or(Authority::UpdateAuthority);
    let validation_ctx = PluginValidationContext {
        accounts,
        asset_info: Some(ctx.accounts.asset),
        collection_info: ctx.accounts.collection,
        self_authority: &Authority::UpdateAuthority,
        authority_info: authority,
        resolved_authorities: None,
        new_owner: None,
        new_asset_authority: None,
        new_collection_authority: None,
        target_plugin: None,
        target_plugin_authority: None,
        target_external_plugin: Some(&external_plugin_adapter),
        target_external_plugin_authority: Some(&external_plugin_adapter_authority),
    };

    if ExternalPluginAdapter::validate_add_external_plugin_adapter(
        &external_plugin_adapter,
        &validation_ctx,
    )? == ValidationResult::Rejected
    {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Validate asset permissions.
    // Validate asset permissions.
    let (mut asset, _, _) = validate_asset_permissions(
        accounts,
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        None,
        None,
        None,
        Some(&external_plugin_adapter),
        Some(&external_plugin_adapter_authority),
        AssetV1::check_add_external_plugin_adapter,
        CollectionV1::check_add_external_plugin_adapter,
        PluginType::check_add_external_plugin_adapter,
        AssetV1::validate_add_external_plugin_adapter,
        CollectionV1::validate_add_external_plugin_adapter,
        Plugin::validate_add_external_plugin_adapter,
        None,
        None,
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    process_add_external_plugin_adapter::<AssetV1>(
        ctx.accounts.asset,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        &args.init_info,
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddCollectionExternalPluginAdapterV1Args {
    /// External plugin adapter initialization info.
    pub init_info: ExternalPluginAdapterInitInfo,
}

pub(crate) fn add_collection_external_plugin_adapter<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddCollectionExternalPluginAdapterV1Args,
) -> ProgramResult {
    let ctx = AddCollectionExternalPluginAdapterV1Accounts::context(accounts)?;

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

    if let ExternalPluginAdapterInitInfo::DataSection(_) = args.init_info {
        return Err(MplCoreError::CannotAddDataSection.into());
    }

    let external_plugin_adapter = ExternalPluginAdapter::from(&args.init_info);
    let external_plugin_adapter_authority = match &args.init_info {
        ExternalPluginAdapterInitInfo::LifecycleHook(lifecycle_hook) => {
            lifecycle_hook.init_plugin_authority
        }
        ExternalPluginAdapterInitInfo::Oracle(oracle) => oracle.init_plugin_authority,
        ExternalPluginAdapterInitInfo::AppData(app_data) => app_data.init_plugin_authority,
        ExternalPluginAdapterInitInfo::LinkedLifecycleHook(lifecycle_hook) => {
            lifecycle_hook.init_plugin_authority
        }
        ExternalPluginAdapterInitInfo::LinkedAppData(app_data) => app_data.init_plugin_authority,
        ExternalPluginAdapterInitInfo::DataSection(_) => unreachable!(),
    }
    .unwrap_or(Authority::UpdateAuthority);
    let validation_ctx = PluginValidationContext {
        accounts,
        asset_info: None,
        collection_info: Some(ctx.accounts.collection),
        self_authority: &Authority::UpdateAuthority,
        authority_info: authority,
        resolved_authorities: None,
        new_owner: None,
        new_asset_authority: None,
        new_collection_authority: None,
        target_plugin: None,
        target_plugin_authority: None,
        target_external_plugin: Some(&external_plugin_adapter),
        target_external_plugin_authority: Some(&external_plugin_adapter_authority),
    };

    if ExternalPluginAdapter::validate_add_external_plugin_adapter(
        &external_plugin_adapter,
        &validation_ctx,
    )? == ValidationResult::Rejected
    {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    let external_plugin_adapter = ExternalPluginAdapter::from(&args.init_info);

    // Validate collection permissions.
    let _ = validate_collection_permissions(
        accounts,
        authority,
        ctx.accounts.collection,
        None,
        None,
        None,
        Some(&external_plugin_adapter),
        Some(&external_plugin_adapter_authority),
        CollectionV1::check_add_external_plugin_adapter,
        PluginType::check_add_external_plugin_adapter,
        CollectionV1::validate_add_external_plugin_adapter,
        Plugin::validate_add_external_plugin_adapter,
        None,
        None,
    )?;

    process_add_external_plugin_adapter::<CollectionV1>(
        ctx.accounts.collection,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        &args.init_info,
    )
}

fn process_add_external_plugin_adapter<'a, T: DataBlob + SolanaAccount>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    init_info: &ExternalPluginAdapterInitInfo,
) -> ProgramResult {
    let (_, header_offset, mut plugin_header, mut plugin_registry) =
        create_meta_idempotent::<T>(account, payer, system_program)?;
    initialize_external_plugin_adapter::<T>(
        init_info,
        header_offset,
        &mut plugin_header,
        &mut plugin_registry,
        account,
        payer,
        system_program,
        None,
    )?;
    Ok(())
}
