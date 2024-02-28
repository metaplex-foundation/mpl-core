use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::AddAuthorityAccounts,
    plugins::{add_authority_to_plugin, PluginType},
    state::{Asset, Authority, CollectionData, Key},
    utils::{fetch_core_data, load_key},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct AddAuthorityArgs {
    pub plugin_type: PluginType,
    pub new_authority: Authority,
}

pub(crate) fn add_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddAuthorityArgs,
) -> ProgramResult {
    let ctx = AddAuthorityAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = match ctx.accounts.payer {
        Some(payer) => {
            assert_signer(payer)?;
            payer
        }
        None => ctx.accounts.authority,
    };

    match load_key(ctx.accounts.asset_address, 0)? {
        Key::Asset => {
            let (asset, plugin_header, plugin_registry) =
                fetch_core_data::<Asset>(ctx.accounts.asset_address)?;

            let plugin_header = match plugin_header {
                Some(header) => header,
                None => return Err(MplCoreError::PluginsNotInitialized.into()),
            };

            let mut plugin_registry = match plugin_registry {
                Some(registry) => registry,
                None => return Err(MplCoreError::PluginsNotInitialized.into()),
            };

            add_authority_to_plugin(
                &args.plugin_type,
                ctx.accounts.authority,
                &args.new_authority,
                &asset,
                ctx.accounts.asset_address,
                &plugin_header,
                &mut plugin_registry,
                payer,
                ctx.accounts.system_program,
            )?;
        }
        Key::Collection => {
            solana_program::msg!("Add authority to collection");
            let (collection, plugin_header, plugin_registry) =
                fetch_core_data::<CollectionData>(ctx.accounts.asset_address)?;

            let plugin_header = match plugin_header {
                Some(header) => header,
                None => return Err(MplCoreError::PluginsNotInitialized.into()),
            };

            let mut plugin_registry = match plugin_registry {
                Some(registry) => registry,
                None => return Err(MplCoreError::PluginsNotInitialized.into()),
            };

            add_authority_to_plugin(
                &args.plugin_type,
                ctx.accounts.authority,
                &args.new_authority,
                &collection,
                ctx.accounts.asset_address,
                &plugin_header,
                &mut plugin_registry,
                payer,
                ctx.accounts.system_program,
            )?;
        }
        _ => return Err(MplCoreError::IncorrectAccount.into()),
    }

    Ok(())
}
