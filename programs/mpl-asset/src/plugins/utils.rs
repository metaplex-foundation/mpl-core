use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::resize_or_reallocate_account_raw;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_memory::sol_memcpy,
};

use crate::{
    error::MplAssetError,
    state::{Asset, Authority, DataBlob, Key, PluginHeader, SolanaAccount},
    utils::assert_authority,
};

use super::{Plugin, PluginRegistry, PluginType, RegistryData, RegistryRecord};

/// Create plugin header and registry if it doesn't exist
pub fn create_meta_idempotent<'a>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let asset = {
        let mut bytes: &[u8] = &(*account.data).borrow();
        Asset::deserialize(&mut bytes)?
    };

    // Check if the plugin header and registry exist.
    if asset.get_size() == account.data_len() {
        // They don't exist, so create them.
        let header = PluginHeader {
            key: Key::PluginHeader,
            plugin_registry_offset: asset.get_size() + PluginHeader::get_initial_size(),
        };
        let registry = PluginRegistry {
            key: Key::PluginRegistry,
            registry: vec![],
            external_plugins: vec![],
        };

        resize_or_reallocate_account_raw(
            account,
            payer,
            system_program,
            header.plugin_registry_offset + PluginRegistry::get_initial_size(),
        )?;

        header.save(account, asset.get_size())?;
        registry.save(account, header.plugin_registry_offset)?;
    }

    Ok(())
}

pub fn assert_plugins_initialized(account: &AccountInfo) -> ProgramResult {
    let mut bytes: &[u8] = &(*account.data).borrow();
    let asset = Asset::deserialize(&mut bytes).unwrap();

    if asset.get_size() == account.data_len() {
        return Err(MplAssetError::PluginsNotInitialized.into());
    }

    Ok(())
}

/// Fetch the plugin from the registry.
pub fn fetch_plugin(
    account: &AccountInfo,
    plugin_type: PluginType,
) -> Result<(Vec<Authority>, Plugin, usize), ProgramError> {
    let mut bytes: &[u8] = &(*account.data).borrow();
    let asset = Asset::deserialize(&mut bytes)?;

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::load(account, header.plugin_registry_offset)?;

    solana_program::msg!("{:?}", registry);

    // Find the plugin in the registry.
    let plugin_data = registry
        .iter()
        .find(
            |RegistryRecord {
                 plugin_type: plugin_type_iter,
                 data: _,
             }| *plugin_type_iter == plugin_type,
        )
        .map(
            |RegistryRecord {
                 plugin_type: _,
                 data,
             }| data,
        )
        .ok_or(MplAssetError::PluginNotFound)?;

    solana_program::msg!("Deserialize plugin at offset");
    // Deserialize the plugin.
    let plugin = Plugin::deserialize(&mut &(*account.data).borrow()[plugin_data.offset..])?;

    solana_program::msg!("Return plugin");
    // Return the plugin and its authorities.
    Ok((plugin_data.authorities.clone(), plugin, plugin_data.offset))
}

/// Create plugin header and registry if it doesn't exist
pub fn list_plugins(account: &AccountInfo) -> Result<Vec<PluginType>, ProgramError> {
    let mut bytes: &[u8] = &(*account.data).borrow();
    let asset = Asset::deserialize(&mut bytes)?;

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::load(account, header.plugin_registry_offset)?;

    Ok(registry
        .iter()
        .map(|registry_record| registry_record.plugin_type)
        .collect())
}

/// Add a plugin into the registry
// pub fn add_plugin_or_authority<'a>(
//     plugin: &Plugin,
//     authority: Authority,
//     account: &AccountInfo<'a>,
//     payer: &AccountInfo<'a>,
//     system_program: &AccountInfo<'a>,
// ) -> ProgramResult {
//     let asset = {
//         let mut bytes: &[u8] = &(*account.data).borrow();
//         Asset::deserialize(&mut bytes)?
//     };

//     //TODO: Bytemuck this.
//     let mut header = PluginHeader::load(account, asset.get_size())?;
//     let mut plugin_registry = PluginRegistry::load(account, header.plugin_registry_offset)?;

//     let plugin_type = plugin.into();
//     let plugin_data = plugin.try_to_vec()?;
//     let plugin_size = plugin_data.len();
//     let authority_bytes = authority.try_to_vec()?;

//     if let Some(RegistryRecord {
//         plugin_type: _,
//         data: registry_data,
//     }) = plugin_registry.registry.iter_mut().find(
//         |RegistryRecord {
//              plugin_type: type_iter,
//              data: _,
//          }| type_iter == &plugin_type,
//     ) {
//         registry_data.authorities.push(authority.clone());

//         let new_size = account
//             .data_len()
//             .checked_add(authority_bytes.len())
//             .ok_or(MplAssetError::NumericalOverflow)?;
//         resize_or_reallocate_account_raw(account, payer, system_program, new_size)?;

//         plugin_registry.save(account, header.plugin_registry_offset)?;
//     } else {
//         let old_registry_offset = header.plugin_registry_offset;
//         let registry_data = RegistryData {
//             offset: old_registry_offset,
//             authorities: vec![authority],
//         };

//         //TODO: There's probably a better way to get the size without having to serialize the registry data.
//         let size_increase = plugin_size
//             .checked_add(Key::get_initial_size())
//             .ok_or(MplAssetError::NumericalOverflow)?
//             .checked_add(registry_data.clone().try_to_vec()?.len())
//             .ok_or(MplAssetError::NumericalOverflow)?;

