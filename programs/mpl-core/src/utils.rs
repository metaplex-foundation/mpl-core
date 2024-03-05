use num_traits::{FromPrimitive, ToPrimitive};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_error::ProgramError, rent::Rent, system_instruction, sysvar::Sysvar,
};

use crate::{
    error::MplCoreError,
    plugins::{PluginHeader, PluginRegistry},
    state::{
        Asset, Authority, Collection, Compressible, CompressionProof, CoreAsset, DataBlob,
        HashablePluginSchema, HashedAsset, HashedAssetSchema, Key, SolanaAccount,
    },
};

/// Load the one byte key from the account data at the given offset.
pub fn load_key(account: &AccountInfo, offset: usize) -> Result<Key, ProgramError> {
    let key =
        Key::from_u8((*account.data).borrow()[offset]).ok_or(MplCoreError::DeserializationError)?;

    Ok(key)
}

/// Assert that the account info address is in the authorities array.
pub fn assert_authority<T: CoreAsset>(
    asset: &T,
    authority: &AccountInfo,
    authorities: &[Authority],
) -> ProgramResult {
    solana_program::msg!("Update authority: {:?}", asset.update_authority());
    for auth_iter in authorities {
        solana_program::msg!("Check if {:?} matches {:?}", authority.key, auth_iter);
        match auth_iter {
            Authority::None => (),
            Authority::Owner => {
                if asset.owner() == authority.key {
                    return Ok(());
                }
            }
            Authority::UpdateAuthority => {
                if asset.update_authority().key() == *authority.key {
                    return Ok(());
                }
            }
            Authority::Pubkey { address } => {
                if authority.key == address {
                    return Ok(());
                }
            }
            Authority::Permanent { address } => {
                if authority.key == address {
                    return Ok(());
                }
            }
        }
    }

    Err(MplCoreError::InvalidAuthority.into())
}

/// Assert that the account info address is in the authorities array.
pub fn assert_collection_authority(
    asset: &Collection,
    authority: &AccountInfo,
    authorities: &[Authority],
) -> ProgramResult {
    for auth_iter in authorities {
        match auth_iter {
            Authority::None | Authority::Owner => (),
            Authority::UpdateAuthority => {
                if &asset.update_authority == authority.key {
                    return Ok(());
                }
            }
            Authority::Pubkey { address } => {
                if authority.key == address {
                    return Ok(());
                }
            }
            Authority::Permanent { address } => {
                if authority.key == address {
                    return Ok(());
                }
            }
        }
    }

    Err(MplCoreError::InvalidAuthority.into())
}

/// Fetch the core data from the account; asset, plugin header (if present), and plugin registry (if present).
pub fn fetch_core_data<T: DataBlob + SolanaAccount>(
    account: &AccountInfo,
) -> Result<(T, Option<PluginHeader>, Option<PluginRegistry>), ProgramError> {
    let asset = T::load(account, 0)?;

    if asset.get_size() != account.data_len() {
        let plugin_header = PluginHeader::load(account, asset.get_size())?;
        let plugin_registry = PluginRegistry::load(account, plugin_header.plugin_registry_offset)?;

        Ok((asset, Some(plugin_header), Some(plugin_registry)))
    } else {
        Ok((asset, None, None))
    }
}

/// Check that a compression proof results in same on-chain hash.
pub fn verify_proof(
    hashed_asset: &AccountInfo,
    compression_proof: &CompressionProof,
) -> Result<(Asset, Vec<HashablePluginSchema>), ProgramError> {
    let asset = Asset::from(compression_proof.clone());
    let asset_hash = asset.hash()?;

    let mut sorted_plugins = compression_proof.plugins.clone();
    sorted_plugins.sort_by(HashablePluginSchema::compare_indeces);

    let plugin_hashes = sorted_plugins
        .iter()
        .map(|plugin| plugin.hash())
        .collect::<Result<Vec<[u8; 32]>, ProgramError>>()?;

    let hashed_asset_schema = HashedAssetSchema {
        asset_hash,
        plugin_hashes,
    };

    let hashed_asset_schema_hash = hashed_asset_schema.hash()?;

    let current_account_hash = HashedAsset::load(hashed_asset, 0)?.hash;
    if hashed_asset_schema_hash != current_account_hash {
        return Err(MplCoreError::IncorrectAssetHash.into());
    }

    Ok((asset, sorted_plugins))
}

pub(crate) fn close_program_account<'a>(
    account_to_close_info: &AccountInfo<'a>,
    funds_dest_account_info: &AccountInfo<'a>,
) -> ProgramResult {
    let rent = Rent::get()?;

    let account_size = account_to_close_info.data_len();
    let account_rent = rent.minimum_balance(account_size);
    let one_byte_rent = rent.minimum_balance(1);

    let amount_to_return = account_rent
        .checked_sub(one_byte_rent)
        .ok_or(MplCoreError::NumericalOverflowError)?;

    // Transfer lamports from the account to the destination account.
    let dest_starting_lamports = funds_dest_account_info.lamports();
    **funds_dest_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(amount_to_return)
        .ok_or(MplCoreError::NumericalOverflowError)?;
    **account_to_close_info.try_borrow_mut_lamports()? -= amount_to_return;

    account_to_close_info.realloc(1, false)?;
    account_to_close_info.data.borrow_mut()[0] = Key::Uninitialized.to_u8().unwrap();

    Ok(())
}

/// Resize an account using realloc and retain any lamport overages, modified from Solana Cookbook
pub(crate) fn resize_or_reallocate_account<'a>(
    target_account: &AccountInfo<'a>,
    funding_account: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    new_size: usize,
) -> ProgramResult {
    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(new_size);
    let current_minimum_balance = rent.minimum_balance(target_account.data_len());
    let account_infos = &[
        funding_account.clone(),
        target_account.clone(),
        system_program.clone(),
    ];

    if new_minimum_balance >= current_minimum_balance {
        let lamports_diff = new_minimum_balance.saturating_sub(current_minimum_balance);
        invoke(
            &system_instruction::transfer(funding_account.key, target_account.key, lamports_diff),
            account_infos
        )?;
    } else {
        // return lamports to the compressor
        let lamports_diff = current_minimum_balance.saturating_sub(new_minimum_balance);

        **funding_account.try_borrow_mut_lamports()? += lamports_diff;
        **target_account.try_borrow_mut_lamports()? -= lamports_diff
    }

    target_account.realloc(new_size, false)?;

    Ok(())
}
