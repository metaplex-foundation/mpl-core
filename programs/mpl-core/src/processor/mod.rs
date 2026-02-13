mod add_assets_to_group;
mod add_collections_to_group;
mod add_external_plugin_adapter;
mod add_group_external_plugin_adapter;
mod add_group_plugin;
mod add_groups_to_group;
mod add_plugin;
mod approve_group_plugin_authority;
mod approve_plugin_authority;
mod burn;
mod close_group;
mod collect;
mod compress;
mod create;
mod create_collection;
mod create_group;
mod decompress;
mod execute;
mod group_collection_plugin;
mod remove_assets_from_group;
mod remove_collections_from_group;
mod remove_external_plugin_adapter;
mod remove_group_external_plugin_adapter;
mod remove_group_plugin;
mod remove_groups_from_group;
mod remove_plugin;
mod revoke_group_plugin_authority;
mod revoke_plugin_authority;
mod transfer;
mod update;
mod update_collection_info;
mod update_external_plugin_adapter;
mod update_group;
mod update_group_plugin;
mod update_plugin;
mod write_external_plugin_adapter_data;
mod write_group_external_plugin_adapter_data;

pub(crate) use add_assets_to_group::*;
pub(crate) use add_collections_to_group::*;
pub(crate) use add_external_plugin_adapter::*;
pub(crate) use add_group_external_plugin_adapter::*;
pub(crate) use add_group_plugin::*;
pub(crate) use add_groups_to_group::*;
pub(crate) use add_plugin::*;
pub(crate) use approve_group_plugin_authority::*;
pub(crate) use approve_plugin_authority::*;
pub(crate) use burn::*;
pub(crate) use close_group::*;
pub(crate) use collect::*;
pub(crate) use compress::*;
pub(crate) use create::*;
pub(crate) use create_collection::*;
pub(crate) use create_group::*;
pub(crate) use decompress::*;
pub(crate) use execute::*;
pub(crate) use remove_assets_from_group::*;
pub(crate) use remove_collections_from_group::*;
pub(crate) use remove_external_plugin_adapter::*;
pub(crate) use remove_group_external_plugin_adapter::*;
pub(crate) use remove_group_plugin::*;
pub(crate) use remove_groups_from_group::*;
pub(crate) use remove_plugin::*;
pub(crate) use revoke_group_plugin_authority::*;
pub(crate) use revoke_plugin_authority::*;
pub(crate) use transfer::*;
pub(crate) use update::*;
pub(crate) use update_collection_info::*;
pub(crate) use update_external_plugin_adapter::*;
pub(crate) use update_group::*;
pub(crate) use update_group_plugin::*;
pub(crate) use update_plugin::*;
pub(crate) use write_external_plugin_adapter_data::*;
pub(crate) use write_group_external_plugin_adapter_data::*;

use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::instruction::MplAssetInstruction;

