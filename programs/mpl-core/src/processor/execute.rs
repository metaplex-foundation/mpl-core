use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::{assert_derivation, assert_signer};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
};
use solana_system_interface::{instruction as system_instruction, program as system_program};

use crate::{
    error::MplCoreError,
    instruction::accounts::ExecuteV1Accounts,
    plugins::{ExternalPluginAdapter, HookableLifecycleEvent, Plugin, PluginType},
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

    let bump = assert_derivation(
        &crate::ID,
        ctx.accounts.asset_signer,
        &[PREFIX.as_bytes(), ctx.accounts.asset.key.as_ref()],
        MplCoreError::InvalidExecutePda,
    )?;

    let payer_is_pda = ctx.accounts.payer.key == ctx.accounts.asset_signer.key;

    let authority = if payer_is_pda {
        let authority = ctx.accounts.authority.ok_or(MplCoreError::MissingSigner)?;
        assert_signer(authority)?;
        authority
    } else {
        assert_signer(ctx.accounts.payer)?;
        resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?
    };

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
        Some(ExternalPluginAdapter::validate_execute),
        Some(HookableLifecycleEvent::Execute),
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    let fee = get_execute_fee()?;
    let transfer_ix =
        system_instruction::transfer(ctx.accounts.payer.key, ctx.accounts.asset.key, fee);

    if payer_is_pda {
        // Payer is the asset signer PDA -- use invoke_signed so the PDA can
        // pay the execute fee from its own lamports.
        invoke_signed(
            &transfer_ix,
            &[ctx.accounts.payer.clone(), ctx.accounts.asset.clone()],
            &[&[PREFIX.as_bytes(), ctx.accounts.asset.key.as_ref(), &[bump]]],
        )?;
    } else {
        invoke(
            &transfer_ix,
            &[ctx.accounts.payer.clone(), ctx.accounts.asset.clone()],
        )?;
    }

    // If the first remaining account is an ExecutionDelegateRecordV1, strip it
    // before passing to the CPI -- it was only needed for plugin validation.
    let cpi_accounts = if let Some(first) = ctx.remaining_accounts.first() {
        if first.owner == &solana_program::pubkey::Pubkey::new_from_array(mpl_agent_tools::ID.to_bytes())
            && first.data_len() > 0
            && first.data.borrow()[0]
                == mpl_agent_tools::types::Key::ExecutionDelegateRecordV1 as u8
        {
            &ctx.remaining_accounts[1..]
        } else {
            ctx.remaining_accounts
        }
    } else {
        ctx.remaining_accounts
    };

    process_execute(
        ctx.accounts.asset.key,
        ctx.accounts.asset_signer.key,
        ctx.accounts.program_id.key,
        args.instruction_data,
        cpi_accounts,
        bump,
    )
}

fn process_execute(
    asset_key: &Pubkey,
    asset_signer: &Pubkey,
    program_id: &Pubkey,
    instruction_data: Vec<u8>,
    remaining_accounts: &[AccountInfo],
    bump: u8,
) -> ProgramResult {
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
