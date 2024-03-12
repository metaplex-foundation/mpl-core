use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    state::{Asset, Authority, CoreAsset, DataBlob, Key, SolanaAccount},
    utils::{assert_authority, resize_or_reallocate_account},
};

use super::{Plugin, PluginHeader, PluginRegistry, PluginType, RegistryRecord};

/// Create plugin header and registry if it doesn't exist
pub fn create_meta_idempotent<'a, T: SolanaAccount + DataBlob>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> Result<(T, PluginHeader, PluginRegistry), ProgramError> {
    let core = T::load(account, 0)?;
    let header_offset = core.get_size();

    // Check if the plugin header and registry exist.
    if header_offset == account.data_len() {
        // They don't exist, so create them.
        let header = PluginHeader {
            key: Key::PluginHeader,
            plugin_registry_offset: header_offset + PluginHeader::get_initial_size(),
        };
        let registry = PluginRegistry {
            key: Key::PluginRegistry,
            registry: vec![],
            external_plugins: vec![],
        };

        resize_or_reallocate_account(
            account,
            payer,
            system_program,
            header.plugin_registry_offset + PluginRegistry::get_initial_size(),
        )?;

        header.save(account, header_offset)?;
        registry.save(account, header.plugin_registry_offset)?;

        Ok((core, header, registry))
    } else {
        // They exist, so load them.
        let header = PluginHeader::load(account, header_offset)?;
        let registry = PluginRegistry::load(account, header.plugin_registry_offset)?;

        Ok((core, header, registry))
    }
}

/// Create plugin header and registry if it doesn't exist
pub fn create_plugin_meta<'a, T: SolanaAccount + DataBlob>(
    asset: T,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> Result<(PluginHeader, PluginRegistry), ProgramError> {
    let header_offset = asset.get_size();

    // They don't exist, so create them.
    let header = PluginHeader {
        key: Key::PluginHeader,
        plugin_registry_offset: header_offset + PluginHeader::get_initial_size(),
    };
    let registry = PluginRegistry {
        key: Key::PluginRegistry,
        registry: vec![],
        external_plugins: vec![],
    };

    resize_or_reallocate_account(
        account,
        payer,
        system_program,
        header.plugin_registry_offset + PluginRegistry::get_initial_size(),
    )?;

    header.save(account, header_offset)?;
    registry.save(account, header.plugin_registry_offset)?;

    Ok((header, registry))
}

/// Assert that the Plugin metadata has been initialized.
pub fn assert_plugins_initialized(account: &AccountInfo) -> ProgramResult {
    let mut bytes: &[u8] = &(*account.data).borrow();
    let asset = Asset::deserialize(&mut bytes)?;

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

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::load(account, header.plugin_registry_offset)?;

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

    // Return the plugin and its authorities.
    Ok((registry_record.authority, inner, registry_record.offset))
}

/// Fetch the plugin from the registry.
pub fn fetch_wrapped_plugin<T: DataBlob + SolanaAccount>(
    account: &AccountInfo,
    plugin_type: PluginType,
) -> Result<(Authority, Plugin), ProgramError> {
    let asset = T::load(account, 0)?;

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::load(account, header.plugin_registry_offset)?;

    // Find the plugin in the registry.
    let registry_record = registry
        .iter()
        .find(|record| record.plugin_type == plugin_type)
        .ok_or(MplCoreError::PluginNotFound)?;

    // Deserialize the plugin.
    let plugin = Plugin::deserialize(&mut &(*account.data).borrow()[registry_record.offset..])?;

    // Return the plugin and its authorities.
    Ok((registry_record.authority, plugin))
}

/// Fetch the plugin registry.
pub fn fetch_plugins(account: &AccountInfo) -> Result<Vec<RegistryRecord>, ProgramError> {
    let asset = Asset::load(account, 0)?;

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::load(account, header.plugin_registry_offset)?;

    Ok(registry)
}

/// Create plugin header and registry if it doesn't exist
pub fn list_plugins(account: &AccountInfo) -> Result<Vec<PluginType>, ProgramError> {
    let asset = Asset::load(account, 0)?;

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::load(account, header.plugin_registry_offset)?;

    Ok(registry
        .iter()
        .map(|registry_record| registry_record.plugin_type)
        .collect())
}

//TODO: Take in the header and registry so we don't have to reload it.
/// Add a plugin to the registry and initialize it.
pub fn initialize_plugin<'a, T: DataBlob + SolanaAccount>(
    plugin: &Plugin,
    authority: &Authority,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let core = T::load(account, 0)?;
    let header_offset = core.get_size();

    //TODO: Bytemuck this.
    let mut header = PluginHeader::load(account, header_offset)?;
    let mut plugin_registry = PluginRegistry::load(account, header.plugin_registry_offset)?;

    let plugin_type = plugin.into();
    let plugin_data = plugin.try_to_vec()?;
    let plugin_size = plugin_data.len();

    // You cannot add a duplicate plugin.
    if plugin_registry
        .registry
        .iter_mut()
        .any(|record| record.plugin_type == plugin_type)
    {
        return Err(MplCoreError::PluginAlreadyExists.into());
    }

    let old_registry_offset = header.plugin_registry_offset;

    let new_registry_record = RegistryRecord {
        plugin_type,
        offset: old_registry_offset,
        authority: *authority,
    };

    let size_increase = plugin_size
        .checked_add(new_registry_record.try_to_vec()?.len())
        .ok_or(MplCoreError::NumericalOverflow)?;

    let new_registry_offset = header
        .plugin_registry_offset
        .checked_add(plugin_size)
        .ok_or(MplCoreError::NumericalOverflow)?;

    header.plugin_registry_offset = new_registry_offset;

    plugin_registry.registry.push(new_registry_record);

    let new_size = account
        .data_len()
        .checked_add(size_increase)
        .ok_or(MplCoreError::NumericalOverflow)?;

    resize_or_reallocate_account(account, payer, system_program, new_size)?;
    header.save(account, header_offset)?;
    plugin.save(account, old_registry_offset)?;
    plugin_registry.save(account, new_registry_offset)?;

    Ok(())
}

