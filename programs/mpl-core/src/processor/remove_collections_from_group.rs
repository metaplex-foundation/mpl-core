use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use super::groups_plugin_utils::save_updated_groups_plugin;
use crate::{
    error::MplCoreError,
    instruction::accounts::{Context, RemoveCollectionsFromGroupV1Accounts},
    plugins::{create_meta_idempotent, Plugin, PluginType},
    state::{CollectionV1, GroupV1, SolanaAccount},
    utils::{
        is_valid_collection_authority, is_valid_group_authority, resolve_authority, save_flat_group,
    },
};

/// Arguments for the `RemoveCollectionsFromGroupV1` instruction.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveCollectionsFromGroupV1Args {
    /// The list of collections to remove from the group.
    pub(crate) collections: Vec<Pubkey>,
}

/// Processor for the `RemoveCollectionsFromGroupV1` instruction.
#[allow(clippy::too_many_arguments)]
pub(crate) fn remove_collections_from_group_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveCollectionsFromGroupV1Args,
) -> ProgramResult {
    // Expected account layout:
    //   0. [writable] Group account
    //   1. [writable, signer] Payer account (also default authority)
    //   2. [signer] Optional authority (update auth/delegate)
    //   3. [] System program
    //   4..N [writable] Collection accounts, one for each pubkey in args.collections
    let ctx: Context<RemoveCollectionsFromGroupV1Accounts> =
        RemoveCollectionsFromGroupV1Accounts::context(accounts)?;
    let group_info = ctx.accounts.group;
    let payer_info = ctx.accounts.payer;
    let authority_info_opt = ctx.accounts.authority;
    let system_program_info = ctx.accounts.system_program;
    let collection_accounts = ctx.remaining_accounts;

    // Basic guards
    assert_signer(payer_info)?;
    let authority_info = resolve_authority(payer_info, authority_info_opt)?;

    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if !group_info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    if collection_accounts.len() != args.collections.len() {
        msg!(
            "Error: Number of collection accounts ({}) does not match number of pubkeys in args ({}).",
            collection_accounts.len(),
            args.collections.len()
        );
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Deserialize group.
    let mut group = GroupV1::load(group_info, 0)?;

    // Authority check: must be the group's update authority or delegate.
    if !is_valid_group_authority(group_info, authority_info)? {
        msg!("Error: Invalid authority for group account");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    for (i, collection_info) in collection_accounts.iter().enumerate() {
        if collection_info.key != &args.collections[i] {
            msg!(
                "Error: Collection account at position {} does not match provided pubkey list",
                i
            );
            return Err(MplCoreError::IncorrectAccount.into());
        }

        if !collection_info.is_writable {
            msg!("Error: Collection account must be writable");
            return Err(ProgramError::InvalidAccountData);
        }

        let _collection_core = CollectionV1::load(collection_info, 0)?;

        if !is_valid_collection_authority(collection_info, authority_info)? {
            msg!("Error: Signer is not collection update authority/delegate");
            return Err(MplCoreError::InvalidAuthority.into());
        }

        // Remove from group.collections if present.
        if let Some(pos) = group
            .collections
            .iter()
            .position(|pk| pk == collection_info.key)
        {
            group.collections.remove(pos);
        } else {
            msg!("Error: Collection is not a child of the provided group");
            return Err(MplCoreError::IncorrectAccount.into());
        }

        // Remove group from collection's Groups plugin.
        process_collection_groups_plugin_remove(
            collection_info,
            *group_info.key,
            payer_info,
            system_program_info,
        )?;
    }

    save_flat_group(group_info, &group, payer_info, system_program_info)?;

    Ok(())
}

fn process_collection_groups_plugin_remove<'a>(
    collection_info: &AccountInfo<'a>,
    parent_group: Pubkey,
    payer_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
) -> ProgramResult {
    let (_collection_core, header_offset, mut plugin_header, mut plugin_registry) =
        create_meta_idempotent::<CollectionV1>(collection_info, payer_info, system_program_info)?;

    let record_index_opt = plugin_registry
        .registry
        .iter()
        .position(|r| r.plugin_type == PluginType::Groups);

    if let Some(index) = record_index_opt {
        let record = plugin_registry.registry[index].clone();
        let mut plugin = Plugin::load(collection_info, record.offset)?;
        if let Plugin::Groups(inner) = &mut plugin {
            if let Some(pos) = inner.groups.iter().position(|pk| pk == &parent_group) {
                inner.groups.remove(pos);
            } else {
                return Ok(());
            }
        } else {
            return Err(MplCoreError::InvalidPlugin.into());
        }

        save_updated_groups_plugin(
            collection_info,
            payer_info,
            system_program_info,
            &plugin,
            &record,
            &mut plugin_header,
            &mut plugin_registry,
            header_offset,
        )?;
    }

    Ok(())
}
