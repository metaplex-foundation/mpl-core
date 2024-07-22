use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_memory::sol_memcpy, pubkey::Pubkey,
};
use std::collections::HashSet;

use crate::{
    error::MplCoreError,
    plugins::{ExternalCheckResult, HookableLifecycleEvent},
    state::{AssetV1, Authority, CoreAsset, DataBlob, Key, SolanaAccount},
    utils::resize_or_reallocate_account,
};

use super::{
    ExternalPluginAdapter, ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey,
    ExternalPluginAdapterType, ExternalRegistryRecord, Plugin, PluginHeaderV1, PluginRegistryV1,
    PluginType, RegistryRecord,
};

/// Create plugin header and registry if it doesn't exist
pub fn create_meta_idempotent<'a, T: SolanaAccount + DataBlob>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> Result<(T, PluginHeaderV1, PluginRegistryV1), ProgramError> {
    let core = T::load(account, 0)?;
    let header_offset = core.get_size();

    // Check if the plugin header and registry exist.
    if header_offset == account.data_len() {
        // They don't exist, so create them.
        let header = PluginHeaderV1 {
            key: Key::PluginHeaderV1,
            plugin_registry_offset: header_offset + PluginHeaderV1::get_initial_size(),
        };
        let registry = PluginRegistryV1 {
            key: Key::PluginRegistryV1,
            registry: vec![],
            external_registry: vec![],
        };

        resize_or_reallocate_account(
            account,
            payer,
            system_program,
            header.plugin_registry_offset + PluginRegistryV1::get_initial_size(),
        )?;

        header.save(account, header_offset)?;
        registry.save(account, header.plugin_registry_offset)?;

        Ok((core, header, registry))
    } else {
        // They exist, so load them.
        let header = PluginHeaderV1::load(account, header_offset)?;
        let registry = PluginRegistryV1::load(account, header.plugin_registry_offset)?;

        Ok((core, header, registry))
    }
}

/// Create plugin header and registry
pub fn create_plugin_meta<'a, T: SolanaAccount + DataBlob>(
    asset: &T,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> Result<(PluginHeaderV1, PluginRegistryV1), ProgramError> {
    let header_offset = asset.get_size();

    // They don't exist, so create them.
    let header = PluginHeaderV1 {
        key: Key::PluginHeaderV1,
        plugin_registry_offset: header_offset + PluginHeaderV1::get_initial_size(),
    };
    let registry = PluginRegistryV1 {
        key: Key::PluginRegistryV1,
        registry: vec![],
        external_registry: vec![],
    };

    resize_or_reallocate_account(
        account,
        payer,
        system_program,
        header.plugin_registry_offset + PluginRegistryV1::get_initial_size(),
    )?;

    header.save(account, header_offset)?;
    registry.save(account, header.plugin_registry_offset)?;

    Ok((header, registry))
}

/// Assert that the Plugin metadata has been initialized.
pub fn assert_plugins_initialized(account: &AccountInfo) -> ProgramResult {
    let mut bytes: &[u8] = &(*account.data).borrow();
    let asset = AssetV1::deserialize(&mut bytes)?;

    if asset.get_size() == account.data_len() {
        return Err(MplCoreError::PluginsNotInitialized.into());
    }

    Ok(())
}

