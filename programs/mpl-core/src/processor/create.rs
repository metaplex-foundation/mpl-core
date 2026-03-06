use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_memory::sol_memcpy, rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::CreateV2Accounts,
    plugins::{
        create_meta_idempotent, create_plugin_meta, initialize_external_plugin_adapter,
        initialize_plugin, CheckResult, ExternalCheckResultBits, ExternalPluginAdapter,
        ExternalPluginAdapterInitInfo, HookableLifecycleEvent, Plugin, PluginAuthorityPair,
        PluginType, PluginValidationContext, ValidationResult,
    },
    state::{
        get_create_fee, AssetV1, Authority, CollectionV1, DataState, SolanaAccount, UpdateAuthority,
    },
    utils::{resolve_authority, validate_asset_permissions},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct CreateV1Args {
    pub(crate) data_state: DataState,
    pub(crate) name: String,
    pub(crate) uri: String,
    pub(crate) plugins: Option<Vec<PluginAuthorityPair>>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct CreateV2Args {
    pub(crate) data_state: DataState,
    pub(crate) name: String,
    pub(crate) uri: String,
    pub(crate) plugins: Option<Vec<PluginAuthorityPair>>,
    pub(crate) external_plugin_adapters: Option<Vec<ExternalPluginAdapterInitInfo>>,
}

impl From<CreateV1Args> for CreateV2Args {
    fn from(item: CreateV1Args) -> Self {
        CreateV2Args {
            data_state: item.data_state,
            name: item.name,
            uri: item.uri,
            plugins: item.plugins,
            external_plugin_adapters: None,
        }
    }
}

pub(crate) fn create_v1<'a>(accounts: &'a [AccountInfo<'a>], args: CreateV1Args) -> ProgramResult {
    process_create(accounts, CreateV2Args::from(args))
}
pub(crate) fn create_v2<'a>(accounts: &'a [AccountInfo<'a>], args: CreateV2Args) -> ProgramResult {
    process_create(accounts, args)
}

pub(crate) fn process_create<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: CreateV2Args,
) -> ProgramResult {
    // Accounts.
    let ctx = CreateV2Accounts::context(accounts)?;
    let rent = Rent::get()?;

    // Guards.
    assert_signer(ctx.accounts.asset)?;
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    if *ctx.accounts.system_program.key != system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if let Some(log_wrapper) = ctx.accounts.log_wrapper {
        if log_wrapper.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    if ctx.accounts.update_authority.is_some() && ctx.accounts.collection.is_some() {
        return Err(MplCoreError::ConflictingAuthority.into());
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

    let lamports = rent.minimum_balance(serialized_data.len()) + get_create_fee()?;

    // CPI to the System Program.
    invoke(
        &system_instruction::create_account(
            ctx.accounts.payer.key,
            ctx.accounts.asset.key,
            lamports,
            serialized_data.len() as u64,
            &crate::ID,
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

    if args.data_state == DataState::AccountState {
        // Validate asset permissions.
        let _ = validate_asset_permissions(
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
            AssetV1::check_create,
            CollectionV1::check_create,
            PluginType::check_create,
            AssetV1::validate_create,
            CollectionV1::validate_create,
            Plugin::validate_create,
            Some(ExternalPluginAdapter::validate_create),
            Some(HookableLifecycleEvent::Create),
        )?;

        // Validate permissions for the created asset.
        let mut approved = true;
        let mut force_approved = false;

        if let Some(plugins) = args.plugins {
            if !plugins.is_empty() {
                let (header_offset, mut plugin_header, mut plugin_registry) =
                    create_plugin_meta::<AssetV1>(
                        &new_asset,
                        ctx.accounts.asset,
                        ctx.accounts.payer,
                        ctx.accounts.system_program,
                    )?;
                for plugin in &plugins {
                    // TODO move into plugin validation when asset/collection is part of validation context
                    let plugin_type = PluginType::from(&plugin.plugin);
                    if plugin_type == PluginType::MasterEdition
                        || plugin_type == PluginType::BubblegumV2
                    {
                        return Err(MplCoreError::InvalidPlugin.into());
                    }
                    if PluginType::check_create(&PluginType::from(&plugin.plugin))
                        != CheckResult::None
                    {
                        let validation_ctx = PluginValidationContext {
                            accounts,
                            asset_info: Some(ctx.accounts.asset),
                            collection_info: ctx.accounts.collection,
                            self_authority: &plugin.authority.unwrap_or(plugin.plugin.manager()),
                            authority_info: authority,
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
                    initialize_plugin::<AssetV1>(
                        &plugin.plugin,
                        &plugin.authority.unwrap_or(plugin.plugin.manager()),
                        header_offset,
                        &mut plugin_header,
                        &mut plugin_registry,
                        ctx.accounts.asset,
                        ctx.accounts.payer,
                        ctx.accounts.system_program,
                    )?;
                }
            }
        }

        if let Some(plugins) = args.external_plugin_adapters {
            if !plugins.is_empty() {
                let (_, header_offset, mut plugin_header, mut plugin_registry) =
                    create_meta_idempotent::<AssetV1>(
                        ctx.accounts.asset,
                        ctx.accounts.payer,
                        ctx.accounts.system_program,
                    )?;
                for plugin_init_info in &plugins {
                    let external_check_result_bits = ExternalCheckResultBits::from(
                        ExternalPluginAdapter::check_create(plugin_init_info),
                    );

                    // TODO: This should be handled in the validate call.
                    match plugin_init_info {
                        ExternalPluginAdapterInitInfo::LinkedLifecycleHook(_)
                        | ExternalPluginAdapterInitInfo::LinkedAppData(_) => {
                            return Err(MplCoreError::InvalidPluginAdapterTarget.into())
                        }
                        ExternalPluginAdapterInitInfo::DataSection(_) => {
                            return Err(MplCoreError::CannotAddDataSection.into())
                        }
                        _ => (),
                    }

                    if external_check_result_bits.can_reject() {
                        let validation_ctx = PluginValidationContext {
                            accounts,
                            asset_info: Some(ctx.accounts.asset),
                            collection_info: ctx.accounts.collection,
                            // External plugin adapters are always managed by the update authority.
                            self_authority: &Authority::UpdateAuthority,
                            authority_info: authority,
                            resolved_authorities: None,
                            new_owner: None,
                            new_asset_authority: None,
                            new_collection_authority: None,
                            target_plugin: None,
                            target_plugin_authority: None,
                            target_external_plugin: None,
                            target_external_plugin_authority: None,
                        };
                        if ExternalPluginAdapter::validate_create(
                            &ExternalPluginAdapter::from(plugin_init_info),
                            &validation_ctx,
                        )? == ValidationResult::Rejected
                        {
                            approved = false;
                        }
                    }
                    initialize_external_plugin_adapter::<AssetV1>(
                        plugin_init_info,
                        header_offset,
                        &mut plugin_header,
                        &mut plugin_registry,
                        ctx.accounts.asset,
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
    }

    if let Some(mut collection) = collection {
        collection.increment_minted()?;
        collection.increment_size()?;
        collection.save(ctx.accounts.collection.unwrap(), 0)?;
    };

    Ok(())
}
