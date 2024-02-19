use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::{
    error::MplAssetError,
    state::{Asset, Authority, Key, SolanaAccount},
};

pub fn load_key(account: &AccountInfo, offset: usize) -> Result<Key, ProgramError> {
    let key = Key::from_u8((*account.data).borrow()[offset])
        .ok_or(MplAssetError::DeserializationError)?;

    Ok(key)
}

pub fn assert_authority(
    account: &AccountInfo,
    authority: &AccountInfo,
    authorities: &[Authority],
) -> ProgramResult {
    let asset = Asset::load(account, 0)?;
    for auth_iter in authorities {
        match auth_iter {
            Authority::Owner => {
                if &asset.owner == authority.key {
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
