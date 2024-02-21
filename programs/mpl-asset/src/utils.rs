use num_traits::{FromPrimitive, ToPrimitive};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError, rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    error::MplAssetError,
    plugins::PluginRegistry,
    state::{Asset, Authority, DataBlob, Key, PluginHeader, SolanaAccount},
};

pub fn load_key(account: &AccountInfo, offset: usize) -> Result<Key, ProgramError> {
    let key = Key::from_u8((*account.data).borrow()[offset])
        .ok_or(MplAssetError::DeserializationError)?;

    Ok(key)
}

pub fn assert_authority(
    asset: &Asset,
    authority: &AccountInfo,
    authorities: &[Authority],
) -> ProgramResult {
    for auth_iter in authorities {
        match auth_iter {
            Authority::None => (),
            Authority::Owner => {
                if &asset.owner == authority.key {
                    return Ok(());
                }
            }
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
            Authority::SameAs { .. } => todo!(),
            Authority::Collection => todo!(),
        }
    }

    Err(MplAssetError::InvalidAuthority.into())
}

pub fn resolve_authority_to_default(asset: &Asset, authority: &AccountInfo) -> Authority {
    if authority.key == &asset.owner {
        Authority::Owner
    } else {
        Authority::UpdateAuthority
    }
}

pub fn fetch_core_data(
    account: &AccountInfo,
) -> Result<(Asset, Option<PluginHeader>, Option<PluginRegistry>), ProgramError> {
    let asset = Asset::load(account, 0)?;

    if asset.get_size() != account.data_len() {
        let plugin_header = PluginHeader::load(account, asset.get_size())?;
        let plugin_registry = PluginRegistry::load(account, plugin_header.plugin_registry_offset)?;

        Ok((asset, Some(plugin_header), Some(plugin_registry)))
    } else {
        Ok((asset, None, None))
    }
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
        .ok_or(MplAssetError::NumericalOverflowError)?;

    // Transfer lamports from the account to the destination account.
    let dest_starting_lamports = funds_dest_account_info.lamports();
    **funds_dest_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(amount_to_return)
        .ok_or(MplAssetError::NumericalOverflowError)?;
    **account_to_close_info.lamports.borrow_mut() = one_byte_rent;

    account_to_close_info.realloc(1, false)?;
    account_to_close_info.data.borrow_mut()[0] = Key::Uninitialized.to_u8().unwrap();

    Ok(())
}
