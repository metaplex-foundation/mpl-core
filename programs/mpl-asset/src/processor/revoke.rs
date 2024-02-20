use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    instruction::accounts::RevokeAccounts,
    plugins::{add_plugin_or_authority, create_meta_idempotent, Delegate, Plugin},
    state::Authority,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct RevokeArgs {}

//TODO: Remove the plugin auth.
pub(crate) fn revoke<'a>(accounts: &'a [AccountInfo<'a>], _args: RevokeArgs) -> ProgramResult {
    let ctx = RevokeAccounts::context(accounts)?;

    assert_signer(ctx.accounts.owner)?;
    let payer = match ctx.accounts.payer {
        Some(payer) => {
            assert_signer(payer)?;
            payer
        }
        None => ctx.accounts.owner,
    };

    create_meta_idempotent(
        ctx.accounts.asset_address,
        ctx.accounts.owner,
        ctx.accounts.system_program,
    )?;

    let plugin = Plugin::Delegate(Delegate::new());

    add_plugin_or_authority(
        &plugin,
        Authority::Pubkey {
            address: *ctx.accounts.delegate.key,
        },
        ctx.accounts.asset_address,
        payer,
        ctx.accounts.system_program,
    )
}
