use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{UpdateCollectionPluginAccounts, UpdatePluginAccounts},
    plugins::{Plugin, PluginType, ValidationResult},
    state::{Asset, Collection, Key},
    utils::{fetch_core_data, load_key},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdatePluginArgs {
    pub plugin: Plugin,
}

pub(crate) fn update_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdatePluginArgs,
) -> ProgramResult {
    // Accounts.
    let ctx = UpdatePluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
    }

    if let Key::HashedAsset = load_key(ctx.accounts.asset, 0)? {
        msg!("Error: Update plugin for compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    let (mut asset, _, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;
    let plugin_registry = plugin_registry.ok_or(MplCoreError::PluginsNotInitialized)?;

    let plugin_type: PluginType = (&args.plugin).into();
    let registry_record = plugin_registry
        .registry
        .iter()
        .find(|record| record.plugin_type == plugin_type)
        .ok_or(MplCoreError::PluginNotFound)?;

    let result = Plugin::validate_update_plugin(
        &Plugin::load(ctx.accounts.asset, registry_record.offset)?,
        &asset,
        ctx.accounts.authority,
        None,
        &registry_record.authority,
        None,
    )?;
    if result == ValidationResult::Rejected {
        return Err(MplCoreError::InvalidAuthority.into());
    } else if result == ValidationResult::Approved {
        //TODO: Handle plugins that are dynamically sized.
        args.plugin
            .save(ctx.accounts.asset, registry_record.offset)?;
    } else {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    process_update_plugin()
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateCollectionPluginArgs {
    pub plugin: Plugin,
}

pub(crate) fn update_collection_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdateCollectionPluginArgs,
) -> ProgramResult {
    // Accounts.
    let ctx = UpdateCollectionPluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
    }

    let (collection, _, plugin_registry) = fetch_core_data::<Collection>(ctx.accounts.collection)?;
    let plugin_registry = plugin_registry.ok_or(MplCoreError::PluginsNotInitialized)?;

    let plugin_type: PluginType = (&args.plugin).into();
    let registry_record = plugin_registry
        .registry
        .iter()
        .find(|record| record.plugin_type == plugin_type)
        .ok_or(MplCoreError::PluginNotFound)?;

    let result = Plugin::validate_update_plugin(
        &Plugin::load(ctx.accounts.collection, registry_record.offset)?,
        &collection,
        ctx.accounts.authority,
        None,
        &registry_record.authority,
        None,
    )?;
    if result == ValidationResult::Rejected {
        return Err(MplCoreError::InvalidAuthority.into());
    } else if result == ValidationResult::Approved {
        //TODO: Handle plugins that are dynamically sized.
        args.plugin
            .save(ctx.accounts.collection, registry_record.offset)?;
    } else {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    process_update_plugin()
}

//TODO
fn process_update_plugin() -> ProgramResult {
    Ok(())
}