/// Standard processor that deserializes and instruction and routes it to the appropriate handler.
pub fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction: MplAssetInstruction = MplAssetInstruction::try_from_slice(instruction_data)?;
    match instruction {
        MplAssetInstruction::CreateV1(args) => {
            msg!("Instruction: Create");
            create_v1(accounts, args)
        }
        MplAssetInstruction::CreateCollectionV1(args) => {
            msg!("Instruction: CreateCollection");
            create_collection_v1(accounts, args)
        }
        MplAssetInstruction::AddPluginV1(args) => {
            msg!("Instruction: AddPlugin");
            add_plugin(accounts, args)
        }
        MplAssetInstruction::AddCollectionPluginV1(args) => {
            msg!("Instruction: AddCollectionPlugin");
            add_collection_plugin(accounts, args)
        }
        MplAssetInstruction::RemovePluginV1(args) => {
            msg!("Instruction: RemovePlugin");
            remove_plugin(accounts, args)
        }
        MplAssetInstruction::RemoveCollectionPluginV1(args) => {
            msg!("Instruction: RemoveCollectionPlugin");
            remove_collection_plugin(accounts, args)
        }
        MplAssetInstruction::UpdatePluginV1(args) => {
            msg!("Instruction: UpdatePlugin");
            update_plugin(accounts, args)
        }
        MplAssetInstruction::UpdateCollectionPluginV1(args) => {
            msg!("Instruction: UpdateCollectionPlugin");
            update_collection_plugin(accounts, args)
        }
        MplAssetInstruction::ApprovePluginAuthorityV1(args) => {
            msg!("Instruction: ApprovePluginAuthority");
            approve_plugin_authority(accounts, args)
        }
        MplAssetInstruction::ApproveCollectionPluginAuthorityV1(args) => {
            msg!("Instruction: ApproveCollectionPluginAuthority");
            approve_collection_plugin_authority(accounts, args)
        }
        MplAssetInstruction::RevokePluginAuthorityV1(args) => {
            msg!("Instruction: RevokePluginAuthority");
            revoke_plugin_authority(accounts, args)
        }
        MplAssetInstruction::RevokeCollectionPluginAuthorityV1(args) => {
            msg!("Instruction: RevokeCollectionPluginAuthority");
            revoke_collection_plugin_authority(accounts, args)
        }
        MplAssetInstruction::BurnV1(args) => {
            msg!("Instruction: Burn");
            burn(accounts, args)
        }
        MplAssetInstruction::BurnCollectionV1(args) => {
            msg!("Instruction: BurnCollection");
            burn_collection(accounts, args)
        }
        MplAssetInstruction::TransferV1(args) => {
            msg!("Instruction: Transfer");
            transfer(accounts, args)
        }
        MplAssetInstruction::UpdateV1(args) => {
            msg!("Instruction: Update");
            update_v1(accounts, args)
        }
        MplAssetInstruction::UpdateCollectionV1(args) => {
            msg!("Instruction: UpdateCollection");
            update_collection(accounts, args)
        }
        MplAssetInstruction::CompressV1(args) => {
            msg!("Instruction: Compress");
            compress(accounts, args)
        }
        MplAssetInstruction::DecompressV1(args) => {
            msg!("Instruction: Decompress");
            decompress(accounts, args)
        }
        MplAssetInstruction::Collect => collect(accounts),
        MplAssetInstruction::CreateV2(args) => {
            msg!("Instruction: CreateV2");
            create_v2(accounts, args)
        }
        MplAssetInstruction::CreateCollectionV2(args) => {
            msg!("Instruction: CreateCollectionV2");
            create_collection_v2(accounts, args)
        }
        MplAssetInstruction::AddExternalPluginAdapterV1(args) => {
            msg!("Instruction: AddExternalPluginAdapter");
            add_external_plugin_adapter(accounts, args)
        }
        MplAssetInstruction::AddCollectionExternalPluginAdapterV1(args) => {
            msg!("Instruction: AddCollectionExternalPluginAdapter");
            add_collection_external_plugin_adapter(accounts, args)
        }
        MplAssetInstruction::RemoveExternalPluginAdapterV1(args) => {
            msg!("Instruction: RemoveExternalPluginAdapter");
            remove_external_plugin_adapter(accounts, args)
        }
        MplAssetInstruction::RemoveCollectionExternalPluginAdapterV1(args) => {
            msg!("Instruction: RemoveCollectionExternalPluginAdapter");
            remove_collection_external_plugin_adapter(accounts, args)
        }
        MplAssetInstruction::WriteExternalPluginAdapterDataV1(args) => {
            msg!("Instruction: WriteExternalPluginAdapterDataV1");
            write_external_plugin_adapter_data(accounts, args)
        }
        MplAssetInstruction::WriteCollectionExternalPluginAdapterDataV1(args) => {
            msg!("Instruction: WriteCollectionExternalPluginAdapterDataV1");
            write_collection_external_plugin_adapter_data(accounts, args)
        }
        MplAssetInstruction::UpdateV2(args) => {
            msg!("Instruction: UpdateV2");
            update_v2(accounts, args)
        }

        MplAssetInstruction::ExecuteV1(args) => {
            msg!("Instruction: Execute");
            execute(accounts, args)
        }

        MplAssetInstruction::UpdateCollectionInfoV1(args) => {
            msg!("Instruction: UpdateCollectionInfoV1");
            update_collection_info(accounts, args)
        }

        MplAssetInstruction::CreateGroupV1(args) => {
            msg!("Instruction: CreateGroup");
            create_group_v1(accounts, args)
        }
        MplAssetInstruction::CloseGroupV1(args) => {
            msg!("Instruction: CloseGroup");
            close_group_v1(accounts, args)
        }
        MplAssetInstruction::UpdateGroupV1(args) => {
            msg!("Instruction: UpdateGroup");
            update_group_v1(accounts, args)
        }
        MplAssetInstruction::AddCollectionsToGroupV1(args) => {
            msg!("Instruction: AddCollectionsToGroup");
            add_collections_to_group_v1(accounts, args)
        }
        MplAssetInstruction::RemoveCollectionsFromGroupV1(args) => {
            msg!("Instruction: RemoveCollectionsFromGroup");
            remove_collections_from_group_v1(accounts, args)
        }
        MplAssetInstruction::AddGroupsToGroupV1(args) => {
            msg!("Instruction: AddGroupsToGroup");
            add_groups_to_group_v1(accounts, args)
        }
        MplAssetInstruction::RemoveGroupsFromGroupV1(args) => {
            msg!("Instruction: RemoveGroupsFromGroup");
            remove_groups_from_group_v1(accounts, args)
        }
        MplAssetInstruction::UpdateExternalPluginAdapterV1(args) => {
            msg!("Instruction: UpdateExternalPluginAdapter");
            update_external_plugin_adapter(accounts, args)
        }
        MplAssetInstruction::UpdateCollectionExternalPluginAdapterV1(args) => {
            msg!("Instruction: UpdateCollectionExternalPluginAdapter");
            update_collection_external_plugin_adapter(accounts, args)
        }
        MplAssetInstruction::AddGroupPluginV1(args) => {
            msg!("Instruction: AddGroupPlugin");
            add_group_plugin(accounts, args)
        }
        MplAssetInstruction::RemoveGroupPluginV1(args) => {
            msg!("Instruction: RemoveGroupPlugin");
            remove_group_plugin(accounts, args)
        }
        MplAssetInstruction::UpdateGroupPluginV1(args) => {
            msg!("Instruction: UpdateGroupPlugin");
            update_group_plugin(accounts, args)
        }
        MplAssetInstruction::ApproveGroupPluginAuthorityV1(args) => {
            msg!("Instruction: ApproveGroupPluginAuthority");
            approve_group_plugin_authority(accounts, args)
        }
        MplAssetInstruction::RevokeGroupPluginAuthorityV1(args) => {
            msg!("Instruction: RevokeGroupPluginAuthority");
            revoke_group_plugin_authority(accounts, args)
        }
        MplAssetInstruction::AddGroupExternalPluginAdapterV1(args) => {
            msg!("Instruction: AddGroupExternalPluginAdapter");
            add_group_external_plugin_adapter(accounts, args)
        }
        MplAssetInstruction::RemoveGroupExternalPluginAdapterV1(args) => {
            msg!("Instruction: RemoveGroupExternalPluginAdapter");
            remove_group_external_plugin_adapter(accounts, args)
        }
        MplAssetInstruction::WriteGroupExternalPluginAdapterDataV1(args) => {
            msg!("Instruction: WriteGroupExternalPluginAdapterDataV1");
            write_group_external_plugin_adapter_data(accounts, args)
        }
        MplAssetInstruction::AddAssetsToGroupV1(args) => {
            msg!("Instruction: AddAssetsToGroup");
            add_assets_to_group_v1(accounts, args)
        }
        MplAssetInstruction::RemoveAssetsFromGroupV1(args) => {
            msg!("Instruction: RemoveAssetsFromGroup");
            remove_assets_from_group_v1(accounts, args)
        }
    }
}
