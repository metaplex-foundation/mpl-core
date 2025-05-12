use crate::error::MplCoreError;
use crate::state::{AssetGroupPluginV1, AssetV1, GroupV1, SolanaAccount};
use crate::utils::group::find_asset_group_plugin_address;
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memcpy,
    pubkey::Pubkey,
};

// Argument structs for asset/group membership instructions.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddAssetsToGroupArgs {
    /// Indices of asset accounts to add.
    pub asset_indices: Vec<u8>,
    /// Indices of authority signer accounts corresponding to each asset (owner
    /// or update authority delegate).
    pub authority_indices: Vec<u8>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveAssetsFromGroupArgs {
    /// Indices of asset accounts to remove.
    pub asset_indices: Vec<u8>,
    /// Indices of authority signer accounts corresponding to each asset.
    pub authority_indices: Vec<u8>,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn add_assets_to_group<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddAssetsToGroupArgs,
) -> ProgramResult {
    if args.asset_indices.len() != args.authority_indices.len() {
        return Err(MplCoreError::InvalidGroupOperation.into());
    }

    let authority_info = accounts
        .get(0)
        .ok_or(MplCoreError::MissingUpdateAuthority)?;
    let group_info = accounts.get(1).ok_or(MplCoreError::GroupNotFound)?;

    assert_signer(authority_info)?;

    let mut group = GroupV1::load(group_info, 0)?;
    group.validate_authority(authority_info.key)?;

    let mut assets_to_add: Vec<Pubkey> = Vec::new();

    for (i, asset_idx) in args.asset_indices.iter().enumerate() {
        let asset_info = accounts
            .get(*asset_idx as usize)
            .ok_or(MplCoreError::CollectionNotFound)?; // reuse error for asset not found
        let authority_idx = args.authority_indices[i] as usize;
        let asset_authority_info = accounts
            .get(authority_idx)
            .ok_or(MplCoreError::MissingUpdateAuthority)?;

        assert_signer(asset_authority_info)?;
        let asset = AssetV1::load(asset_info, 0)?;

        let mut authority_match = asset_authority_info.key == &asset.owner;
        if !authority_match {
            if let crate::state::UpdateAuthority::Address(addr) = asset.update_authority {
                if &addr == asset_authority_info.key {
                    authority_match = true;
                }
            }
        }
        if !authority_match {
            return Err(MplCoreError::InvalidAuthority.into());
        }

        if !group.assets.contains(asset_info.key) {
            assets_to_add.push(*asset_info.key);
        }
    }

    if !assets_to_add.is_empty() {
        group.assets.extend(assets_to_add.iter());
        let serialized = group.try_to_vec()?;
        sol_memcpy(
            &mut group_info.try_borrow_mut_data()?[0..serialized.len()],
            &serialized,
            serialized.len(),
        );

        for asset_pubkey in &assets_to_add {
            let (plugin_address, _bump) = find_asset_group_plugin_address(asset_pubkey);
            if let Some(plugin_info) = accounts.iter().find(|a| a.key == &plugin_address) {
                if !plugin_info.is_writable {
                    return Err(MplCoreError::IncorrectAccount.into());
                }

                if plugin_info.data_len() > 0 {
                    let mut plugin = AssetGroupPluginV1::load(plugin_info, 0)?;
                    if &plugin.asset != asset_pubkey {
                        return Err(MplCoreError::InvalidCollection.into());
                    }
                    if !plugin.groups.contains(group_info.key) {
                        plugin.groups.push(*group_info.key);
                        plugin.save(plugin_info, 0)?;
                    }
                } else {
                    if plugin_info.owner != &crate::ID {
                        return Err(MplCoreError::IncorrectAccount.into());
                    }
                    let mut plugin = AssetGroupPluginV1::new(*asset_pubkey);
                    plugin.groups.push(*group_info.key);
                    plugin.save(plugin_info, 0)?;
                }
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(crate) fn remove_assets_from_group<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveAssetsFromGroupArgs,
) -> ProgramResult {
    if args.asset_indices.len() != args.authority_indices.len() {
        return Err(MplCoreError::InvalidGroupOperation.into());
    }

    let authority_info = accounts
        .get(0)
        .ok_or(MplCoreError::MissingUpdateAuthority)?;
    let group_info = accounts.get(1).ok_or(MplCoreError::GroupNotFound)?;

    assert_signer(authority_info)?;
    let mut group = GroupV1::load(group_info, 0)?;
    group.validate_authority(authority_info.key)?;

    for (i, asset_idx) in args.asset_indices.iter().enumerate() {
        let asset_info = accounts
            .get(*asset_idx as usize)
            .ok_or(MplCoreError::CollectionNotFound)?; // reuse error
        let authority_idx = args.authority_indices[i] as usize;
        let asset_authority_info = accounts
            .get(authority_idx)
            .ok_or(MplCoreError::MissingUpdateAuthority)?;

        assert_signer(asset_authority_info)?;
        let asset = AssetV1::load(asset_info, 0)?;

        let mut authority_match = false;
        if asset_authority_info.key == &asset.owner {
            authority_match = true;
        }
        if !authority_match {
            if let crate::state::UpdateAuthority::Address(addr) = asset.update_authority {
                if &addr == asset_authority_info.key {
                    authority_match = true;
                }
            }
        }
        if !authority_match {
            return Err(MplCoreError::InvalidAuthority.into());
        }

        group.assets.retain(|k| k != asset_info.key);

        let (plugin_address, _bump) = find_asset_group_plugin_address(asset_info.key);
        if let Some(plugin_info) = accounts.iter().find(|a| a.key == &plugin_address) {
            if plugin_info.data_len() > 0 {
                let mut plugin = AssetGroupPluginV1::load(plugin_info, 0)?;
                plugin.groups.retain(|k| k != group_info.key);
                plugin.save(plugin_info, 0)?;
            }
        }
    }

    let serialized = group.try_to_vec()?;
    sol_memcpy(
        &mut group_info.try_borrow_mut_data()?[0..serialized.len()],
        &serialized,
        serialized.len(),
    );

    Ok(())
}
