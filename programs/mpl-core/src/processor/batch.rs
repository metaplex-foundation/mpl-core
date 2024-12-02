use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::instruction::accounts::BatchV1Accounts;

use super::{
    add_collection_external_plugin_adapter, add_collection_plugin, add_external_plugin_adapter,
    add_plugin, approve_collection_plugin_authority, approve_plugin_authority, burn,
    burn_collection, collect, compress, create_collection_v1, create_collection_v2, create_v1,
    create_v2, decompress, remove_collection_external_plugin_adapter, remove_collection_plugin,
    remove_external_plugin_adapter, remove_plugin, revoke_collection_plugin_authority,
    revoke_plugin_authority, transfer, update_collection,
    update_collection_external_plugin_adapter, update_collection_plugin,
    update_external_plugin_adapter, update_plugin, update_v1, update_v2,
    write_collection_external_plugin_adapter_data, write_external_plugin_adapter_data,
    MplAssetInstruction,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct BatchV1Args {
    pub(crate) num_accounts: Vec<u8>,
    pub(crate) instructions: Vec<u8>,
}

pub(crate) fn batch_v1<'a>(accounts: &'a [AccountInfo<'a>], args: BatchV1Args) -> ProgramResult {
    // Accounts.
    let ctx = BatchV1Accounts::context(accounts)?;
    let mut index = 0;
    let mut accounts_index = 0;

    let mut ix_slice = args.instructions.as_slice();
    while !ix_slice.is_empty() {
        let ix = MplAssetInstruction::deserialize(&mut ix_slice)?;
        match ix {
            MplAssetInstruction::CreateV1(args) => {
                create_v1(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::CreateCollectionV1(args) => {
                create_collection_v1(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::AddPluginV1(args) => {
                add_plugin(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::AddCollectionPluginV1(args) => {
                add_collection_plugin(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::RemovePluginV1(args) => {
                remove_plugin(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::RemoveCollectionPluginV1(args) => {
                remove_collection_plugin(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::UpdatePluginV1(args) => {
                update_plugin(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::UpdateCollectionPluginV1(args) => {
                update_collection_plugin(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::ApprovePluginAuthorityV1(args) => {
                approve_plugin_authority(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::ApproveCollectionPluginAuthorityV1(args) => {
                approve_collection_plugin_authority(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::RevokePluginAuthorityV1(args) => {
                revoke_plugin_authority(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::RevokeCollectionPluginAuthorityV1(args) => {
                revoke_collection_plugin_authority(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::BurnV1(args) => burn(&ctx.remaining_accounts[index..], args),
            MplAssetInstruction::BurnCollectionV1(args) => {
                burn_collection(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::TransferV1(args) => {
                transfer(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::UpdateV1(args) => {
                update_v1(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::UpdateCollectionV1(args) => {
                update_collection(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::CompressV1(args) => {
                compress(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::DecompressV1(args) => {
                decompress(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::Collect => collect(accounts),
            MplAssetInstruction::CreateV2(args) => {
                create_v2(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::CreateCollectionV2(args) => {
                create_collection_v2(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::AddExternalPluginAdapterV1(args) => {
                add_external_plugin_adapter(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::AddCollectionExternalPluginAdapterV1(args) => {
                add_collection_external_plugin_adapter(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::RemoveExternalPluginAdapterV1(args) => {
                remove_external_plugin_adapter(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::RemoveCollectionExternalPluginAdapterV1(args) => {
                remove_collection_external_plugin_adapter(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::UpdateExternalPluginAdapterV1(args) => {
                update_external_plugin_adapter(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::UpdateCollectionExternalPluginAdapterV1(args) => {
                update_collection_external_plugin_adapter(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::WriteExternalPluginAdapterDataV1(args) => {
                write_external_plugin_adapter_data(&ctx.remaining_accounts[index..], args)
            }
            MplAssetInstruction::WriteCollectionExternalPluginAdapterDataV1(args) => {
                write_collection_external_plugin_adapter_data(
                    &ctx.remaining_accounts[index..],
                    args,
                )
            }
            MplAssetInstruction::UpdateV2(args) => {
                update_v2(&ctx.remaining_accounts[index..], args)
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }?;

        index += args.num_accounts[accounts_index] as usize;
        accounts_index += 1;
    }

    Ok(())
}
