use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_instruction, system_program,
    sysvar::Sysvar,
};

use super::add_assets_to_group::process_asset_groups_plugin_add;
use super::group_collection_plugin::process_collection_groups_plugin_add;
use crate::{
    error::MplCoreError,
    instruction::accounts::CreateGroupV1Accounts,
    state::{GroupV1, SolanaAccount, MAX_GROUP_NESTING_DEPTH, MAX_GROUP_VECTOR_SIZE},
    utils::{
        is_valid_asset_authority, is_valid_collection_authority, is_valid_group_authority,
        resolve_authority, save_flat_group,
    },
};

use crate::state::RelationshipKind;
use std::collections::HashSet;

/// Arguments for the `CreateGroupV1` instruction.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct CreateGroupV1Args {
    /// Human-readable display name for the group.
    pub(crate) name: String,
    /// URI pointing to off-chain JSON describing the group.
    pub(crate) uri: String,
    /// Relationships (collections, child groups, parent groups, assets) to link at creation.
    pub(crate) relationships: Vec<crate::state::RelationshipEntry>,
}

/// Processor for the `CreateGroupV1` instruction.
pub(crate) fn create_group_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: CreateGroupV1Args,
) -> ProgramResult {
    // Derive the typed account context from the raw slice.
    let ctx = CreateGroupV1Accounts::context(accounts)?;
    let rent = Rent::get()?;

    // Basic guards.
    assert_signer(ctx.accounts.group)?;
    assert_signer(ctx.accounts.payer)?;
    let authority_info = resolve_authority(ctx.accounts.payer, ctx.accounts.update_authority)?;

    // Ensure the canonical system program is provided.
    if *ctx.accounts.system_program.key != system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if !ctx.accounts.group.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    // --- PREP INPUT VECTORS -------------------------------------------------
    let CreateGroupV1Args {
        name,
        uri,
        relationships,
    } = args;

    let mut collections_vec: Vec<Pubkey> = Vec::new();
    let mut child_groups_vec: Vec<Pubkey> = Vec::new();
    let mut parent_groups_vec: Vec<Pubkey> = Vec::new();
    let mut assets_vec: Vec<Pubkey> = Vec::new();

    // Track all relationship keys we have already seen to prevent duplicates across
    // categories (e.g., the same account referenced twice or in two different
    // categories).  If a duplicate is detected, abort early with an error.
    let mut seen: HashSet<Pubkey> = HashSet::new();

    for rel in relationships.into_iter() {
        // If insert returns false, the value was already present – duplicate detected.
        if !seen.insert(rel.key) {
            return Err(MplCoreError::DuplicateEntry.into());
        }

        if rel.key == *ctx.accounts.group.key
            && matches!(
                rel.kind,
                RelationshipKind::ChildGroup | RelationshipKind::ParentGroup
            )
        {
            msg!("Error: Group cannot reference itself as child or parent group");
            return Err(MplCoreError::IncorrectAccount.into());
        }

        match rel.kind {
            RelationshipKind::Collection => collections_vec.push(rel.key),
            RelationshipKind::ChildGroup => child_groups_vec.push(rel.key),
            RelationshipKind::ParentGroup => parent_groups_vec.push(rel.key),
            RelationshipKind::Asset => assets_vec.push(rel.key),
        }
    }

    if collections_vec.len() > MAX_GROUP_VECTOR_SIZE
        || child_groups_vec.len() > MAX_GROUP_VECTOR_SIZE
        || assets_vec.len() > MAX_GROUP_VECTOR_SIZE
    {
        return Err(MplCoreError::GroupVectorFull.into());
    }

    if parent_groups_vec.len() > MAX_GROUP_NESTING_DEPTH {
        return Err(MplCoreError::GroupNestingDepthExceeded.into());
    }

    let new_group = GroupV1::new(
        *authority_info.key,
        name.clone(),
        uri.clone(),
        collections_vec.clone(),
        child_groups_vec.clone(),
        parent_groups_vec.clone(),
        assets_vec.clone(),
    );

    let serialized_data = new_group.try_to_vec()?;
    let lamports = rent.minimum_balance(serialized_data.len());

    // Create the on-chain account for the group via CPI to System Program.
    invoke(
        &system_instruction::create_account(
            ctx.accounts.payer.key,
            ctx.accounts.group.key,
            lamports,
            serialized_data.len() as u64,
            &crate::ID,
        ),
        &[
            ctx.accounts.payer.clone(),
            ctx.accounts.group.clone(),
            ctx.accounts.system_program.clone(),
        ],
    )?;

    save_flat_group(
        ctx.accounts.group,
        &new_group,
        ctx.accounts.payer,
        ctx.accounts.system_program,
    )?;

    // ----------------------------------------------------------------------
    // POST-CREATION LINKING LOGIC
    // ----------------------------------------------------------------------
    let remaining_accounts = ctx.remaining_accounts;
    let expected_accounts =
        collections_vec.len() + child_groups_vec.len() + parent_groups_vec.len() + assets_vec.len();

    if remaining_accounts.len() != expected_accounts {
        msg!(
            "Error: Incorrect number of remaining accounts (expected {}, got {}).",
            expected_accounts,
            remaining_accounts.len()
        );
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Offsets into remaining_accounts slice
    let mut cursor: usize = 0;

    // ----------------- LINK COLLECTIONS -----------------------------------
    for (i, collection_key) in collections_vec.iter().enumerate() {
        let collection_info = &remaining_accounts[cursor + i];

        // Account correctness checks
        if collection_info.key != collection_key {
            msg!("Error: Collection account mismatch at index {}", i);
            return Err(MplCoreError::IncorrectAccount.into());
        }

        if !collection_info.is_writable {
            msg!("Error: Collection account must be writable");
            return Err(ProgramError::InvalidAccountData);
        }

        // Authority check (collection update authority or delegate)
        if !is_valid_collection_authority(collection_info, authority_info)? {
            msg!("Error: Signer is not collection update authority/delegate");
            return Err(MplCoreError::InvalidAuthority.into());
        }

        // Add Groups plugin entry.
        process_collection_groups_plugin_add(
            collection_info,
            *ctx.accounts.group.key,
            ctx.accounts.payer,
            ctx.accounts.system_program,
        )?;
    }

    cursor += collections_vec.len();

    // ----------------- LINK CHILD GROUPS ----------------------------------
    for (i, child_key) in child_groups_vec.iter().enumerate() {
        let child_info = &remaining_accounts[cursor + i];

        if child_info.key != child_key {
            msg!("Error: Child group account mismatch at index {}", i);
            return Err(MplCoreError::IncorrectAccount.into());
        }

        if !child_info.is_writable {
            msg!("Error: Child group account must be writable");
            return Err(ProgramError::InvalidAccountData);
        }

        let mut child_group = GroupV1::load(child_info, 0)?;

        if !is_valid_group_authority(child_info, authority_info)? {
            msg!("Error: Signer is not child group update authority/delegate");
            return Err(MplCoreError::InvalidAuthority.into());
        }

        if !child_group.parent_groups.contains(ctx.accounts.group.key) {
            child_group.parent_groups.push(*ctx.accounts.group.key);
            save_flat_group(
                child_info,
                &child_group,
                ctx.accounts.payer,
                ctx.accounts.system_program,
            )?;
        }
    }

    cursor += child_groups_vec.len();

    // ----------------- LINK PARENT GROUPS ---------------------------------
    for (i, parent_key) in parent_groups_vec.iter().enumerate() {
        let parent_info = &remaining_accounts[cursor + i];

        if parent_info.key != parent_key {
            msg!("Error: Parent group account mismatch at index {}", i);
            return Err(MplCoreError::IncorrectAccount.into());
        }

        if !parent_info.is_writable {
            msg!("Error: Parent group account must be writable");
            return Err(ProgramError::InvalidAccountData);
        }

        let mut parent_group = GroupV1::load(parent_info, 0)?;

        if !is_valid_group_authority(parent_info, authority_info)? {
            msg!("Error: Signer is not parent group update authority/delegate");
            return Err(MplCoreError::InvalidAuthority.into());
        }

        if !parent_group.groups.contains(ctx.accounts.group.key) {
            parent_group.groups.push(*ctx.accounts.group.key);
            save_flat_group(
                parent_info,
                &parent_group,
                ctx.accounts.payer,
                ctx.accounts.system_program,
            )?;
        }
    }

    cursor += parent_groups_vec.len();

    // ----------------- LINK ASSETS -----------------------------------------
    for (i, asset_key) in assets_vec.iter().enumerate() {
        let asset_info = &remaining_accounts[cursor + i];

        if asset_info.key != asset_key {
            msg!("Error: Asset account mismatch at index {}", i);
            return Err(MplCoreError::IncorrectAccount.into());
        }

        if !asset_info.is_writable {
            msg!("Error: Asset account must be writable");
            return Err(ProgramError::InvalidAccountData);
        }

        // Authority check (asset update authority or delegate)
        if !is_valid_asset_authority(asset_info, authority_info)? {
            msg!("Error: Signer is not asset update authority/delegate");
            return Err(MplCoreError::InvalidAuthority.into());
        }

        // Add Assets plugin entry.
        process_asset_groups_plugin_add(
            asset_info,
            *ctx.accounts.group.key,
            ctx.accounts.payer,
            ctx.accounts.system_program,
        )?;
    }

    Ok(())
}
