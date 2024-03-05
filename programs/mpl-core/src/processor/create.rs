use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_memory::sol_memcpy, rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::CreateAccounts,
    plugins::{create_meta_idempotent, initialize_plugin, CheckResult, Plugin, ValidationResult},
    state::{Asset, Compressible, DataState, HashedAsset, Key, UpdateAuthority, COLLECT_AMOUNT},
    utils::fetch_core_data,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CreateArgs {
    pub data_state: DataState,
    pub name: String,
    pub uri: String,
    pub plugins: Vec<Plugin>,
}

pub(crate) fn create<'a>(accounts: &'a [AccountInfo<'a>], args: CreateArgs) -> ProgramResult {
    // Accounts.
    let ctx = CreateAccounts::context(accounts)?;
    let rent = Rent::get()?;

    // Guards.
    assert_signer(ctx.accounts.asset)?;
    assert_signer(ctx.accounts.payer)?;

    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    let update_authority = match ctx.accounts.collection {
        Some(collection) => UpdateAuthority::Collection(*collection.key),
        None => UpdateAuthority::Address(
            *ctx.accounts
                .update_authority
                .unwrap_or(ctx.accounts.payer)
                .key,
        ),
    };

    if update_authority.validate_create(&ctx.accounts, &args)? == ValidationResult::Rejected {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    let new_asset = Asset {
        key: Key::Asset,
        update_authority,
        owner: *ctx
            .accounts
            .owner
            .unwrap_or(ctx.accounts.update_authority.unwrap_or(ctx.accounts.payer))
            .key,
        name: args.name.clone(),
        uri: args.uri.clone(),
    };

    let serialized_data = new_asset.try_to_vec()?;

    let serialized_data = match args.data_state {
        DataState::AccountState => serialized_data,
        DataState::LedgerState => {
            new_asset.wrap()?;
            let hashed_asset = HashedAsset::new(new_asset.hash()?);
            hashed_asset.try_to_vec()?
        }
    };

    let lamports = rent.minimum_balance(serialized_data.len()) + COLLECT_AMOUNT;

    // CPI to the System Program.
    invoke(
        &system_instruction::create_account(
            ctx.accounts.payer.key,
            ctx.accounts.asset.key,
            lamports,
            serialized_data.len() as u64,
            &crate::id(),
        ),
        &[
            ctx.accounts.payer.clone(),
            ctx.accounts.asset.clone(),
            ctx.accounts.system_program.clone(),
        ],
    )?;

    sol_memcpy(
        &mut ctx.accounts.asset.try_borrow_mut_data()?,
        &serialized_data,
        serialized_data.len(),
    );

    //TODO: Do compressed state
    if args.data_state == DataState::AccountState {
        create_meta_idempotent(
            ctx.accounts.asset,
            ctx.accounts.payer,
            ctx.accounts.system_program,
        )?;

        for plugin in &args.plugins {
            initialize_plugin(
                plugin,
                &[plugin.default_authority()?],
                ctx.accounts.asset,
                ctx.accounts.payer,
                ctx.accounts.system_program,
            )?;
        }

        let (_, _, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

        let mut approved = true;
        // match Asset::check_create() {
        //     CheckResult::CanApprove | CheckResult::CanReject => {
        //         match asset.validate_create(&ctx.accounts)? {
        //             ValidationResult::Approved => {
        //                 approved = true;
        //             }
        //             ValidationResult::Rejected => return Err(MplCoreError::InvalidAuthority.into()),
        //             ValidationResult::Pass => (),
        //         }
        //     }
        //     CheckResult::None => (),
        // };

        if let Some(plugin_registry) = plugin_registry {
            for record in plugin_registry.registry {
                if matches!(
                    record.plugin_type.check_create(),
                    CheckResult::CanApprove | CheckResult::CanReject
                ) {
                    let result = Plugin::load(ctx.accounts.asset, record.offset)?.validate_create(
                        ctx.accounts
                            .owner
                            .unwrap_or(ctx.accounts.update_authority.unwrap_or(ctx.accounts.payer)),
                        &args,
                        &record.authorities,
                    )?;
                    if result == ValidationResult::Rejected {
                        return Err(MplCoreError::InvalidAuthority.into());
                    } else if result == ValidationResult::Approved {
                        approved = true;
                    }
                }
            }
        };

        if !approved {
            return Err(MplCoreError::InvalidAuthority.into());
        }
    }

    Ok(())
}