/// Fetch the plugin from the registry.
pub fn fetch_plugin<T: DataBlob + SolanaAccount, U: BorshDeserialize>(
    account: &AccountInfo,
    plugin_type: PluginType,
) -> Result<(Authority, U, usize), ProgramError> {
    let asset = T::load(account, 0)?;

    if asset.get_size() == account.data_len() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let header = PluginHeaderV1::load(account, asset.get_size())?;
    let PluginRegistryV1 { registry, .. } =
        PluginRegistryV1::load(account, header.plugin_registry_offset)?;

    // Find the plugin in the registry.
    let registry_record = registry
        .iter()
        .find(|record| record.plugin_type == plugin_type)
        .ok_or(MplCoreError::PluginNotFound)?;

    // Deserialize the plugin.
    let plugin = Plugin::deserialize(&mut &(*account.data).borrow()[registry_record.offset..])?;

    if PluginType::from(&plugin) != plugin_type {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let inner = U::deserialize(
        &mut &(*account.data).borrow()[registry_record
            .offset
            .checked_add(1)
            .ok_or(MplCoreError::NumericalOverflow)?..],
    )?;

    // Return the plugin and its authority.
    Ok((registry_record.authority, inner, registry_record.offset))
}

/// Fetch the plugin from the registry.
pub fn fetch_wrapped_plugin<T: DataBlob + SolanaAccount>(
    account: &AccountInfo,
    core: Option<&T>,
    plugin_type: PluginType,
) -> Result<(Authority, Plugin), ProgramError> {
    let size = match core {
        Some(core) => core.get_size(),
        None => {
            let asset = T::load(account, 0)?;

            if asset.get_size() == account.data_len() {
                return Err(MplCoreError::PluginNotFound.into());
            }

            asset.get_size()
        }
    };

    let header = PluginHeaderV1::load(account, size)?;
    let PluginRegistryV1 { registry, .. } =
        PluginRegistryV1::load(account, header.plugin_registry_offset)?;

    // Find the plugin in the registry.
    let registry_record = registry
        .iter()
        .find(|record| record.plugin_type == plugin_type)
        .ok_or(MplCoreError::PluginNotFound)?;

    // Deserialize the plugin.
    let plugin = Plugin::deserialize(&mut &(*account.data).borrow()[registry_record.offset..])?;

    // Return the plugin and its authority.
    Ok((registry_record.authority, plugin))
}

/// Fetch the external plugin adapter from the registry.
pub fn fetch_wrapped_external_plugin_adapter<T: DataBlob + SolanaAccount>(
    account: &AccountInfo,
    core: Option<&T>,
    plugin_key: &ExternalPluginAdapterKey,
) -> Result<(Authority, ExternalPluginAdapter), ProgramError> {
    let size = match core {
        Some(core) => core.get_size(),
        None => {
            let asset = T::load(account, 0)?;

            if asset.get_size() == account.data_len() {
                return Err(MplCoreError::ExternalPluginAdapterNotFound.into());
            }

            asset.get_size()
        }
    };

    let header = PluginHeaderV1::load(account, size)?;
    let plugin_registry = PluginRegistryV1::load(account, header.plugin_registry_offset)?;

    // Find the plugin in the registry.
    let result = find_external_plugin_adapter(&plugin_registry, plugin_key, account)?;

    if let (_, Some(record)) = result {
        // Deserialize the plugin.
        let plugin =
            ExternalPluginAdapter::deserialize(&mut &(*account.data).borrow()[record.offset..])?;

        // Return the plugin and its authority.
        Ok((record.authority, plugin))
    } else {
        Err(MplCoreError::ExternalPluginAdapterNotFound.into())
    }
}

/// Fetch the plugin registry.
pub fn fetch_plugins(account: &AccountInfo) -> Result<Vec<RegistryRecord>, ProgramError> {
    let asset = AssetV1::load(account, 0)?;

    if asset.get_size() == account.data_len() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let header = PluginHeaderV1::load(account, asset.get_size())?;
    let PluginRegistryV1 { registry, .. } =
        PluginRegistryV1::load(account, header.plugin_registry_offset)?;

    Ok(registry)
}

/// List all plugins in an account.
pub fn list_plugins(account: &AccountInfo) -> Result<Vec<PluginType>, ProgramError> {
    let asset = AssetV1::load(account, 0)?;

    if asset.get_size() == account.data_len() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let header = PluginHeaderV1::load(account, asset.get_size())?;
    let PluginRegistryV1 { registry, .. } =
        PluginRegistryV1::load(account, header.plugin_registry_offset)?;

    Ok(registry
        .iter()
        .map(|registry_record| registry_record.plugin_type)
        .collect())
}

/// Add a plugin to the registry and initialize it.
pub fn initialize_plugin<'a, T: DataBlob + SolanaAccount>(
    plugin: &Plugin,
    authority: &Authority,
    plugin_header: &mut PluginHeaderV1,
    plugin_registry: &mut PluginRegistryV1,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let core = T::load(account, 0)?;
    let header_offset = core.get_size();
    let plugin_type = plugin.into();
    let plugin_data = plugin.try_to_vec()?;
    let plugin_size = plugin_data.len();

    // You cannot add a duplicate plugin.
    if plugin_registry
        .registry
        .iter()
        .any(|record| record.plugin_type == plugin_type)
    {
        return Err(MplCoreError::PluginAlreadyExists.into());
    }

    let old_registry_offset = plugin_header.plugin_registry_offset;

    let new_registry_record = RegistryRecord {
        plugin_type,
        offset: old_registry_offset,
        authority: *authority,
    };

    let size_increase = plugin_size
        .checked_add(new_registry_record.try_to_vec()?.len())
        .ok_or(MplCoreError::NumericalOverflow)?;

    let new_registry_offset = plugin_header
        .plugin_registry_offset
        .checked_add(plugin_size)
        .ok_or(MplCoreError::NumericalOverflow)?;

    plugin_header.plugin_registry_offset = new_registry_offset;

    plugin_registry.registry.push(new_registry_record);

    let new_size = account
        .data_len()
        .checked_add(size_increase)
        .ok_or(MplCoreError::NumericalOverflow)?;

    resize_or_reallocate_account(account, payer, system_program, new_size)?;
    plugin_header.save(account, header_offset)?;
    plugin.save(account, old_registry_offset)?;
    plugin_registry.save(account, new_registry_offset)?;

    Ok(())
}

