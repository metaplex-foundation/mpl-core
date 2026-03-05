use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_memory::sol_memcpy, rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};
use std::collections::HashSet;

use crate::{
    error::MplCoreError,
    instruction::accounts::CreateCollectionV2Accounts,
    plugins::{
        create_meta_idempotent, create_plugin_meta, initialize_external_plugin_adapter,
        initialize_plugin, BubblegumV2, CheckResult, ExternalPluginAdapterInitInfo, Plugin,
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
    pub(crate) external_plugin_adapters: Option<Vec<ExternalPluginAdapterInitInfo>>,
}

impl From<CreateCollectionV1Args> for CreateCollectionV2Args {
    fn from(item: CreateCollectionV1Args) -> Self {
        CreateCollectionV2Args {
            name: item.name,
            uri: item.uri,
            plugins: item.plugins,
            external_plugin_adapters: None,
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
    let mut has_bubblegum_v2 = false;
    if let Some(plugins) = args.plugins {
        if !plugins.is_empty() {
            let (header_offset, mut plugin_header, mut plugin_registry) =
                create_plugin_meta::<CollectionV1>(
                    &new_collection,
                    ctx.accounts.collection,
                    ctx.accounts.payer,
                    ctx.accounts.system_program,
                )?;

            // See if the new collection will have the Bubblegum V2 plugin.
            has_bubblegum_v2 = plugins
                .iter()
                .any(|p| PluginType::from(&p.plugin) == PluginType::BubblegumV2);

            // If the Bubblegum V2 plugin is present, only a limited set of other plugins is
            // allowed on the collection.
            let bubblegum_v2_allow_list: Option<HashSet<PluginType>> = if has_bubblegum_v2 {
                Some(BubblegumV2::ALLOW_LIST.iter().cloned().collect())
            } else {
                None
            };

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

                // Validate against the Bubblegum V2 allow list.
                if let Some(allow_list) = &bubblegum_v2_allow_list {
                    if plugin_type != PluginType::BubblegumV2 && !allow_list.contains(&plugin_type)
                    {
                        return Err(MplCoreError::BlockedByBubblegumV2.into());
                    }
                }

                if PluginType::check_create(&plugin_type) != CheckResult::None {
                    let validation_ctx = PluginValidationContext {
                        accounts,
                        asset_info: None,
                        collection_info: Some(ctx.accounts.collection),
                        self_authority: &plugin.authority.unwrap_or(plugin.plugin.manager()),
                        authority_info: ctx.accounts.payer,
                        resolved_authorities: None,
                        new_owner: None,
                        new_asset_authority: None,
                        new_collection_authority: None,
                        target_plugin: None,
                        target_plugin_authority: None,
                        target_external_plugin: None,
                        target_external_plugin_authority: None,
                    };
                    match Plugin::validate_create(&plugin.plugin, &validation_ctx)? {
                        ValidationResult::Rejected => approved = false,
                        ValidationResult::ForceApproved => force_approved = true,
                        _ => (),
                    };
                }

                // Bubblegum V2 plugin always has a fixed authority.
                let authority = if plugin_type == PluginType::BubblegumV2 {
                    if let Some(supplied) = &plugin.authority {
                        if supplied != &plugin.plugin.manager() {
                            return Err(MplCoreError::InvalidAuthority.into());
                        }
                    }
                    plugin.plugin.manager()
                } else {
                    plugin.authority.unwrap_or(plugin.plugin.manager())
                };

                initialize_plugin::<CollectionV1>(
                    &plugin.plugin,
                    &authority,
                    header_offset,
                    &mut plugin_header,
                    &mut plugin_registry,
                    ctx.accounts.collection,
                    ctx.accounts.payer,
                    ctx.accounts.system_program,
                )?;
            }
        }
    }

    if let Some(plugins) = args.external_plugin_adapters {
        if !plugins.is_empty() {
            if has_bubblegum_v2 {
                return Err(MplCoreError::BlockedByBubblegumV2.into());
            }

            let (_, header_offset, mut plugin_header, mut plugin_registry) =
                create_meta_idempotent::<CollectionV1>(
                    ctx.accounts.collection,
                    ctx.accounts.payer,
                    ctx.accounts.system_program,
                )?;
            for plugin_init_info in &plugins {
                if let ExternalPluginAdapterInitInfo::DataSection(_) = plugin_init_info {
                    return Err(MplCoreError::CannotAddDataSection.into());
                }

                initialize_external_plugin_adapter::<CollectionV1>(
                    plugin_init_info,
                    header_offset,
                    &mut plugin_header,
                    &mut plugin_registry,
                    ctx.accounts.collection,
                    ctx.accounts.payer,
                    ctx.accounts.system_program,
                    None,
                )?;
            }
        }
    }

    if !(approved || force_approved) {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    Ok(())
}