/// Remove a plugin from the registry and delete it.
//TODO: Do the work to prevent deleting a plugin with other authorities.
pub fn delete_plugin<'a, T: DataBlob>(
    plugin_type: &PluginType,
    asset: &T,
    authority_type: &Authority,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    //TODO: Bytemuck this.
    let mut header = PluginHeader::load(account, asset.get_size())?;
    let mut plugin_registry = PluginRegistry::load(account, header.plugin_registry_offset)?;

    if let Some(index) = plugin_registry
        .registry
        .iter_mut()
        .position(|record| record.plugin_type == *plugin_type)
    {
        let registry_record = plugin_registry.registry.remove(index);
        let serialized_registry_record = registry_record.try_to_vec()?;

        // Only allow the default authority to delete the plugin.
        if authority_type != &registry_record.plugin_type.manager() {
            return Err(MplCoreError::InvalidAuthority.into());
        }

        let plugin_offset = registry_record.offset;
        let plugin = Plugin::load(account, plugin_offset)?;
        let serialized_plugin = plugin.try_to_vec()?;

        let next_plugin_offset = plugin_offset
            .checked_add(serialized_plugin.len())
            .ok_or(MplCoreError::NumericalOverflow)?;

        let new_size = account
            .data_len()
            .checked_sub(serialized_registry_record.len())
            .ok_or(MplCoreError::NumericalOverflow)?
            .checked_sub(serialized_plugin.len())
            .ok_or(MplCoreError::NumericalOverflow)?;

        let new_offset = header
            .plugin_registry_offset
            .checked_sub(serialized_plugin.len())
            .ok_or(MplCoreError::NumericalOverflow)?;

        let data_to_move = header
            .plugin_registry_offset
            .checked_sub(new_offset)
            .ok_or(MplCoreError::NumericalOverflow)?;

        //TODO: This is memory intensive, we should use memmove instead probably.
        let src = account.data.borrow()[next_plugin_offset..].to_vec();
        sol_memcpy(
            &mut account.data.borrow_mut()[plugin_offset..],
            &src,
            data_to_move,
        );

        header.plugin_registry_offset = new_offset;
        header.save(account, asset.get_size())?;

        plugin_registry.save(account, new_offset)?;

        resize_or_reallocate_account(account, payer, system_program, new_size)?;
    } else {
        return Err(MplCoreError::PluginNotFound.into());
    }

    Ok(())
}

/// Add an authority to a plugin.
//TODO: Prevent duplicate authorities.
#[allow(clippy::too_many_arguments)]
pub fn approve_authority_on_plugin<'a, T: CoreAsset>(
    plugin_type: &PluginType,
    authority: &AccountInfo<'a>,
    new_authority: &Authority,
    asset: &T,
    account: &AccountInfo<'a>,
    plugin_header: &PluginHeader,
    plugin_registry: &mut PluginRegistry,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let registry_record = &mut plugin_registry
        .registry
        .iter_mut()
        .find(|record| record.plugin_type == *plugin_type)
        .ok_or(MplCoreError::PluginNotFound)?;

    assert_authority(asset, authority, &registry_record.authority)?;

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
    authority_type: &Authority,
    account: &AccountInfo<'a>,
    plugin_header: &PluginHeader,
    plugin_registry: &mut PluginRegistry,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let registry_record = &mut plugin_registry
        .registry
        .iter_mut()
        .find(|record| record.plugin_type == *plugin_type)
        .ok_or(MplCoreError::PluginNotFound)?;

    solana_program::msg!("authority_type: {:?}", authority_type);
    solana_program::msg!("registry_record.authority: {:?}", registry_record.authority);

    // TODO inspect this logic
    if (*authority_type != registry_record.plugin_type.manager() &&
        // pubkey authorities can remove themselves if they are a signer
        authority_type != &registry_record.authority) ||
        // Unable to revoke a None authority
        registry_record.authority == Authority::None
    {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    let authority_bytes = registry_record.authority.try_to_vec()?;
    registry_record.authority = registry_record.plugin_type.manager();

    let new_size = account
        .data_len()
        .checked_sub(authority_bytes.len())
        .ok_or(MplCoreError::NumericalOverflow)?;
    resize_or_reallocate_account(account, payer, system_program, new_size)?;

    plugin_registry.save(account, plugin_header.plugin_registry_offset)?;

    Ok(())
}
