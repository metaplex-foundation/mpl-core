use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        WriteCollectionExternalPluginAdapterDataV1Accounts,
        WriteExternalPluginAdapterDataV1Accounts,
    },
    plugins::{
        create_meta_idempotent, fetch_wrapped_external_plugin_adapter,
        initialize_external_plugin_adapter, update_external_plugin_adapter_data, AppData,
        DataSectionInitInfo, ExternalPluginAdapter, ExternalPluginAdapterInitInfo,
        ExternalPluginAdapterKey, ExternalRegistryRecord, LifecycleHook, LinkedAppData,
        LinkedDataKey, LinkedLifecycleHook, PluginHeaderV1, PluginRegistryV1,
    },
    state::{AssetV1, Authority, CollectionV1, DataBlob, Key, SolanaAccount},
    utils::{
        fetch_core_data, load_key, resolve_authority, resolve_pubkey_to_authorities,
        resolve_pubkey_to_authorities_collection,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct WriteExternalPluginAdapterDataV1Args {
    /// External plugin adapter key.
    pub key: ExternalPluginAdapterKey,
    /// The data to write.
    pub data: Option<Vec<u8>>,
}

pub(crate) fn write_external_plugin_adapter_data<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: WriteExternalPluginAdapterDataV1Args,
) -> ProgramResult {
    let ctx = WriteExternalPluginAdapterDataV1Accounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    let (asset, mut header, mut registry) = fetch_core_data::<AssetV1>(ctx.accounts.asset)?;

    let authorities = resolve_pubkey_to_authorities(authority, ctx.accounts.collection, &asset)?;

    if ctx.accounts.system_program.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if let Some(log_wrapper) = ctx.accounts.log_wrapper {
        if log_wrapper.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    if let Key::HashedAssetV1 = load_key(ctx.accounts.asset, 0)? {
        msg!("Error: Update plugin for compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    let (record, plugin) = match args.key {
        ExternalPluginAdapterKey::LifecycleHook(_) | ExternalPluginAdapterKey::AppData(_) => {
            fetch_wrapped_external_plugin_adapter::<AssetV1>(ctx.accounts.asset, None, &args.key)
        }
        ExternalPluginAdapterKey::LinkedLifecycleHook(_)
        | ExternalPluginAdapterKey::LinkedAppData(_) => {
            let collection = ctx
                .accounts
                .collection
                .ok_or(MplCoreError::MissingCollection)?;
            fetch_wrapped_external_plugin_adapter::<CollectionV1>(collection, None, &args.key)
        }
        _ => return Err(MplCoreError::UnsupportedOperation.into()),
    }?;

    process_write_external_plugin_data::<AssetV1>(
        ctx.accounts.asset,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        ctx.accounts.buffer,
        args.data.as_deref(),
        &asset,
        &record,
        &plugin,
        header.as_mut(),
        registry.as_mut(),
        &authorities,
        &args.key,
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct WriteCollectionExternalPluginAdapterDataV1Args {
    /// External plugin adapter key.
    pub key: ExternalPluginAdapterKey,
    /// The data to write.
    pub data: Option<Vec<u8>>,
}

pub(crate) fn write_collection_external_plugin_adapter_data<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: WriteCollectionExternalPluginAdapterDataV1Args,
) -> ProgramResult {
    let ctx = WriteCollectionExternalPluginAdapterDataV1Accounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    let (collection, mut header, mut registry) =
        fetch_core_data::<CollectionV1>(ctx.accounts.collection)?;

    let authorities = resolve_pubkey_to_authorities_collection(authority, ctx.accounts.collection)?;

    if ctx.accounts.system_program.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if let Some(log_wrapper) = ctx.accounts.log_wrapper {
        if log_wrapper.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    let (record, plugin) = fetch_wrapped_external_plugin_adapter::<CollectionV1>(
        ctx.accounts.collection,
        None,
        &args.key,
    )?;

    process_write_external_plugin_data::<CollectionV1>(
        ctx.accounts.collection,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        ctx.accounts.buffer,
        args.data.as_deref(),
        &collection,
        &record,
        &plugin,
        header.as_mut(),
        registry.as_mut(),
        &authorities,
        &args.key,
    )
}

#[allow(clippy::too_many_arguments)]
fn process_write_external_plugin_data<'a, T: DataBlob + SolanaAccount>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    buffer: Option<&AccountInfo<'a>>,
    data: Option<&[u8]>,
    core: &T,
    record: &ExternalRegistryRecord,
    wrapped_plugin: &ExternalPluginAdapter,
    header: Option<&mut PluginHeaderV1>,
    registry: Option<&mut PluginRegistryV1>,
    authorities: &[Authority],
    _key: &ExternalPluginAdapterKey,
) -> ProgramResult {
    // Check that the authority is the same as the plugin's data authority.
    match wrapped_plugin {
        ExternalPluginAdapter::LifecycleHook(LifecycleHook {
            data_authority: Some(data_authority),
            ..
        })
        | ExternalPluginAdapter::LinkedLifecycleHook(LinkedLifecycleHook {
            data_authority: Some(data_authority),
            ..
        })
        | ExternalPluginAdapter::AppData(AppData { data_authority, .. })
        | ExternalPluginAdapter::LinkedAppData(LinkedAppData { data_authority, .. }) => {
            if !authorities.contains(data_authority) {
                return Err(MplCoreError::InvalidAuthority.into());
            }
        }
        _ => return Err(MplCoreError::UnsupportedOperation.into()),
    }

    // AppData and LifecycleHook both write the data after the plugin.
    // LinkedAppData writes the data to the asset directly.
    match wrapped_plugin {
        ExternalPluginAdapter::LifecycleHook(_) | ExternalPluginAdapter::AppData(_) => {
            let header = header.ok_or(MplCoreError::PluginsNotInitialized)?;
            let registry = registry.ok_or(MplCoreError::PluginsNotInitialized)?;
            match (data, buffer) {
                (Some(data), None) => update_external_plugin_adapter_data(
                    record,
                    Some(core),
                    header,
                    registry,
                    account,
                    payer,
                    system_program,
                    data,
                ),
                (None, Some(buffer)) => update_external_plugin_adapter_data(
                    record,
                    Some(core),
                    header,
                    registry,
                    account,
                    payer,
                    system_program,
                    &buffer.data.borrow(),
                ),
                (Some(_), Some(_)) => Err(MplCoreError::TwoDataSources.into()),
                (None, None) => Err(MplCoreError::NoDataSources.into()),
            }
        }
        ExternalPluginAdapter::LinkedAppData(app_data) => {
            let (_, header_offset, mut header, mut registry) =
                create_meta_idempotent::<T>(account, payer, system_program)?;

            match fetch_wrapped_external_plugin_adapter(
                account,
                Some(core),
                &ExternalPluginAdapterKey::DataSection(LinkedDataKey::LinkedAppData(
                    app_data.data_authority,
                )),
            ) {
                Ok((section_record, _)) => match (data, buffer) {
                    (Some(data), None) => update_external_plugin_adapter_data(
                        &section_record,
                        Some(core),
                        &mut header,
                        &mut registry,
                        account,
                        payer,
                        system_program,
                        data,
                    ),
                    (None, Some(buffer)) => update_external_plugin_adapter_data(
                        &section_record,
                        Some(core),
                        &mut header,
                        &mut registry,
                        account,
                        payer,
                        system_program,
                        &buffer.data.borrow(),
                    ),
                    (Some(_), Some(_)) => Err(MplCoreError::TwoDataSources.into()),
                    (None, None) => Err(MplCoreError::NoDataSources.into()),
                },
                Err(ref e)
                    if *e
                        == ProgramError::Custom(
                            MplCoreError::ExternalPluginAdapterNotFound as u32,
                        ) =>
                {
                    match (data, buffer) {
                        (Some(data), None) => initialize_external_plugin_adapter::<T>(
                            &ExternalPluginAdapterInitInfo::DataSection(DataSectionInitInfo {
                                parent_key: LinkedDataKey::LinkedAppData(app_data.data_authority),
                                schema: app_data.schema,
                            }),
                            header_offset,
                            &mut header,
                            &mut registry,
                            account,
                            payer,
                            system_program,
                            Some(data),
                        ),
                        (None, Some(buffer)) => initialize_external_plugin_adapter::<T>(
                            &ExternalPluginAdapterInitInfo::DataSection(DataSectionInitInfo {
                                parent_key: LinkedDataKey::LinkedAppData(app_data.data_authority),
                                schema: app_data.schema,
                            }),
                            header_offset,
                            &mut header,
                            &mut registry,
                            account,
                            payer,
                            system_program,
                            Some(&buffer.data.borrow()),
                        ),
                        (Some(_), Some(_)) => Err(MplCoreError::TwoDataSources.into()),
                        (None, None) => Err(MplCoreError::NoDataSources.into()),
                    }
                }
                Err(e) => Err(e),
            }
        }
        _ => Err(MplCoreError::UnsupportedOperation.into()),
    }
}
