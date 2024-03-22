use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_memory::sol_memcpy, rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::CreateV1Accounts,
    plugins::{
        create_plugin_meta, initialize_plugin, CheckResult, Plugin, PluginAuthorityPair,
        PluginType, ValidationResult,
    },
    state::{AssetV1, CollectionV1, DataState, SolanaAccount, UpdateAuthority, COLLECT_AMOUNT},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct CreateV1Args {
    pub(crate) data_state: DataState,
    pub(crate) name: String,
    pub(crate) uri: String,
    pub(crate) plugins: Option<Vec<PluginAuthorityPair>>,
}

pub(crate) fn create<'a>(accounts: &'a [AccountInfo<'a>], args: CreateV1Args) -> ProgramResult {
    // Accounts.
    let ctx = CreateV1Accounts::context(accounts)?;
    let rent = Rent::get()?;

    // Guards.
    assert_signer(ctx.accounts.asset)?;
    assert_signer(ctx.accounts.payer)?;

    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    let (update_authority, collection) = match ctx.accounts.collection {
        Some(collection) => (
            UpdateAuthority::Collection(*collection.key),
            Some(CollectionV1::load(collection, 0)?),
        ),
        None => (
            UpdateAuthority::Address(
                *ctx.accounts
                    .update_authority
                    .unwrap_or(ctx.accounts.payer)
                    .key,
            ),
            None,
        ),
    };

    if update_authority.validate_create(&ctx.accounts, &args)? == ValidationResult::Rejected {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    let new_asset = AssetV1::new(
        *ctx.accounts
            .owner
            .unwrap_or(ctx.accounts.update_authority.unwrap_or(ctx.accounts.payer))
            .key,
        update_authority,
        args.name.clone(),
        args.uri.clone(),
    );

    let serialized_data = new_asset.try_to_vec()?;

    let serialized_data = match args.data_state {
        DataState::AccountState => serialized_data,
        DataState::LedgerState => {
            // TODO: Implement minting compressed.
            solana_program::msg!("Error: Minting compressed is currently not available");
            return Err(MplCoreError::NotAvailable.into());
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

    if let (Some(plugins), DataState::AccountState) = (args.plugins, args.data_state) {
        if !plugins.is_empty() {
            let (mut plugin_header, mut plugin_registry) = create_plugin_meta::<AssetV1>(
                new_asset,
                ctx.accounts.asset,
                ctx.accounts.payer,
                ctx.accounts.system_program,
            )?;
            let mut approved = true;
            let mut force_approved = false;
            for plugin in &plugins {
                if PluginType::check_create(&PluginType::from(&plugin.plugin)) != CheckResult::None
                {
                    match Plugin::validate_create(
                        &plugin.plugin,
                        ctx.accounts.authority.unwrap_or(ctx.accounts.payer),
                        None,
                        &plugin.authority.unwrap_or(plugin.plugin.manager()),
                        None,
                        None,
                    )? {
                        ValidationResult::Rejected => approved = false,
                        ValidationResult::ForceApproved => force_approved = true,
                        _ => (),
                    };
                }
                initialize_plugin::<AssetV1>(
                    &plugin.plugin,
                    &plugin.authority.unwrap_or(plugin.plugin.manager()),
                    &mut plugin_header,
                    &mut plugin_registry,
                    ctx.accounts.asset,
                    ctx.accounts.payer,
                    ctx.accounts.system_program,
                )?;
            }

            if !(approved || force_approved) {
                return Err(MplCoreError::InvalidAuthority.into());
            }
        }
    }

    if let Some(mut collection) = collection {
        collection.increment()?;
        collection.save(ctx.accounts.collection.unwrap(), 0)?;
    };

    Ok(())
}
