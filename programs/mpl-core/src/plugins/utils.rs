use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_memory::{sol_memcpy, sol_memmove},
    pubkey::Pubkey,
};
use std::collections::HashSet;

use crate::{
    error::MplCoreError,
    plugins::{ExternalCheckResult, HookableLifecycleEvent},
    state::{AssetV1, Authority, CoreAsset, DataBlob, Key, SolanaAccount},
    utils::resize_or_reallocate_account,
};

use super::{
    AppDataInitInfo, ExternalPluginAdapter, ExternalPluginAdapterInitInfo,
    ExternalPluginAdapterKey, ExternalPluginAdapterType, ExternalRegistryRecord,
    LinkedAppDataInitInfo, LinkedDataKey, Plugin, PluginHeaderV1, PluginRegistryV1, PluginType,
    RegistryRecord,
};

/// Create plugin header and registry if it doesn't exist
pub fn create_meta_idempotent<'a, T: SolanaAccount + DataBlob>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> Result<(T, usize, PluginHeaderV1, PluginRegistryV1), ProgramError> {
    let core = T::load(account, 0)?;
    let header_offset = core.len();

    // Check if the plugin header and registry exist.
    if header_offset == account.data_len() {
        // They don't exist, so create them.
        let header = PluginHeaderV1 {
            key: Key::PluginHeaderV1,
            plugin_registry_offset: header_offset
                + 1 // Plugin Header Key
                + 8, // Plugin Registry Offset
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
            header.plugin_registry_offset + registry.len(),
        )?;

        header.save(account, header_offset)?;
        registry.save(account, header.plugin_registry_offset)?;

        Ok((core, header_offset, header, registry))
    } else {
        // They exist, so load them.
        let header = PluginHeaderV1::load(account, header_offset)?;
        let registry = PluginRegistryV1::load(account, header.plugin_registry_offset)?;

        Ok((core, header_offset, header, registry))
    }
}

/// Create plugin header and registry
pub fn create_plugin_meta<'a, T: SolanaAccount + DataBlob>(
    asset: &T,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> Result<(usize, PluginHeaderV1, PluginRegistryV1), ProgramError> {
    let header_offset = asset.len();

    // They don't exist, so create them.
    let header = PluginHeaderV1 {
        key: Key::PluginHeaderV1,
        plugin_registry_offset: header_offset
            + 1 // Plugin Header Key
            + 8, // Plugin Registry Offset
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
        header.plugin_registry_offset + registry.len(),
    )?;

    header.save(account, header_offset)?;
    registry.save(account, header.plugin_registry_offset)?;

    Ok((header_offset, header, registry))
}

