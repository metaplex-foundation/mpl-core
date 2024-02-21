use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
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