/// Add an external plugin adapter to the registry and initialize it.
pub fn initialize_external_plugin_adapter<'a, T: DataBlob + SolanaAccount>(
    init_info: &ExternalPluginAdapterInitInfo,
    plugin_header: &mut PluginHeaderV1,
    plugin_registry: &mut PluginRegistryV1,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let core = T::load(account, 0)?;
    let header_offset = core.get_size();
    let plugin_type = init_info.into();

    // Note currently we are blocking adding LifecycleHook and DataStore external plugin adapters as they
    // are still in development.
    match init_info {
        ExternalPluginAdapterInitInfo::LifecycleHook(_)
        | ExternalPluginAdapterInitInfo::DataStore(_) => {
            return Err(MplCoreError::NotAvailable.into());
        }
        ExternalPluginAdapterInitInfo::Oracle(_) => (),
    }

    // You cannot add a duplicate plugin.
    for record in plugin_registry.external_registry.iter() {
        if ExternalPluginAdapterKey::from_record(account, record)?
            == ExternalPluginAdapterKey::from(init_info)
        {
            return Err(MplCoreError::ExternalPluginAdapterAlreadyExists.into());
        }
    }

    let (authority, lifecycle_checks) = match init_info {
        ExternalPluginAdapterInitInfo::LifecycleHook(init_info) => {
            validate_lifecycle_checks(&init_info.lifecycle_checks, false)?;
            (
                init_info.init_plugin_authority,
                Some(init_info.lifecycle_checks.clone()),
            )
        }
        ExternalPluginAdapterInitInfo::Oracle(init_info) => {
            validate_lifecycle_checks(&init_info.lifecycle_checks, true)?;
            (
                init_info.init_plugin_authority,
                Some(init_info.lifecycle_checks.clone()),
            )
        }
        ExternalPluginAdapterInitInfo::DataStore(init_info) => {
            (init_info.init_plugin_authority, None)
        }
    };

    let old_registry_offset = plugin_header.plugin_registry_offset;

    let mut new_registry_record = ExternalRegistryRecord {
        plugin_type,
        authority: authority.unwrap_or(Authority::UpdateAuthority),
        lifecycle_checks: lifecycle_checks.clone(),
        offset: old_registry_offset,
        data_offset: None,
        data_len: None,
    };

    let mut plugin = ExternalPluginAdapter::from(init_info);

    // If the plugin is a LifecycleHook or DataStore, then we need to set the data offset and length.
    match &mut plugin {
        ExternalPluginAdapter::LifecycleHook(_) | ExternalPluginAdapter::DataStore(_) => {
            new_registry_record.data_offset = Some(old_registry_offset);
            new_registry_record.data_len = Some(0);
        }
        _ => {}
    };

    let plugin_metadata = plugin.try_to_vec()?;
    let plugin_size = plugin_metadata.len();

    let size_increase = plugin_size
        .checked_add(new_registry_record.try_to_vec()?.len())
        .ok_or(MplCoreError::NumericalOverflow)?;

    let new_registry_offset = plugin_header
        .plugin_registry_offset
        .checked_add(plugin_size)
        .ok_or(MplCoreError::NumericalOverflow)?;

    plugin_header.plugin_registry_offset = new_registry_offset;

    plugin_registry.external_registry.push(new_registry_record);

    let new_size = account
        .data_len()
        .checked_add(size_increase)
        .ok_or(MplCoreError::NumericalOverflow)?;

    resize_or_reallocate_account(account, payer, system_program, new_size)?;
    plugin_header.save(account, header_offset)?;
    plugin.save(account, old_registry_offset)?;
    plugin_registry.save(account, new_registry_offset)?;

    Ok(())
}