/// Assert that the Plugin metadata has been initialized.
pub fn assert_plugins_initialized(account: &AccountInfo) -> ProgramResult {
    let mut bytes: &[u8] = &(*account.data).borrow();
    let asset = AssetV1::deserialize(&mut bytes)?;

    if asset.len() == account.data_len() {
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

    if asset.len() == account.data_len() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let header = PluginHeaderV1::load(account, asset.len())?;
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
        Some(core) => core.len(),
        None => {
            let asset = T::load(account, 0)?;

            if asset.len() == account.data_len() {
                return Err(MplCoreError::PluginNotFound.into());
            }

            asset.len()
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
) -> Result<(ExternalRegistryRecord, ExternalPluginAdapter), ProgramError> {
    let size = match core {
        Some(core) => core.len(),
        None => {
            let asset = T::load(account, 0)?;

            if asset.len() == account.data_len() {
                return Err(MplCoreError::ExternalPluginAdapterNotFound.into());
            }

            asset.len()
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
        Ok((record.clone(), plugin))
    } else {
        Err(MplCoreError::ExternalPluginAdapterNotFound.into())
    }
}

/// Fetch the plugin registry.
pub fn fetch_plugins(account: &AccountInfo) -> Result<Vec<RegistryRecord>, ProgramError> {
    let asset = AssetV1::load(account, 0)?;

    if asset.len() == account.data_len() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let header = PluginHeaderV1::load(account, asset.len())?;
    let PluginRegistryV1 { registry, .. } =
        PluginRegistryV1::load(account, header.plugin_registry_offset)?;

    Ok(registry)
}

/// List all plugins in an account.
pub fn list_plugins<T: DataBlob + SolanaAccount>(
    account: &AccountInfo,
) -> Result<Vec<PluginType>, ProgramError> {
    let asset = T::load(account, 0)?;

    if asset.len() == account.data_len() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let header = PluginHeaderV1::load(account, asset.len())?;
    let PluginRegistryV1 { registry, .. } =
        PluginRegistryV1::load(account, header.plugin_registry_offset)?;

    Ok(registry
        .iter()
        .map(|registry_record| registry_record.plugin_type)
        .collect())
}

/// Add a plugin to the registry and initialize it.
#[allow(clippy::too_many_arguments)]
pub fn initialize_plugin<'a, T: DataBlob + SolanaAccount>(
    plugin: &Plugin,
    authority: &Authority,
    header_offset: usize,
    plugin_header: &mut PluginHeaderV1,
    plugin_registry: &mut PluginRegistryV1,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let plugin_type = plugin.into();
    let plugin_size = plugin.len();

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
        .checked_add(new_registry_record.len())
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
#[allow(clippy::too_many_arguments)]
pub fn initialize_external_plugin_adapter<'a, T: DataBlob + SolanaAccount>(
    init_info: &ExternalPluginAdapterInitInfo,
    header_offset: usize,
    plugin_header: &mut PluginHeaderV1,
    plugin_registry: &mut PluginRegistryV1,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    appended_data: Option<&[u8]>,
) -> ProgramResult {
    let plugin_type = init_info.into();

    // Note currently we are blocking adding LifecycleHook and LinkedLifecycleHook external plugin adapters as they
    // are still in development.
    match init_info {
        ExternalPluginAdapterInitInfo::LifecycleHook(_)
        | ExternalPluginAdapterInitInfo::LinkedLifecycleHook(_) => {
            return Err(MplCoreError::NotAvailable.into())
        }
        _ => {}
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
        ExternalPluginAdapterInitInfo::LinkedLifecycleHook(init_info) => {
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
        ExternalPluginAdapterInitInfo::AppData(AppDataInitInfo {
            init_plugin_authority,
            ..
        })
        | ExternalPluginAdapterInitInfo::LinkedAppData(LinkedAppDataInitInfo {
            init_plugin_authority,
            ..
        }) => (*init_plugin_authority, None),
        // The DataSection is only updated via its managing plugin so it has no authority.
        ExternalPluginAdapterInitInfo::DataSection(_) => (Some(Authority::None), None),
        ExternalPluginAdapterInitInfo::AgentIdentity(init_info) => {
            validate_lifecycle_checks(&init_info.lifecycle_checks, false)?;
            (
                init_info.init_plugin_authority,
                Some(init_info.lifecycle_checks.clone()),
            )
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

    let plugin = ExternalPluginAdapter::from(init_info);

    // If the plugin is a LifecycleHook or AppData, then we need to set the data offset and length.
    match plugin {
        ExternalPluginAdapter::LifecycleHook(_)
        | ExternalPluginAdapter::AppData(_)
        | ExternalPluginAdapter::DataSection(_) => {
            // Here we use a 0 value for the data offset as it will be updated after the data is appended.
            new_registry_record.data_offset = Some(0);
            new_registry_record.data_len = match appended_data {
                Some(data) => Some(data.len()),
                None => Some(0),
            };
        }
        _ => {}
    };

    let serialized_plugin = borsh::to_vec(&plugin)?;
    let plugin_size = serialized_plugin.len();

    let size_increase = plugin_size
        .checked_add(borsh::to_vec(&new_registry_record)?.len())
        .ok_or(MplCoreError::NumericalOverflow)?
        .checked_add(new_registry_record.data_len.unwrap_or(0))
        .ok_or(MplCoreError::NumericalOverflow)?;

    let data_offset = plugin_header
        .plugin_registry_offset
        .checked_add(plugin_size)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // If the data offset has been initialized, then we need to set it to the correct value.
    if new_registry_record.data_offset.is_some() {
        new_registry_record.data_offset = Some(data_offset);
    }

    let new_registry_offset = data_offset
        .checked_add(new_registry_record.data_len.unwrap_or(0))
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

    if let Some(data) = appended_data {
        // SAFETY: `data` cannot alias the account data, and the account was just
        // resized to fit `data.len()` bytes at `data_offset`.
        unsafe {
            sol_memcpy(
                &mut account.data.borrow_mut()[data_offset..],
                data,
                data.len(),
            );
        }
    };

    plugin_registry.save(account, new_registry_offset)?;

    Ok(())
}

/// Add an external plugin adapter to the registry and initialize it.
#[allow(clippy::too_many_arguments)]
pub fn update_external_plugin_adapter_data<'a, T: DataBlob + SolanaAccount>(
    record: &ExternalRegistryRecord,
    core: Option<&T>,
    plugin_header: &mut PluginHeaderV1,
    plugin_registry: &mut PluginRegistryV1,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    data: &[u8],
) -> ProgramResult {
    // Extract the data offset and data length as they should always be set.
    let data_offset = record.data_offset.ok_or(MplCoreError::InvalidPlugin)?;
    let data_len = record.data_len.ok_or(MplCoreError::InvalidPlugin)?;
    let new_data_len = data.len();
    let old_registry_offset = plugin_header.plugin_registry_offset;
    let size_diff = (new_data_len as isize)
        .checked_sub(data_len as isize)
        .ok_or(MplCoreError::NumericalOverflow)?;

    let next_plugin_offset = data_offset
        .checked_add(data_len)
        .ok_or(MplCoreError::NumericalOverflow)?;
    let new_next_plugin_offset: usize = (next_plugin_offset as isize)
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?
        .try_into()
        .map_err(|_| MplCoreError::NumericalOverflow)?;

    // Update any offsets that will change.
    plugin_registry.bump_offsets(record.offset, size_diff)?;

    let new_registry_offset: usize = (old_registry_offset as isize)
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?
        .try_into()
        .map_err(|_| MplCoreError::NumericalOverflow)?;
    plugin_header.plugin_registry_offset = new_registry_offset;

    let new_size: usize = (account.data_len() as isize)
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?
        .try_into()
        .map_err(|_| MplCoreError::NumericalOverflow)?;

    // Capture old data length before any realloc, as realloc changes data_len().
    let old_data_len = account.data_len();

    if size_diff > 0 {
        // Growing: realloc first to make room for the rightward shift.
        resize_or_reallocate_account(account, payer, system_program, new_size)?;
    }

    // SAFETY: `borrow_mut` will always return a valid pointer.
    // new_next_plugin_offset is derived from next_plugin_offset and size_diff using
    // checked arithmetic, so it will always be less than or equal to account.data_len().
    // This will fail and revert state if there is a memory violation.
    unsafe {
        let base = account.data.borrow_mut().as_mut_ptr();
        sol_memmove(
            base.add(new_next_plugin_offset),
            base.add(next_plugin_offset),
            old_data_len.saturating_sub(next_plugin_offset),
        )
    }

    if size_diff < 0 {
        // Shrinking: realloc after memmove to preserve data before truncation.
        resize_or_reallocate_account(account, payer, system_program, new_size)?;
    }

    // SAFETY: `data` cannot alias the account data, and the account is sized to
    // fit `new_data_len` bytes at `data_offset`.
    unsafe {
        sol_memcpy(
            &mut account.data.borrow_mut()[data_offset..],
            data,
            new_data_len,
        );
    }

    // Find the record in the registry and update the data length.
    let record_index = plugin_registry
        .external_registry
        .iter()
        .position(|r| r == record)
        .ok_or(MplCoreError::InvalidPlugin)?;
    plugin_registry.external_registry[record_index].data_len = Some(new_data_len);

    plugin_registry.save(account, new_registry_offset)?;
    plugin_header.save(account, core.map_or(0, |core| core.len()))?;

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
    if asset.len() == account.data_len() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    //TODO: Bytemuck this.
    let mut header = PluginHeaderV1::load(account, asset.len())?;
    let mut plugin_registry = PluginRegistryV1::load(account, header.plugin_registry_offset)?;

    if let Some(index) = plugin_registry
        .registry
        .iter_mut()
        .position(|record| record.plugin_type == *plugin_type)
    {
        let registry_record = plugin_registry.registry.remove(index);
        let serialized_registry_record = borsh::to_vec(&registry_record)?;

        // Fetch the offset of the plugin to be removed.
        let plugin_offset = registry_record.offset;
        let plugin = Plugin::load(account, plugin_offset)?;
        let serialized_plugin = borsh::to_vec(&plugin)?;

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

        // SAFETY: `borrow_mut` will always return a valid pointer.
        // plugin_offset is derived from next_plugin_offset and size_diff using
        // checked arithmetic, so it will always be less than or equal to account.data_len().
        // This will fail and revert state if there is a memory violation.
        unsafe {
            let base = account.data.borrow_mut().as_mut_ptr();
            sol_memmove(
                base.add(plugin_offset),
                base.add(next_plugin_offset),
                data_to_move,
            );
        }

        header.plugin_registry_offset = new_registry_offset;
        header.save(account, asset.len())?;

        // Move offsets for existing registry records.
        plugin_registry.bump_offsets(plugin_offset, -(serialized_plugin.len() as isize))?;

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
    if asset.len() == account.data_len() {
        return Err(MplCoreError::ExternalPluginAdapterNotFound.into());
    }

    //TODO: Bytemuck this.
    let mut header = PluginHeaderV1::load(account, asset.len())?;
    let mut plugin_registry = PluginRegistryV1::load(account, header.plugin_registry_offset)?;

    let result = find_external_plugin_adapter(&plugin_registry, plugin_key, account)?;

    if let (Some(index), _) = result {
        let registry_record = plugin_registry.external_registry.remove(index);
        let serialized_registry_record = borsh::to_vec(&registry_record)?;

        // Fetch the offset of the plugin to be removed.
        let plugin_offset = registry_record.offset;
        let plugin = ExternalPluginAdapter::load(account, plugin_offset)?;
        let serialized_plugin = borsh::to_vec(&plugin)?;
        let serialized_plugin_len = serialized_plugin
            .len()
            .checked_add(registry_record.data_len.unwrap_or(0))
            .ok_or(MplCoreError::NumericalOverflow)?;

        // Get the offset of the plugin after the one being removed.
        let next_plugin_offset = plugin_offset
            .checked_add(serialized_plugin_len)
            .ok_or(MplCoreError::NumericalOverflow)?;

        // Calculate the new size of the account.
        let new_size = account
            .data_len()
            .checked_sub(serialized_registry_record.len())
            .ok_or(MplCoreError::NumericalOverflow)?
            .checked_sub(serialized_plugin_len)
            .ok_or(MplCoreError::NumericalOverflow)?;

        let new_registry_offset = header
            .plugin_registry_offset
            .checked_sub(serialized_plugin_len)
            .ok_or(MplCoreError::NumericalOverflow)?;

        let data_to_move = header
            .plugin_registry_offset
            .checked_sub(next_plugin_offset)
            .ok_or(MplCoreError::NumericalOverflow)?;

        // SAFETY: `borrow_mut` will always return a valid pointer.
        // plugin_offset is derived from next_plugin_offset and size_diff using
        // checked arithmetic, so it will always be less than or equal to account.data_len().
        // This will fail and revert state if there is a memory violation.
        unsafe {
            let base = account.data.borrow_mut().as_mut_ptr();
            sol_memmove(
                base.add(plugin_offset),
                base.add(next_plugin_offset),
                data_to_move,
            );
        }

        header.plugin_registry_offset = new_registry_offset;
        header.save(account, asset.len())?;

        // Move offsets for existing registry records.
        plugin_registry.bump_offsets(plugin_offset, -(serialized_plugin_len as isize))?;

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

    let old_authority_bytes = borsh::to_vec(&registry_record.authority)?;
    let new_authority_bytes = borsh::to_vec(new_authority)?;
    let size_diff = (new_authority_bytes.len() as isize)
        .checked_sub(old_authority_bytes.len() as isize)
        .ok_or(MplCoreError::NumericalOverflow)?;

    registry_record.authority = *new_authority;

    if size_diff != 0 {
        let new_size = (account.data_len() as isize)
            .checked_add(size_diff)
            .ok_or(MplCoreError::NumericalOverflow)?;
        let new_size: usize = new_size
            .try_into()
            .map_err(|_| MplCoreError::NumericalOverflow)?;
        resize_or_reallocate_account(account, payer, system_program, new_size)?;
    }

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

    let old_authority_bytes = borsh::to_vec(&registry_record.authority)?;
    registry_record.authority = registry_record.plugin_type.manager();
    let new_authority_bytes = borsh::to_vec(&registry_record.authority)?;

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
            | ExternalPluginAdapterKey::Oracle(address)
            | ExternalPluginAdapterKey::LinkedLifecycleHook(address) => {
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
            ExternalPluginAdapterKey::AppData(authority)
            | ExternalPluginAdapterKey::LinkedAppData(authority) => {
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
            ExternalPluginAdapterKey::DataSection(linked_data_key) => {
                let linked_data_key_offset = record_ref
                    .offset
                    .checked_add(1)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                linked_data_key
                    == &match LinkedDataKey::deserialize(
                        &mut &account.data.borrow()[linked_data_key_offset..],
                    ) {
                        Ok(linked_data_key) => linked_data_key,
                        Err(_) => return Err(MplCoreError::DeserializationError.into()),
                    }
            }
            // AgentIdentity is a unit variant - only one per asset, so type match is sufficient.
            ExternalPluginAdapterKey::AgentIdentity => true,
        })
    {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod validation_tests {
    use {
        super::*,
        crate::plugins::{
            test_ctx::{default_ctx, FakeAccount},
            AgentIdentity, AppData, DataSection, ExternalPluginAdapterSchema, LifecycleHook,
            LinkedAppData, LinkedDataKey, LinkedLifecycleHook, PluginValidation, ValidationResult,
        },
    };

    fn core_err<T>(result: Result<T, ProgramError>) -> MplCoreError {
        match result {
            Err(ProgramError::Custom(code)) => {
                num_traits::FromPrimitive::from_u32(code).expect("an MplCoreError code")
            }
            Err(other) => panic!("expected a custom program error, got {other:?}"),
            Ok(_) => panic!("expected an error"),
        }
    }

    fn check(
        event: HookableLifecycleEvent,
        flags: u32,
    ) -> (HookableLifecycleEvent, ExternalCheckResult) {
        (event, ExternalCheckResult { flags })
    }

    #[test]
    fn validate_lifecycle_checks_rules() {
        // Empty is always refused.
        for can_reject_only in [true, false] {
            assert_eq!(
                core_err(validate_lifecycle_checks(&[], can_reject_only)),
                MplCoreError::RequiresLifecycleCheck
            );
        }

        // Duplicate events are refused before the flag check.
        assert_eq!(
            core_err(validate_lifecycle_checks(
                &[
                    check(HookableLifecycleEvent::Transfer, 0x4),
                    check(HookableLifecycleEvent::Transfer, 0x4),
                ],
                true,
            )),
            MplCoreError::DuplicateLifecycleChecks
        );

        // Reject-only adapters accept exactly `0x4`.
        assert!(validate_lifecycle_checks(
            &[
                check(HookableLifecycleEvent::Transfer, 0x4),
                check(HookableLifecycleEvent::Burn, 0x4),
            ],
            true,
        )
        .is_ok());
        for flags in [0x0, 0x1, 0x2, 0x3, 0x5, 0x6, 0x7] {
            assert_eq!(
                core_err(validate_lifecycle_checks(
                    &[check(HookableLifecycleEvent::Transfer, flags)],
                    true,
                )),
                MplCoreError::OracleCanRejectOnly,
                "flags {flags:#x} must be refused for a reject-only adapter"
            );
        }

        // Everything else accepts any flags, including bits outside the three
        // that `ExternalCheckResultBits` decodes (roadmap section 11,
        // finding 6).
        for flags in [0x0, 0x1, 0x7, 0xFFFF_FFF8] {
            assert!(
                validate_lifecycle_checks(
                    &[check(HookableLifecycleEvent::Transfer, flags)],
                    false,
                )
                .is_ok(),
                "flags {flags:#x} are accepted for a non-Oracle adapter"
            );
        }
    }

    /// Validators that no instruction can route to today (roadmap section 11,
    /// class D), kept working so a future change is a deliberate one.
    #[test]
    fn blocked_and_unrouted_adapter_validators() {
        let self_authority = Authority::UpdateAuthority;
        let data_authority = Authority::Address {
            address: Pubkey::new_unique(),
        };

        let mut signer = FakeAccount::wallet();
        let mut asset = FakeAccount::wallet();
        let signer_info = signer.info();
        let asset_info = asset.info();

        // `AppData` records are stored with `lifecycle_checks: None`, so
        // `check_adapter_registry` never selects them for a transfer.
        let app_data = AppData {
            data_authority,
            schema: ExternalPluginAdapterSchema::Binary,
        };
        let ctx = default_ctx(&[], &signer_info, &self_authority);
        assert_eq!(
            app_data.validate_transfer(&ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            app_data.validate_add_external_plugin_adapter(&ctx).unwrap(),
            ValidationResult::Pass
        );

        // `LinkedAppData::validate_create` is never called because
        // `check_create` returns `none()` for it. Both branches still work.
        let linked = LinkedAppData {
            data_authority,
            schema: ExternalPluginAdapterSchema::Binary,
        };
        assert_eq!(
            linked.validate_create(&ctx).unwrap(),
            ValidationResult::Pass,
            "on a collection (no asset) the create is allowed"
        );
        let mut ctx_with_asset = default_ctx(&[], &signer_info, &self_authority);
        ctx_with_asset.asset_info = Some(&asset_info);
        assert_eq!(
            linked.validate_create(&ctx_with_asset).unwrap(),
            ValidationResult::Rejected,
            "on an asset it is rejected"
        );

        // The hook adapters are blocked at initialization, so their constant
        // validators never run on-chain.
        let hook = LifecycleHook {
            hooked_program: Pubkey::new_unique(),
            extra_accounts: None,
            data_authority: None,
            schema: ExternalPluginAdapterSchema::Binary,
        };
        assert_eq!(
            hook.validate_transfer(&ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            hook.validate_add_external_plugin_adapter(&ctx).unwrap(),
            ValidationResult::Pass
        );
        let linked_hook = LinkedLifecycleHook {
            hooked_program: Pubkey::new_unique(),
            extra_accounts: None,
            data_authority: None,
            schema: ExternalPluginAdapterSchema::Binary,
        };
        assert_eq!(
            linked_hook.validate_transfer(&ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            linked_hook
                .validate_add_external_plugin_adapter(&ctx)
                .unwrap(),
            ValidationResult::Pass
        );

        // A `DataSection` overrides nothing at all.
        let section = DataSection {
            parent_key: LinkedDataKey::LinkedAppData(data_authority),
            schema: ExternalPluginAdapterSchema::Binary,
        };
        assert_eq!(
            section.validate_transfer(&ctx).unwrap(),
            ValidationResult::Pass
        );

        // `AgentIdentity` rejects a collection target on both the create and
        // the add path; the add path's arm is unreachable because
        // `add_external_plugin_adapter.rs` refuses collections first.
        let agent = AgentIdentity {
            uri: "https://example.com/agent.json".to_string(),
        };
        assert_eq!(
            agent.validate_create(&ctx).unwrap(),
            ValidationResult::Rejected
        );
        assert_eq!(
            agent.validate_add_external_plugin_adapter(&ctx).unwrap(),
            ValidationResult::Rejected
        );
        // Its lifecycle hooks are constant abstains.
        assert_eq!(
            agent.validate_transfer(&ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(agent.validate_burn(&ctx).unwrap(), ValidationResult::Pass);
        assert_eq!(agent.validate_update(&ctx).unwrap(), ValidationResult::Pass);
    }
}

#[cfg(test)]
mod account_reader_tests {
    use {
        super::*,
        crate::{
            plugins::{
                test_ctx::{asset_bytes, collection_bytes, FakeAccount},
                Attributes, FreezeDelegate, UpdateDelegate,
            },
            state::CollectionV1,
        },
    };

    fn core_err<T>(result: Result<T, ProgramError>) -> MplCoreError {
        match result {
            Err(ProgramError::Custom(code)) => {
                num_traits::FromPrimitive::from_u32(code).expect("an MplCoreError code")
            }
            Err(other) => panic!("expected a custom program error, got {other:?}"),
            Ok(_) => panic!("expected an error"),
        }
    }

    fn plugins() -> Vec<(Plugin, Authority)> {
        vec![
            (
                Plugin::Attributes(Attributes {
                    attribute_list: vec![],
                }),
                Authority::UpdateAuthority,
            ),
            (
                Plugin::UpdateDelegate(UpdateDelegate {
                    additional_delegates: vec![],
                }),
                Authority::UpdateAuthority,
            ),
        ]
    }

    /// `assert_plugins_initialized` and `fetch_plugins` have no callers in the
    /// program (roadmap section 11, class D); these keep them honest.
    #[test]
    fn assert_plugins_initialized_and_fetch_plugins() {
        let owner = Pubkey::new_unique();

        let mut bare = FakeAccount::with_data(asset_bytes(owner, &[]));
        let bare_info = bare.info();
        assert_eq!(
            core_err(assert_plugins_initialized(&bare_info)),
            MplCoreError::PluginsNotInitialized
        );
        assert_eq!(
            core_err(fetch_plugins(&bare_info)),
            MplCoreError::PluginNotFound
        );

        let mut with_plugins = FakeAccount::with_data(asset_bytes(owner, &plugins()));
        let info = with_plugins.info();
        assert!(assert_plugins_initialized(&info).is_ok());
        let records = fetch_plugins(&info).unwrap();
        assert_eq!(
            records
                .iter()
                .map(|record| record.plugin_type)
                .collect::<Vec<_>>(),
            vec![PluginType::Attributes, PluginType::UpdateDelegate]
        );
    }

    #[test]
    fn fetch_plugin_reads_the_inner_value_and_its_authority() {
        let owner = Pubkey::new_unique();
        let mut account = FakeAccount::with_data(collection_bytes(owner, &plugins()));
        let info = account.info();

        let (authority, delegate, offset) =
            fetch_plugin::<CollectionV1, UpdateDelegate>(&info, PluginType::UpdateDelegate)
                .unwrap();
        assert_eq!(authority, Authority::UpdateAuthority);
        assert_eq!(delegate.additional_delegates, Vec::<Pubkey>::new());
        assert!(offset > 0);

        // A type that is not in the registry.
        assert_eq!(
            core_err(fetch_plugin::<CollectionV1, FreezeDelegate>(
                &info,
                PluginType::FreezeDelegate
            )),
            MplCoreError::PluginNotFound
        );

        // A bare account has no header at all.
        let mut bare = FakeAccount::with_data(collection_bytes(owner, &[]));
        let bare_info = bare.info();
        assert_eq!(
            core_err(fetch_plugin::<CollectionV1, UpdateDelegate>(
                &bare_info,
                PluginType::UpdateDelegate
            )),
            MplCoreError::PluginNotFound
        );
        assert_eq!(
            core_err(list_plugins::<CollectionV1>(&bare_info)),
            MplCoreError::PluginNotFound
        );

        assert_eq!(
            list_plugins::<CollectionV1>(&info).unwrap(),
            vec![PluginType::Attributes, PluginType::UpdateDelegate]
        );
    }
}
