use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::{
    error::MplAssetError,
    instruction::accounts::FreezeAccounts,
    plugins::{fetch_plugin, Delegate, Plugin, PluginType},
    utils::assert_authority,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct FreezeArgs {}

pub(crate) fn freeze<'a>(accounts: &'a [AccountInfo<'a>], _args: FreezeArgs) -> ProgramResult {
    let ctx = FreezeAccounts::context(accounts)?;

    assert_signer(ctx.accounts.delegate)?;

    let (authorities, mut plugin, offset) =
        fetch_plugin(ctx.accounts.asset_address, PluginType::Delegate)?;

    assert_authority(
        ctx.accounts.asset_address,
        ctx.accounts.delegate,
        &authorities,
    )?;

    let delegate = match &mut plugin {
        Plugin::Delegate(delegate) => {
            delegate.frozen = true;
            Ok::<&Delegate, ProgramError>(delegate)
        }
        _ => Err(MplAssetError::InvalidPlugin.into()),
    }?;

    Plugin::Delegate(*delegate).save(ctx.accounts.asset_address, offset)?;

    Ok(())
}