pub(crate) fn validate_lifecycle_checks(
    lifecycle_checks: &[(HookableLifecycleEvent, ExternalCheckResult)],
    can_reject_only: bool,
) -> ProgramResult {
    if lifecycle_checks.is_empty() {
        return Err(MplCoreError::RequiresLifecycleCheck.into());
    }

    let mut seen_events = HashSet::new();
    if !lifecycle_checks
        .iter()
        .all(|(event, _)| seen_events.insert(event))
    {
        // If `insert` returns false, it means the event was already in the set, indicating a duplicate
        return Err(MplCoreError::DuplicateLifecycleChecks.into());
    }

    if can_reject_only {
        let reject_only = ExternalCheckResult::can_reject_only();
        if lifecycle_checks
            .iter()
            .any(|(_, result)| *result != reject_only)
        {
            return Err(MplCoreError::OracleCanRejectOnly.into());
        }
    }

    Ok(())
}

/// Remove a plugin from the registry and delete it.
pub fn delete_plugin<'a, T: DataBlob>(
    plugin_type: &PluginType,
    asset: &T,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    if asset.get_size() == account.data_len() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    //TODO: Bytemuck this.
    let mut header = PluginHeaderV1::load(account, asset.get_size())?;
    let mut plugin_registry = PluginRegistryV1::load(account, header.plugin_registry_offset)?;

    if let Some(index) = plugin_registry
        .registry
        .iter_mut()
        .position(|record| record.plugin_type == *plugin_type)
    {
        let registry_record = plugin_registry.registry.remove(index);
        let serialized_registry_record = registry_record.try_to_vec()?;

        // Fetch the offset of the plugin to be removed.
        let plugin_offset = registry_record.offset;
        let plugin = Plugin::load(account, plugin_offset)?;
        let serialized_plugin = plugin.try_to_vec()?;

        // Get the offset of the plugin after the one being removed.
        let next_plugin_offset = plugin_offset
            .checked_add(serialized_plugin.len())
            .ok_or(MplCoreError::NumericalOverflow)?;

        // Calculate the new size of the account.
        let new_size = account
            .data_len()
            .checked_sub(serialized_registry_record.len())
            .ok_or(MplCoreError::NumericalOverflow)?
            .checked_sub(serialized_plugin.len())
            .ok_or(MplCoreError::NumericalOverflow)?;

        let new_registry_offset = header
            .plugin_registry_offset
            .checked_sub(serialized_plugin.len())
            .ok_or(MplCoreError::NumericalOverflow)?;

        let data_to_move = header
            .plugin_registry_offset
            .checked_sub(next_plugin_offset)
            .ok_or(MplCoreError::NumericalOverflow)?;

        //TODO: This is memory intensive, we should use memmove instead probably.
        let src = account.data.borrow()[next_plugin_offset..].to_vec();
        sol_memcpy(
            &mut account.data.borrow_mut()[plugin_offset..],
            &src,
            data_to_move,
        );

        header.plugin_registry_offset = new_registry_offset;
        header.save(account, asset.get_size())?;

        // Move offsets for existing registry records.
        for record in &mut plugin_registry.registry {
            if plugin_offset < record.offset {
                record.offset -= serialized_plugin.len()
            }
        }

        for record in &mut plugin_registry.external_registry {
            if plugin_offset < record.offset {
                record.offset -= serialized_plugin.len()
            }
        }

        plugin_registry.save(account, new_registry_offset)?;

        resize_or_reallocate_account(account, payer, system_program, new_size)?;
    } else {
        return Err(MplCoreError::PluginNotFound.into());
    }

    Ok(())
}

