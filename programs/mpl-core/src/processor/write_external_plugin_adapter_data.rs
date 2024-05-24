use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        WriteCollectionExternalPluginAdapterDataV1Accounts,
        WriteExternalPluginAdapterDataV1Accounts,
    },
    plugins::{
        create_meta_idempotent, fetch_wrapped_external_plugin_adapter,
        initialize_external_plugin_adapter, update_external_plugin_adapter_data,
        AssetLinkedSecureDataStore, DataSectionInitInfo, ExternalPluginAdapter,
        ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey, ExternalRegistryRecord,
        LifecycleHook, LinkedDataKey, PluginHeaderV1, PluginRegistryV1, SecureDataStore,
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

    msg!("Fetching plugin");
    let (record, plugin) = match args.key {
        ExternalPluginAdapterKey::LifecycleHook(_)
        | ExternalPluginAdapterKey::SecureDataStore(_) => {
            msg!("Fetching plugin from Asset");
            fetch_wrapped_external_plugin_adapter::<AssetV1>(ctx.accounts.asset, None, &args.key)
        }
        ExternalPluginAdapterKey::AssetLinkedLifecycleHook(_)
        | ExternalPluginAdapterKey::AssetLinkedSecureDataStore(_) => {
            msg!("Fetching plugin from Collection");
            let collection = ctx
                .accounts
                .collection
                .ok_or(MplCoreError::MissingCollection)?;
            fetch_wrapped_external_plugin_adapter::<CollectionV1>(collection, None, &args.key)
        }
        _ => return Err(MplCoreError::UnsupportedOperation.into()),
    }?;

    msg!("Processing plugin");
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
    )?;

    let (_asset, _header, registry) = fetch_core_data::<AssetV1>(ctx.accounts.asset)?;

    if let Some(registry) = registry {
        msg!("Registry: {:?}", registry);
    }
    Ok(())
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
    msg!("Checking authority");
    match wrapped_plugin {
        ExternalPluginAdapter::LifecycleHook(LifecycleHook {
            data_authority: Some(data_authority),
            ..
        })
        | ExternalPluginAdapter::SecureDataStore(SecureDataStore { data_authority, .. })
        | ExternalPluginAdapter::AssetLinkedSecureDataStore(AssetLinkedSecureDataStore {
            data_authority,
            ..
        }) => {
            if !authorities.contains(data_authority) {
                return Err(MplCoreError::InvalidAuthority.into());
            }
        }
        _ => return Err(MplCoreError::UnsupportedOperation.into()),
    }

    // if data.is_some() && buffer.is_some() {
    //     return Err(MplCoreError::TwoDataSources.into());
    // } else if data.is_none() && buffer.is_none() {
    //     return Err(MplCoreError::NoDataSources.into());
    // }
    solana_program::msg!("Data: {:?}", data);

    // SecureDataStore and LifecycleHook both write the data after the plugin.
    // AssetLinkedSecureDataStore writes the data to the asset directly.
    match wrapped_plugin {
        ExternalPluginAdapter::LifecycleHook(_) | ExternalPluginAdapter::SecureDataStore(_) => {
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
        ExternalPluginAdapter::AssetLinkedSecureDataStore(data_store) => {
            let (_, mut header, mut registry) =
                create_meta_idempotent::<T>(account, payer, system_program)?;
            match fetch_wrapped_external_plugin_adapter(
                account,
                Some(core),
                &ExternalPluginAdapterKey::DataSection(LinkedDataKey::AssetLinkedSecureDataStore(
                    data_store.data_authority,
                )),
            ) {
                Ok(_) => update_external_plugin_adapter_data(
                    record,
                    Some(core),
                    &mut header,
                    &mut registry,
                    account,
                    payer,
                    system_program,
                    data.unwrap_or(&buffer.unwrap().data.borrow()),
                ),
                Err(_) => initialize_external_plugin_adapter::<T>(
                    &ExternalPluginAdapterInitInfo::DataSection(DataSectionInitInfo {
                        parent_key: LinkedDataKey::AssetLinkedSecureDataStore(
                            data_store.data_authority,
                        ),
                        schema: data_store.schema,
                    }),
                    Some(core),
                    &mut header,
                    &mut registry,
                    account,
                    payer,
                    system_program,
                    Some(data.unwrap_or(&buffer.unwrap().data.borrow())),
                ),
            }
        }
        _ => Err(MplCoreError::UnsupportedOperation.into()),
    }
}
