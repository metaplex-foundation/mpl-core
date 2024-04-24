use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_memory::sol_memcpy, rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::CreateCollectionV1Accounts,
    plugins::{
        create_plugin_meta, initialize_plugin, CheckResult, Plugin, PluginAuthorityPair,
        PluginType, PluginValidationContext, ValidationResult,
    },
    state::{Authority, CollectionV1, Key},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct CreateCollectionV1Args {
    pub(crate) name: String,
    pub(crate) uri: String,
    pub(crate) plugins: Option<Vec<PluginAuthorityPair>>,
}

pub(crate) fn create_collection<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: CreateCollectionV1Args,
) -> ProgramResult {
    // Accounts.
    let ctx = CreateCollectionV1Accounts::context(accounts)?;
    let rent = Rent::get()?;

    // Guards.
    assert_signer(ctx.accounts.collection)?;
    assert_signer(ctx.accounts.payer)?;

    if *ctx.accounts.system_program.key != system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    let new_collection = CollectionV1 {
        key: Key::CollectionV1,
        update_authority: *ctx
            .accounts
            .update_authority
            .unwrap_or(ctx.accounts.payer)
            .key,
        name: args.name,
        uri: args.uri,
        num_minted: 0,
        current_size: 0,
    };

    let serialized_data = new_collection.try_to_vec()?;

    let lamports = rent.minimum_balance(serialized_data.len());

    // CPI to the System Program.
    invoke(
        &system_instruction::create_account(
            ctx.accounts.payer.key,
            ctx.accounts.collection.key,
            lamports,
            serialized_data.len() as u64,
            &crate::ID,
        ),
        &[
            ctx.accounts.payer.clone(),
            ctx.accounts.collection.clone(),
            ctx.accounts.system_program.clone(),
        ],
    )?;

    sol_memcpy(
        &mut ctx.accounts.collection.try_borrow_mut_data()?,
        &serialized_data,
        serialized_data.len(),
    );

    if let Some(plugins) = args.plugins {
        if !plugins.is_empty() {
            let (mut plugin_header, mut plugin_registry) = create_plugin_meta::<CollectionV1>(
                new_collection,
                ctx.accounts.collection,
                ctx.accounts.payer,
                ctx.accounts.system_program,
            )?;

            let mut approved = true;
            let mut force_approved = false;
            for plugin in &plugins {
                // Cannot have owner-managed plugins on collection.
                if plugin.plugin.manager() == Authority::Owner {
                    return Err(MplCoreError::InvalidAuthority.into());
                }

                // TODO move into plugin validation when asset/collection is part of validation context
                let plugin_type = PluginType::from(&plugin.plugin);
                if plugin_type == PluginType::Edition {
                    return Err(MplCoreError::InvalidPlugin.into());
                }

                if PluginType::check_create(&plugin_type) != CheckResult::None {
                    let validation_ctx = PluginValidationContext {
                        self_authority: &plugin.authority.unwrap_or(plugin.plugin.manager()),
                        authority_info: ctx.accounts.payer,
                        resolved_authorities: None,
                        new_owner: None,
                        target_plugin: None,
                    };
                    match Plugin::validate_create(&plugin.plugin, &validation_ctx)? {
                        ValidationResult::Rejected => approved = false,
                        ValidationResult::ForceApproved => force_approved = true,
                        _ => (),
                    };
                }
                initialize_plugin::<CollectionV1>(
                    &plugin.plugin,
                    &plugin.authority.unwrap_or(plugin.plugin.manager()),
                    &mut plugin_header,
                    &mut plugin_registry,
                    ctx.accounts.collection,
                    ctx.accounts.payer,
                    ctx.accounts.system_program,
                )?;
            }

            if !(approved || force_approved) {
                return Err(MplCoreError::InvalidAuthority.into());
            }
        }
    }

    Ok(())
}
