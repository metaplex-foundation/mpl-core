use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    system_instruction, system_program,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::ExecuteV1Accounts,
    plugins::{Plugin, PluginType},
    state::{get_execute_fee, AssetV1, CollectionV1, Key},
    utils::{load_key, resolve_authority, validate_asset_permissions},
};

const PREFIX: &str = "mpl-core-execute";

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct ExecuteV1Args {
    pub instruction_data: Vec<u8>,
}

pub(crate) fn execute<'a>(accounts: &'a [AccountInfo<'a>], args: ExecuteV1Args) -> ProgramResult {
    // Accounts.
    let ctx = ExecuteV1Accounts::context(accounts)?;

    // Guards.
    if ctx.accounts.asset.owner != &crate::ID {
        return Err(MplCoreError::InvalidAsset.into());
    }

    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    if *ctx.accounts.system_program.key != system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if let Key::HashedAssetV1 = load_key(ctx.accounts.asset, 0)? {
        msg!("Error: Update plugin for compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    let (mut asset, _, _) = validate_asset_permissions(
        accounts,
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        None,
        None,
        None,
        None,
        None,
        AssetV1::check_execute,
        CollectionV1::check_execute,
        PluginType::check_execute,
        AssetV1::validate_execute,
        CollectionV1::validate_execute,
        Plugin::validate_execute,
        None,
        None,
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    invoke(
        &system_instruction::transfer(
            ctx.accounts.payer.key,
            ctx.accounts.asset.key,
            get_execute_fee()?,
        ),
        &[ctx.accounts.payer.clone(), ctx.accounts.asset.clone()],
    )?;

    process_execute(
        ctx.accounts.asset.key,
        ctx.accounts.asset_signer.key,
        ctx.accounts.program_id.key,
        args.instruction_data,
        ctx.remaining_accounts,
    )
}

fn process_execute(
    asset_key: &Pubkey,
    asset_signer: &Pubkey,
    program_id: &Pubkey,
    instruction_data: Vec<u8>,
    remaining_accounts: &[AccountInfo],
) -> ProgramResult {
    let (pda, bump) =
        Pubkey::find_program_address(&[PREFIX.as_bytes(), asset_key.as_ref()], &crate::ID);

    if pda != *asset_signer {
        return Err(MplCoreError::InvalidExecutePda.into());
    }

    invoke_signed(
        &Instruction {
            program_id: *program_id,
            accounts: remaining_accounts
                .iter()
                .map(|account| {
                    if *account.key == *asset_signer {
                        AccountMeta {
                            pubkey: *account.key,
                            is_signer: true,
                            is_writable: account.is_writable,
                        }
                    } else {
                        AccountMeta {
                            pubkey: *account.key,
                            is_signer: account.is_signer,
                            is_writable: account.is_writable,
                        }
                    }
                })
                .collect(),
            data: instruction_data,
        },
        remaining_accounts,
        &[&[PREFIX.as_bytes(), asset_key.as_ref(), &[bump]]],
    )
}