/// Remove a plugin from the registry and delete it.
pub fn delete_external_plugin_adapter<'a, T: DataBlob>(
    plugin_key: &ExternalPluginAdapterKey,
    asset: &T,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    if asset.get_size() == account.data_len() {
        return Err(MplCoreError::ExternalPluginAdapterNotFound.into());
    }

    //TODO: Bytemuck this.
    let mut header = PluginHeaderV1::load(account, asset.get_size())?;
    let mut plugin_registry = PluginRegistryV1::load(account, header.plugin_registry_offset)?;

    let result = find_external_plugin_adapter(&plugin_registry, plugin_key, account)?;

    if let (Some(index), _) = result {
        let registry_record = plugin_registry.external_registry.remove(index);
        let serialized_registry_record = registry_record.try_to_vec()?;

        // Fetch the offset of the plugin to be removed.
        let plugin_offset = registry_record.offset;
        let plugin = ExternalPluginAdapter::load(account, plugin_offset)?;
        let serialized_plugin = plugin.try_to_vec()?;

        // Get the offset of the plugin after the one being removed.
        let next_plugin_offset = plugin_offset
            .checked_add(serialized_plugin.len())
            .ok_or(MplCoreError::NumericalOverflow)?;

        // Calculate the new size of the account.
        let new_size = account
            .data_len()
            .checked_sub(serialized_registry_record.len())
            .ok_or(MplCoreError::NumericalOverflow)?
            .checked_sub(serialized_plugin.len())
            .ok_or(MplCoreError::NumericalOverflow)?;

        let new_registry_offset = header
            .plugin_registry_offset
            .checked_sub(serialized_plugin.len())
            .ok_or(MplCoreError::NumericalOverflow)?;

        let data_to_move = header
            .plugin_registry_offset
            .checked_sub(next_plugin_offset)
            .ok_or(MplCoreError::NumericalOverflow)?;

        //TODO: This is memory intensive, we should use memmove instead probably.
        let src = account.data.borrow()[next_plugin_offset..].to_vec();
        sol_memcpy(
            &mut account.data.borrow_mut()[plugin_offset..],
            &src,
            data_to_move,
        );

        header.plugin_registry_offset = new_registry_offset;
        header.save(account, asset.get_size())?;

        // Move offsets for existing registry records.
        for record in &mut plugin_registry.external_registry {
            if plugin_offset < record.offset {
                record.offset -= serialized_plugin.len()
            }
        }

        for record in &mut plugin_registry.registry {
            if plugin_offset < record.offset {
                record.offset -= serialized_plugin.len()
            }
        }

        plugin_registry.save(account, new_registry_offset)?;

        resize_or_reallocate_account(account, payer, system_program, new_size)?;
    } else {
        return Err(MplCoreError::ExternalPluginAdapterNotFound.into());
    }

    Ok(())
}