//         let new_registry_offset = header
//             .plugin_registry_offset
//             .checked_add(plugin_size)
//             .ok_or(MplAssetError::NumericalOverflow)?;

//         header.plugin_registry_offset = new_registry_offset;

//         plugin_registry.registry.push(RegistryRecord {
//             plugin_type,
//             data: registry_data.clone(),
//         });

//         let new_size = account
//             .data_len()
//             .checked_add(size_increase)
//             .ok_or(MplAssetError::NumericalOverflow)?;

//         resize_or_reallocate_account_raw(account, payer, system_program, new_size)?;

//         header.save(account, asset.get_size())?;
//         plugin.save(account, old_registry_offset)?;
//         plugin_registry.save(account, new_registry_offset)?;
//     }

//     Ok(())
// }

/// Add a plugin to the registry and initialize it.
pub fn initialize_plugin<'a>(
    plugin: &Plugin,
    authority: &Authority,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let asset = {
        let mut bytes: &[u8] = &(*account.data).borrow();
        Asset::deserialize(&mut bytes)?
    };

    //TODO: Bytemuck this.
    let mut header = PluginHeader::load(account, asset.get_size())?;
    let mut plugin_registry = PluginRegistry::load(account, header.plugin_registry_offset)?;

    let plugin_type = plugin.into();
    let plugin_data = plugin.try_to_vec()?;
    let plugin_size = plugin_data.len();
    // let authority_bytes = authority.try_to_vec()?;

    if let Some(RegistryRecord {
        plugin_type: _,
        data: _,
    }) = plugin_registry.registry.iter_mut().find(
        |RegistryRecord {
             plugin_type: type_iter,
             data: _,
         }| type_iter == &plugin_type,
    ) {
        return Err(MplAssetError::PluginAlreadyExists.into());
    }

    let old_registry_offset = header.plugin_registry_offset;
    let registry_data = RegistryData {
        offset: old_registry_offset,
        authorities: vec![authority.clone()],
    };
    let size_increase = plugin_size
        .checked_add(Key::get_initial_size())
        .ok_or(MplAssetError::NumericalOverflow)?
        .checked_add(registry_data.clone().try_to_vec()?.len())
        .ok_or(MplAssetError::NumericalOverflow)?;
    let new_registry_offset = header
        .plugin_registry_offset
        .checked_add(plugin_size)
        .ok_or(MplAssetError::NumericalOverflow)?;
    header.plugin_registry_offset = new_registry_offset;
    plugin_registry.registry.push(RegistryRecord {
        plugin_type,
        data: registry_data.clone(),
    });
    let new_size = account
        .data_len()
        .checked_add(size_increase)
        .ok_or(MplAssetError::NumericalOverflow)?;
    resize_or_reallocate_account_raw(account, payer, system_program, new_size)?;
    header.save(account, asset.get_size())?;
    plugin.save(account, old_registry_offset)?;
    plugin_registry.save(account, new_registry_offset)?;

    Ok(())
}

/// Remoc a plugin from the registry and delete it.
pub fn delete_plugin<'a>(
    plugin_type: &PluginType,
    authority: &AccountInfo<'a>,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let asset = {
        let mut bytes: &[u8] = &(*account.data).borrow();
        Asset::deserialize(&mut bytes)?
    };

    //TODO: Bytemuck this.
    let mut header = PluginHeader::load(account, asset.get_size())?;
    let mut plugin_registry = PluginRegistry::load(account, header.plugin_registry_offset)?;

    if let Some(index) = plugin_registry.registry.iter_mut().position(
        |RegistryRecord {
             plugin_type: type_iter,
             data: _,
         }| type_iter == plugin_type,
    ) {
        let registry_record = plugin_registry.registry.swap_remove(index);
        let serialized_registry_record = registry_record.try_to_vec()?;

        assert_authority(&asset, authority, &registry_record.data.authorities)?;

        let plugin_offset = registry_record.data.offset;
        let plugin = Plugin::load(account, plugin_offset)?;
        let serialized_plugin = plugin.try_to_vec()?;

        let next_plugin_offset = plugin_offset
            .checked_add(serialized_plugin.len())
            .ok_or(MplAssetError::NumericalOverflow)?;
        solana_program::msg!("next_plugin_offset: {:?}", next_plugin_offset);

        let new_size = account
            .data_len()
            .checked_sub(serialized_registry_record.len())
            .ok_or(MplAssetError::NumericalOverflow)?
            .checked_sub(serialized_plugin.len())
            .ok_or(MplAssetError::NumericalOverflow)?;
        solana_program::msg!("new_size: {:?}", new_size);

        let new_offset = header
            .plugin_registry_offset
            .checked_sub(serialized_plugin.len())
            .ok_or(MplAssetError::NumericalOverflow)?;
        solana_program::msg!("new_offset: {:?}", new_offset);

        let data_to_move = header
            .plugin_registry_offset
            .checked_sub(new_offset)
            .ok_or(MplAssetError::NumericalOverflow)?;
        solana_program::msg!("data_to_move: {:?}", data_to_move);

        let src = account.data.borrow()[next_plugin_offset..].to_vec();
        solana_program::msg!("src: {:?}", src);
        sol_memcpy(
            &mut account.data.borrow_mut()[plugin_offset..],
            &src,
            data_to_move,
        );

        header.plugin_registry_offset = new_offset;
        header.save(account, asset.get_size())?;

        plugin_registry.save(account, new_offset)?;

        resize_or_reallocate_account_raw(account, payer, system_program, new_size)?;
    } else {
        return Err(MplAssetError::PluginNotFound.into());
    }

    Ok(())
}
