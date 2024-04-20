use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_memory::sol_memcpy, rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::CreateCollectionV2Accounts,
    plugins::{
        create_meta_idempotent, create_plugin_meta, initialize_external_plugin, initialize_plugin,
        CheckResult, ExternalCheckResult, ExternalPlugin, ExternalPluginInitInfo, Plugin,
        PluginAuthorityPair, PluginType, PluginValidationContext, ValidationResult,
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

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct CreateCollectionV2Args {
    pub(crate) name: String,
    pub(crate) uri: String,
    pub(crate) plugins: Option<Vec<PluginAuthorityPair>>,
    pub(crate) external_plugins: Option<Vec<ExternalPluginInitInfo>>,
}

impl From<CreateCollectionV1Args> for CreateCollectionV2Args {
    fn from(item: CreateCollectionV1Args) -> Self {
        CreateCollectionV2Args {
            name: item.name,
            uri: item.uri,
            plugins: item.plugins,
            external_plugins: None,
        }
    }
}

pub(crate) fn create_collection_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: CreateCollectionV1Args,
) -> ProgramResult {
    process_create_collection(accounts, CreateCollectionV2Args::from(args))
}
pub(crate) fn create_collection_v2<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: CreateCollectionV2Args,
) -> ProgramResult {
    process_create_collection(accounts, args)
}

pub(crate) fn process_create_collection<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: CreateCollectionV2Args,
) -> ProgramResult {
    // Accounts.
    let ctx = CreateCollectionV2Accounts::context(accounts)?;
    let rent = Rent::get()?;

    // Guards.
    assert_signer(ctx.accounts.collection)?;
    assert_signer(ctx.accounts.payer)?;
    let authority = ctx.accounts.update_authority.unwrap_or(ctx.accounts.payer);

    if *ctx.accounts.system_program.key != system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    let new_collection = CollectionV1 {
        key: Key::CollectionV1,
        update_authority: *authority.key,
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

    let mut approved = true;
    let mut force_approved = false;
    if let Some(plugins) = args.plugins {
        if !plugins.is_empty() {
            let (mut plugin_header, mut plugin_registry) = create_plugin_meta::<CollectionV1>(
                &new_collection,
                ctx.accounts.collection,
                ctx.accounts.payer,
                ctx.accounts.system_program,
            )?;
            for plugin in &plugins {
                // Cannot have owner-managed plugins on collection.
                if plugin.plugin.manager() == Authority::Owner {
                    return Err(MplCoreError::InvalidAuthority.into());
                }

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
        }
    }

    if let Some(plugins) = args.external_plugins {
        if !plugins.is_empty() {
            let (_, mut plugin_header, mut plugin_registry) = create_meta_idempotent::<CollectionV1>(
                ctx.accounts.collection,
                ctx.accounts.payer,
                ctx.accounts.system_program,
            )?;
            for plugin_init_info in &plugins {
                if ExternalPlugin::check_create(plugin_init_info) != ExternalCheckResult::none() {
                    let validation_ctx = PluginValidationContext {
                        // External plugins are always managed by the update authority.
                        self_authority: &Authority::UpdateAuthority,
                        authority_info: authority,
                        resolved_authorities: None,
                        new_owner: None,
                        target_plugin: None,
                    };
                    if ExternalPlugin::validate_create(plugin_init_info, &validation_ctx)?
                        == ValidationResult::Rejected
                    {
                        approved = false;
                    };
                }
                initialize_external_plugin::<CollectionV1>(
                    plugin_init_info,
                    &mut plugin_header,
                    &mut plugin_registry,
                    ctx.accounts.collection,
                    ctx.accounts.payer,
                    ctx.accounts.system_program,
                )?;
            }
        }
    }

    if !(approved || force_approved) {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    Ok(())
}
