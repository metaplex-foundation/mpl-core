use crate::instruction::MplAssetInstruction;
use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

mod create;
pub(crate) use create::*;

mod create_collection;
pub(crate) use create_collection::*;

mod add_plugin;
pub(crate) use add_plugin::*;

mod remove_plugin;
pub(crate) use remove_plugin::*;

mod add_plugin_authority;
pub(crate) use add_plugin_authority::*;

mod remove_plugin_authority;
pub(crate) use remove_plugin_authority::*;

mod burn;
pub(crate) use burn::*;

mod transfer;
pub(crate) use transfer::*;

mod update;
pub(crate) use update::*;

mod compress;
pub(crate) use compress::*;

mod decompress;
pub(crate) use decompress::*;

mod update_plugin;
pub(crate) use update_plugin::*;

/// Standard processor that deserializes and instruction and routes it to the appropriate handler.
pub fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction: MplAssetInstruction = MplAssetInstruction::try_from_slice(instruction_data)?;
    match instruction {
        MplAssetInstruction::Create(args) => {
            msg!("Instruction: Create");
            create(accounts, args)
        }
        MplAssetInstruction::CreateCollection(args) => {
            msg!("Instruction: CreateCollection");
            create_collection(accounts, args)
        }
        MplAssetInstruction::AddPlugin(args) => {
            msg!("Instruction: AddPlugin");
            add_plugin(accounts, args)
        }
        MplAssetInstruction::AddCollectionPlugin(args) => {
            msg!("Instruction: AddCollectionPlugin");
            add_collection_plugin(accounts, args)
        }
        MplAssetInstruction::RemovePlugin(args) => {
            msg!("Instruction: RemovePlugin");
            remove_plugin(accounts, args)
        }
        MplAssetInstruction::RemoveCollectionPlugin(args) => {
            msg!("Instruction: RemoveCollectionPlugin");
            remove_collection_plugin(accounts, args)
        }
        MplAssetInstruction::UpdatePlugin(args) => {
            msg!("Instruction: UpdatePlugin");
            update_plugin(accounts, args)
        }
        MplAssetInstruction::UpdateCollectionPlugin(args) => {
            msg!("Instruction: UpdateCollectionPlugin");
            update_collection_plugin(accounts, args)
        }
        MplAssetInstruction::AddPluginAuthority(args) => {
            msg!("Instruction: AddPluginAuthority");
            add_plugin_authority(accounts, args)
        }
        MplAssetInstruction::AddCollectionPluginAuthority(args) => {
            msg!("Instruction: AddCollectionPluginAuthority");
            add_collection_plugin_authority(accounts, args)
        }
        MplAssetInstruction::RemovePluginAuthority(args) => {
            msg!("Instruction: RemovePluginAuthority");
            remove_plugin_authority(accounts, args)
        }
        MplAssetInstruction::RemoveCollectionPluginAuthority(args) => {
            msg!("Instruction: RemoveCollectionPluginAuthority");
            remove_collection_plugin_authority(accounts, args)
        }
        MplAssetInstruction::Burn(args) => {
            msg!("Instruction: Burn");
            burn(accounts, args)
        }
        MplAssetInstruction::BurnCollection => {
            msg!("Instruction: BurnCollection");
            burn_collection(accounts)
        }
        MplAssetInstruction::Transfer(args) => {
            msg!("Instruction: Transfer");
            transfer(accounts, args)
        }
        MplAssetInstruction::Update(args) => {
            msg!("Instruction: Update");
            update(accounts, args)
        }
        MplAssetInstruction::UpdateCollection(args) => {
            msg!("Instruction: UpdateCollection");
            update_collection(accounts, args)
        }
        MplAssetInstruction::Compress(args) => {
            msg!("Instruction: Compress");
            compress(accounts, args)
        }
        MplAssetInstruction::Decompress(args) => {
            msg!("Instruction: Decompress");
            decompress(accounts, args)
        }
    }
}