/// Add an authority to a plugin.
#[allow(clippy::too_many_arguments)]
pub fn approve_authority_on_plugin<'a, T: CoreAsset>(
    plugin_type: &PluginType,
    new_authority: &Authority,
    account: &AccountInfo<'a>,
    plugin_header: &PluginHeaderV1,
    plugin_registry: &mut PluginRegistryV1,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let registry_record = &mut plugin_registry
        .registry
        .iter_mut()
        .find(|record| record.plugin_type == *plugin_type)
        .ok_or(MplCoreError::PluginNotFound)?;

    registry_record.authority = *new_authority;

    let authority_bytes = new_authority.try_to_vec()?;

    let new_size = account
        .data_len()
        .checked_add(authority_bytes.len())
        .ok_or(MplCoreError::NumericalOverflow)?;
    resize_or_reallocate_account(account, payer, system_program, new_size)?;

    plugin_registry.save(account, plugin_header.plugin_registry_offset)?;

    Ok(())
}

/// Remove an authority from a plugin.
#[allow(clippy::too_many_arguments)]
pub fn revoke_authority_on_plugin<'a>(
    plugin_type: &PluginType,
    account: &AccountInfo<'a>,
    plugin_header: &PluginHeaderV1,
    plugin_registry: &mut PluginRegistryV1,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let registry_record = &mut plugin_registry
        .registry
        .iter_mut()
        .find(|record| record.plugin_type == *plugin_type)
        .ok_or(MplCoreError::PluginNotFound)?;

    let old_authority_bytes = registry_record.authority.try_to_vec()?;
    registry_record.authority = registry_record.plugin_type.manager();
    let new_authority_bytes = registry_record.authority.try_to_vec()?;

    let size_diff = (new_authority_bytes.len() as isize)
        .checked_sub(old_authority_bytes.len() as isize)
        .ok_or(MplCoreError::NumericalOverflow)?;

    let new_size = (account.data_len() as isize)
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;
    resize_or_reallocate_account(account, payer, system_program, new_size as usize)?;

    plugin_registry.save(account, plugin_header.plugin_registry_offset)?;

    Ok(())
}

pub(crate) fn find_external_plugin_adapter<'b>(
    plugin_registry: &'b PluginRegistryV1,
    plugin_key: &ExternalPluginAdapterKey,
    account: &AccountInfo<'_>,
) -> Result<(Option<usize>, Option<&'b ExternalRegistryRecord>), ProgramError> {
    for (i, record) in plugin_registry.external_registry.iter().enumerate() {
        if check_plugin_key(record, plugin_key, account)? {
            return Ok((Some(i), Some(record)));
        }
    }

    Ok((None, None))
}

pub(crate) fn find_external_plugin_adapter_mut<'b>(
    plugin_registry: &'b mut PluginRegistryV1,
    plugin_key: &ExternalPluginAdapterKey,
    account: &AccountInfo<'_>,
) -> Result<(Option<usize>, Option<&'b mut ExternalRegistryRecord>), ProgramError> {
    for (i, record) in plugin_registry.external_registry.iter_mut().enumerate() {
        let record_ref = &*record;

        if check_plugin_key(record_ref, plugin_key, account)? {
            return Ok((Some(i), Some(record)));
        }
    }

    Ok((None, None))
}

fn check_plugin_key(
    record_ref: &ExternalRegistryRecord,
    plugin_key: &ExternalPluginAdapterKey,
    account: &AccountInfo,
) -> Result<bool, ProgramError> {
    if record_ref.plugin_type == ExternalPluginAdapterType::from(plugin_key)
        && (match plugin_key {
            ExternalPluginAdapterKey::LifecycleHook(address)
            | ExternalPluginAdapterKey::Oracle(address) => {
                let pubkey_offset = record_ref
                    .offset
                    .checked_add(1)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                address
                    == &match Pubkey::deserialize(&mut &account.data.borrow()[pubkey_offset..]) {
                        Ok(address) => address,
                        Err(_) => return Err(MplCoreError::DeserializationError.into()),
                    }
            }
            ExternalPluginAdapterKey::DataStore(authority) => {
                let authority_offset = record_ref
                    .offset
                    .checked_add(1)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                authority
                    == &match Authority::deserialize(
                        &mut &account.data.borrow()[authority_offset..],
                    ) {
                        Ok(authority) => authority,
                        Err(_) => return Err(MplCoreError::DeserializationError.into()),
                    }
            }
        })
    {
        Ok(true)
    } else {
        Ok(false)
    }
}
