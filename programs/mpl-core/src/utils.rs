use std::collections::BTreeMap;

use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use num_traits::{FromPrimitive, ToPrimitive};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_error::ProgramError, program_memory::sol_memcpy, pubkey::Pubkey, rent::Rent,
    system_instruction, sysvar::Sysvar,
};

use crate::{
    error::MplCoreError,
    plugins::{
        create_meta_idempotent, initialize_plugin, validate_external_plugin_adapter_checks,
        validate_plugin_checks, CheckResult, ExternalCheckResultBits, ExternalPluginAdapter,
        ExternalPluginAdapterKey, ExternalRegistryRecord, HookableLifecycleEvent, Plugin,
        PluginHeaderV1, PluginRegistryV1, PluginType, PluginValidationContext, RegistryRecord,
        ValidationResult,
    },
    state::{
        AssetV1, Authority, CollectionV1, Compressible, CompressionProof, CoreAsset, DataBlob,
        HashablePluginSchema, HashedAssetSchema, HashedAssetV1, Key, SolanaAccount,
        UpdateAuthority,
    },
};

/// Load the one byte key from the account data at the given offset.
pub fn load_key(account: &AccountInfo, offset: usize) -> Result<Key, ProgramError> {
    let key =
        Key::from_u8((*account.data).borrow()[offset]).ok_or(MplCoreError::DeserializationError)?;

    Ok(key)
}

/// Assert that the account info address is in the same as the authority.
pub fn assert_authority<T: CoreAsset>(
    asset: &T,
    authority_info: &AccountInfo,
    authority: &Authority,
) -> ProgramResult {
    match authority {
        Authority::None => (),
        Authority::Owner => {
            if asset.owner() == authority_info.key {
                return Ok(());
            }
        }
        Authority::UpdateAuthority => {
            if asset.update_authority().key() == *authority_info.key {
                return Ok(());
            }
        }
        Authority::Address { address } => {
            if authority_info.key == address {
                return Ok(());
            }
        }
    }

    Err(MplCoreError::InvalidAuthority.into())
}

/// Assert that the account info address is the same as the authority.
pub fn assert_collection_authority(
    collection: &CollectionV1,
    authority_info: &AccountInfo,
    authority: &Authority,
) -> ProgramResult {
    match authority {
        Authority::None | Authority::Owner => (),
        Authority::UpdateAuthority => {
            if &collection.update_authority == authority_info.key {
                return Ok(());
            }
        }
        Authority::Address { address } => {
            if authority_info.key == address {
                return Ok(());
            }
        }
    }

    Err(MplCoreError::InvalidAuthority.into())
}

/// Fetch the core data from the account; asset, plugin header (if present), and plugin registry (if present).
pub fn fetch_core_data<T: DataBlob + SolanaAccount>(
    account: &AccountInfo,
) -> Result<(T, Option<PluginHeaderV1>, Option<PluginRegistryV1>), ProgramError> {
    let asset = T::load(account, 0)?;

    if asset.get_size() != account.data_len() {
        let plugin_header = PluginHeaderV1::load(account, asset.get_size())?;
        let plugin_registry =
            PluginRegistryV1::load(account, plugin_header.plugin_registry_offset)?;

        Ok((asset, Some(plugin_header), Some(plugin_registry)))
    } else {
        Ok((asset, None, None))
    }
}

/// Check that a compression proof results in same on-chain hash.
pub fn verify_proof(
    hashed_asset: &AccountInfo,
    compression_proof: &CompressionProof,
) -> Result<(AssetV1, Vec<HashablePluginSchema>), ProgramError> {
    let asset = AssetV1::from(compression_proof.clone());
    let asset_hash = asset.hash()?;

    let mut sorted_plugins = compression_proof.plugins.clone();
    sorted_plugins.sort_by(HashablePluginSchema::compare_indeces);

    let plugin_hashes = sorted_plugins
        .iter()
        .map(|plugin| plugin.hash())
        .collect::<Result<Vec<[u8; 32]>, ProgramError>>()?;

    let hashed_asset_schema = HashedAssetSchema {
        asset_hash,
        plugin_hashes,
    };

    let hashed_asset_schema_hash = hashed_asset_schema.hash()?;

    let current_account_hash = HashedAssetV1::load(hashed_asset, 0)?.hash;
    if hashed_asset_schema_hash != current_account_hash {
        return Err(MplCoreError::IncorrectAssetHash.into());
    }

    Ok((asset, sorted_plugins))
}

pub(crate) fn close_program_account<'a>(
    account_to_close_info: &AccountInfo<'a>,
    funds_dest_account_info: &AccountInfo<'a>,
) -> ProgramResult {
    let rent = Rent::get()?;

    let account_size = account_to_close_info.data_len();
    let account_rent = rent.minimum_balance(account_size);
    let one_byte_rent = rent.minimum_balance(1);

    let amount_to_return = account_rent
        .checked_sub(one_byte_rent)
        .ok_or(MplCoreError::NumericalOverflowError)?;

    // Transfer lamports from the account to the destination account.
    let dest_starting_lamports = funds_dest_account_info.lamports();
    **funds_dest_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(amount_to_return)
        .ok_or(MplCoreError::NumericalOverflowError)?;
    **account_to_close_info.try_borrow_mut_lamports()? -= amount_to_return;

    account_to_close_info.realloc(1, false)?;
    account_to_close_info.data.borrow_mut()[0] = Key::Uninitialized.to_u8().unwrap();

    Ok(())
}

/// Resize an account using realloc and retain any lamport overages, modified from Solana Cookbook
pub(crate) fn resize_or_reallocate_account<'a>(
    target_account: &AccountInfo<'a>,
    funding_account: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    new_size: usize,
) -> ProgramResult {
    // If the account is already the correct size, return.
    if new_size == target_account.data_len() {
        return Ok(());
    }

    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(new_size);
    let current_minimum_balance = rent.minimum_balance(target_account.data_len());
    let account_infos = &[
        funding_account.clone(),
        target_account.clone(),
        system_program.clone(),
    ];

    if new_minimum_balance >= current_minimum_balance {
        let lamports_diff = new_minimum_balance.saturating_sub(current_minimum_balance);
        invoke(
            &system_instruction::transfer(funding_account.key, target_account.key, lamports_diff),
            account_infos,
        )?;
    } else {
        // return lamports to the compressor
        let lamports_diff = current_minimum_balance.saturating_sub(new_minimum_balance);

        **funding_account.try_borrow_mut_lamports()? += lamports_diff;
        **target_account.try_borrow_mut_lamports()? -= lamports_diff
    }

    target_account.realloc(new_size, false)?;

    Ok(())
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
/// Validate asset permissions using lifecycle validations for asset, collection, and plugins.
pub(crate) fn validate_asset_permissions<'a>(
    accounts: &'a [AccountInfo<'a>],
    authority_info: &'a AccountInfo<'a>,
    asset: &'a AccountInfo<'a>,
    collection: Option<&'a AccountInfo<'a>>,
    new_owner: Option<&'a AccountInfo<'a>>,
    new_authority: Option<&UpdateAuthority>,
    new_plugin: Option<&Plugin>,
    new_external_plugin_adapter: Option<&ExternalPluginAdapter>,
    asset_check_fp: fn() -> CheckResult,
    collection_check_fp: fn() -> CheckResult,
    plugin_check_fp: fn(&PluginType) -> CheckResult,
    asset_validate_fp: fn(
        &AssetV1,
        &AccountInfo,
        Option<&Plugin>,
        Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError>,
    collection_validate_fp: fn(
        &CollectionV1,
        &AccountInfo,
        Option<&Plugin>,
        Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError>,
    plugin_validate_fp: fn(
        &Plugin,
        &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError>,
    external_plugin_adapter_validate_fp: Option<
        fn(
            &ExternalPluginAdapter,
            &PluginValidationContext,
        ) -> Result<ValidationResult, ProgramError>,
    >,
    hookable_lifecycle_event: Option<HookableLifecycleEvent>,
) -> Result<(AssetV1, Option<PluginHeaderV1>, Option<PluginRegistryV1>), ProgramError> {
    if external_plugin_adapter_validate_fp.is_some() && hookable_lifecycle_event.is_none()
        || external_plugin_adapter_validate_fp.is_none() && hookable_lifecycle_event.is_some()
    {
        panic!("Missing function parameters to validate_asset_permissions");
    }

    let (deserialized_asset, plugin_header, plugin_registry) = fetch_core_data::<AssetV1>(asset)?;
    let resolved_authorities =
        resolve_pubkey_to_authorities(authority_info, collection, &deserialized_asset)?;

    // If the asset is part of a collection, the collection must be passed in and it must be correct.
    if let UpdateAuthority::Collection(collection_address) = deserialized_asset.update_authority {
        if collection.is_none() {
            return Err(MplCoreError::MissingCollection.into());
        } else if collection.unwrap().key != &collection_address {
            return Err(MplCoreError::InvalidCollection.into());
        }
    } else if collection.is_some() {
        return Err(MplCoreError::InvalidCollection.into());
    }

    let mut checks: BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)> = BTreeMap::new();
    let mut external_checks: BTreeMap<
        ExternalPluginAdapterKey,
        (Key, ExternalCheckResultBits, ExternalRegistryRecord),
    > = BTreeMap::new();

    // The asset approval overrides the collection approval.
    let asset_check = asset_check_fp();
    let collection_check = if collection.is_some() {
        collection_check_fp()
    } else {
        CheckResult::None
    };

    // Check the collection plugins first.
    if let Some(collection_info) = collection {
        let (_, _, registry) = fetch_core_data::<CollectionV1>(collection_info)?;

        if let Some(r) = registry {
            r.check_registry(Key::CollectionV1, plugin_check_fp, &mut checks);

            if let Some(lifecycle_event) = &hookable_lifecycle_event {
                r.check_adapter_registry(
                    collection_info,
                    Key::CollectionV1,
                    lifecycle_event,
                    &mut external_checks,
                )?;
            }
        }
    }

    // Next check the asset plugins. Plugins on the asset override the collection plugins,
    // so we don't need to validate the collection plugins if the asset has a plugin.
    if let Some(registry) = plugin_registry.as_ref() {
        registry.check_registry(Key::AssetV1, plugin_check_fp, &mut checks);
        if let Some(lifecycle_event) = &hookable_lifecycle_event {
            registry.check_adapter_registry(
                asset,
                Key::AssetV1,
                lifecycle_event,
                &mut external_checks,
            )?;
        }
    }

    // Do the core validation.
    let mut approved = false;
    let mut rejected = false;
    if asset_check != CheckResult::None {
        match asset_validate_fp(
            &deserialized_asset,
            authority_info,
            new_plugin,
            new_external_plugin_adapter,
        )? {
            ValidationResult::Approved => approved = true,
            ValidationResult::Rejected => rejected = true,
            ValidationResult::Pass => (),
            ValidationResult::ForceApproved => {
                return Ok((deserialized_asset, plugin_header, plugin_registry))
            }
        }
    };

    if collection_check != CheckResult::None {
        match collection_validate_fp(
            &CollectionV1::load(collection.ok_or(MplCoreError::MissingCollection)?, 0)?,
            authority_info,
            new_plugin,
            new_external_plugin_adapter,
        )? {
            ValidationResult::Approved => approved = true,
            ValidationResult::Rejected => rejected = true,
            ValidationResult::Pass => (),
            ValidationResult::ForceApproved => {
                return Ok((deserialized_asset, plugin_header, plugin_registry))
            }
        }
    };

    match validate_plugin_checks(
        Key::CollectionV1,
        accounts,
        &checks,
        authority_info,
        new_owner,
        new_authority,
        None,
        new_plugin,
        Some(asset),
        collection,
        &resolved_authorities,
        plugin_validate_fp,
    )? {
        ValidationResult::Approved => approved = true,
        ValidationResult::Rejected => rejected = true,
        ValidationResult::Pass => (),
        ValidationResult::ForceApproved => {
            return Ok((deserialized_asset, plugin_header, plugin_registry))
        }
    };

    match validate_plugin_checks(
        Key::AssetV1,
        accounts,
        &checks,
        authority_info,
        new_owner,
        new_authority,
        None,
        new_plugin,
        Some(asset),
        collection,
        &resolved_authorities,
        plugin_validate_fp,
    )? {
        ValidationResult::Approved => approved = true,
        ValidationResult::Rejected => rejected = true,
        ValidationResult::Pass => (),
        ValidationResult::ForceApproved => {
            return Ok((deserialized_asset, plugin_header, plugin_registry))
        }
    };

    if let Some(external_plugin_adapter_validate_fp) = external_plugin_adapter_validate_fp {
        match validate_external_plugin_adapter_checks(
            Key::CollectionV1,
            accounts,
            &external_checks,
            authority_info,
            new_owner,
            new_authority,
            None,
            new_plugin,
            Some(asset),
            collection,
            &resolved_authorities,
            external_plugin_adapter_validate_fp,
        )? {
            ValidationResult::Approved => approved = true,
            ValidationResult::Rejected => rejected = true,
            ValidationResult::Pass => (),
            // Force approved will not be possible from external plugin adapters.
            ValidationResult::ForceApproved => unreachable!(),
        };

        match validate_external_plugin_adapter_checks(
            Key::AssetV1,
            accounts,
            &external_checks,
            authority_info,
            new_owner,
            new_authority,
            None,
            new_plugin,
            Some(asset),
            collection,
            &resolved_authorities,
            external_plugin_adapter_validate_fp,
        )? {
            ValidationResult::Approved => approved = true,
            ValidationResult::Rejected => rejected = true,
            ValidationResult::Pass => (),
            // Force approved will not be possible from external plugin adapters.
            ValidationResult::ForceApproved => unreachable!(),
        };
    }

    if rejected {
        return Err(MplCoreError::InvalidAuthority.into());
    } else if !approved {
        return Err(MplCoreError::NoApprovals.into());
    }

    Ok((deserialized_asset, plugin_header, plugin_registry))
}

/// Validate collection permissions using lifecycle validations for collection and plugins.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn validate_collection_permissions<'a>(
    accounts: &'a [AccountInfo<'a>],
    authority_info: &'a AccountInfo<'a>,
    collection: &'a AccountInfo<'a>,
    new_authority: Option<&Pubkey>,
    new_plugin: Option<&Plugin>,
    new_external_plugin_adapter: Option<&ExternalPluginAdapter>,
    collection_check_fp: fn() -> CheckResult,
    plugin_check_fp: fn(&PluginType) -> CheckResult,
    collection_validate_fp: fn(
        &CollectionV1,
        &AccountInfo,
        Option<&Plugin>,
        Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError>,
    plugin_validate_fp: fn(
        &Plugin,
        &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError>,
    external_plugin_adapter_validate_fp: Option<
        fn(
            &ExternalPluginAdapter,
            &PluginValidationContext,
        ) -> Result<ValidationResult, ProgramError>,
    >,
    hookable_lifecycle_event: Option<HookableLifecycleEvent>,
) -> Result<
    (
        CollectionV1,
        Option<PluginHeaderV1>,
        Option<PluginRegistryV1>,
    ),
    ProgramError,
> {
    if external_plugin_adapter_validate_fp.is_some() && hookable_lifecycle_event.is_none()
        || external_plugin_adapter_validate_fp.is_none() && hookable_lifecycle_event.is_some()
    {
        panic!("Missing function parameters to validate_asset_permissions");
    }

    let (deserialized_collection, plugin_header, plugin_registry) =
        fetch_core_data::<CollectionV1>(collection)?;
    let resolved_authorities =
        resolve_pubkey_to_authorities_collection(authority_info, collection)?;
    let mut checks: BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)> = BTreeMap::new();
    let mut external_checks: BTreeMap<
        ExternalPluginAdapterKey,
        (Key, ExternalCheckResultBits, ExternalRegistryRecord),
    > = BTreeMap::new();

    let core_check = (Key::CollectionV1, collection_check_fp());

    // Check the collection plugins.
    if let Some(registry) = plugin_registry.as_ref() {
        registry.check_registry(Key::CollectionV1, plugin_check_fp, &mut checks);
        if let Some(lifecycle_event) = hookable_lifecycle_event {
            registry.check_adapter_registry(
                collection,
                Key::CollectionV1,
                &lifecycle_event,
                &mut external_checks,
            )?;
        }
    }

    // Do the core validation.
    let mut approved = false;
    let mut rejected = false;
    if matches!(
        core_check,
        (
            Key::CollectionV1,
            CheckResult::CanApprove | CheckResult::CanReject | CheckResult::CanForceApprove
        )
    ) {
        let result = match core_check.0 {
            Key::CollectionV1 => collection_validate_fp(
                &deserialized_collection,
                authority_info,
                new_plugin,
                new_external_plugin_adapter,
            )?,
            _ => return Err(MplCoreError::IncorrectAccount.into()),
        };
        match result {
            ValidationResult::Approved => approved = true,
            ValidationResult::Rejected => rejected = true,
            ValidationResult::Pass => (),
            ValidationResult::ForceApproved => {
                return Ok((deserialized_collection, plugin_header, plugin_registry))
            }
        }
    };

    match validate_plugin_checks(
        Key::CollectionV1,
        accounts,
        &checks,
        authority_info,
        None,
        None,
        new_authority,
        new_plugin,
        None,
        Some(collection),
        &resolved_authorities,
        plugin_validate_fp,
    )? {
        ValidationResult::Approved => approved = true,
        ValidationResult::Rejected => rejected = true,
        ValidationResult::Pass => (),
        ValidationResult::ForceApproved => {
            return Ok((deserialized_collection, plugin_header, plugin_registry))
        }
    };

    if let Some(external_plugin_adapter_validate_fp) = external_plugin_adapter_validate_fp {
        match validate_external_plugin_adapter_checks(
            Key::CollectionV1,
            accounts,
            &external_checks,
            authority_info,
            None,
            None,
            new_authority,
            new_plugin,
            None,
            Some(collection),
            &resolved_authorities,
            external_plugin_adapter_validate_fp,
        )? {
            ValidationResult::Approved => approved = true,
            ValidationResult::Rejected => rejected = true,
            ValidationResult::Pass => (),
            // Force approved will not be possible from external plugin adapters.
            ValidationResult::ForceApproved => unreachable!(),
        };
    }

    if rejected || !approved {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    Ok((deserialized_collection, plugin_header, plugin_registry))
}

/// Take an `Asset` and Vec of `HashablePluginSchema` and rebuild the asset in account space.
pub fn rebuild_account_state_from_proof_data<'a>(
    asset: AssetV1,
    plugins: Vec<HashablePluginSchema>,
    asset_info: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let serialized_data = asset.try_to_vec()?;
    resize_or_reallocate_account(asset_info, payer, system_program, serialized_data.len())?;

    sol_memcpy(
        &mut asset_info.try_borrow_mut_data()?,
        &serialized_data,
        serialized_data.len(),
    );

    // Add the plugins.
    if !plugins.is_empty() {
        let (_, mut plugin_header, mut plugin_registry) =
            create_meta_idempotent::<AssetV1>(asset_info, payer, system_program)?;

        for plugin in plugins {
            initialize_plugin::<AssetV1>(
                &plugin.plugin,
                &plugin.authority,
                &mut plugin_header,
                &mut plugin_registry,
                asset_info,
                payer,
                system_program,
            )?;
        }
    }

    Ok(())
}

/// Take `Asset` and `PluginRegistry` for a decompressed asset, and compress into account space.
pub fn compress_into_account_space<'a>(
    mut asset: AssetV1,
    plugin_registry: Option<PluginRegistryV1>,
    asset_info: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> Result<CompressionProof, ProgramError> {
    // Initialize or increment the sequence number when compressing.
    let seq = asset.seq.unwrap_or(0).saturating_add(1);
    asset.seq = Some(seq);

    let asset_hash = asset.hash()?;
    let mut compression_proof = CompressionProof::new(asset, seq, vec![]);
    let mut plugin_hashes = vec![];
    if let Some(plugin_registry) = plugin_registry {
        let mut registry_records = plugin_registry.registry;

        // It should already be sorted but we just want to make sure.
        registry_records.sort_by(RegistryRecord::compare_offsets);

        for (i, record) in registry_records.into_iter().enumerate() {
            let plugin = Plugin::deserialize(&mut &(*asset_info.data).borrow()[record.offset..])?;

            let hashable_plugin_schema = HashablePluginSchema {
                index: i,
                authority: record.authority,
                plugin,
            };

            let plugin_hash = hashable_plugin_schema.hash()?;
            plugin_hashes.push(plugin_hash);

            compression_proof.plugins.push(hashable_plugin_schema);
        }
    }

    let hashed_asset_schema = HashedAssetSchema {
        asset_hash,
        plugin_hashes,
    };

    let hashed_asset = HashedAssetV1::new(hashed_asset_schema.hash()?);
    let serialized_data = hashed_asset.try_to_vec()?;

    resize_or_reallocate_account(asset_info, payer, system_program, serialized_data.len())?;

    sol_memcpy(
        &mut asset_info.try_borrow_mut_data()?,
        &serialized_data,
        serialized_data.len(),
    );

    Ok(compression_proof)
}

pub(crate) fn resolve_pubkey_to_authorities(
    authority_info: &AccountInfo,
    maybe_collection_info: Option<&AccountInfo>,
    asset: &AssetV1,
) -> Result<Vec<Authority>, ProgramError> {
    let mut authorities = Vec::with_capacity(3);
    if authority_info.key == &asset.owner {
        authorities.push(Authority::Owner);
    }

    if asset.update_authority == UpdateAuthority::Address(*authority_info.key) {
        authorities.push(Authority::UpdateAuthority);
    } else if let UpdateAuthority::Collection(collection_address) = asset.update_authority {
        match maybe_collection_info {
            Some(collection_info) => {
                if collection_info.key != &collection_address {
                    return Err(MplCoreError::InvalidCollection.into());
                }
                let collection: CollectionV1 = CollectionV1::load(collection_info, 0)?;
                if authority_info.key == &collection.update_authority {
                    authorities.push(Authority::UpdateAuthority);
                }
            }
            None => return Err(MplCoreError::MissingCollection.into()),
        }
    }

    authorities.push(Authority::Address {
        address: *authority_info.key,
    });

    Ok(authorities)
}

pub(crate) fn resolve_pubkey_to_authorities_collection(
    authority_info: &AccountInfo,
    collection_info: &AccountInfo,
) -> Result<Vec<Authority>, ProgramError> {
    let collection: CollectionV1 = CollectionV1::load(collection_info, 0)?;
    let mut authorities = Vec::with_capacity(3);
    if authority_info.key == collection.owner() {
        authorities.push(Authority::Owner);
    }

    if authority_info.key == &collection.update_authority {
        authorities.push(Authority::UpdateAuthority)
    }

    authorities.push(Authority::Address {
        address: *authority_info.key,
    });

    Ok(authorities)
}

/// Resolves the authority for the transaction for an optional authority pattern.
pub(crate) fn resolve_authority<'a>(
    payer: &'a AccountInfo<'a>,
    authority: Option<&'a AccountInfo<'a>>,
) -> Result<&'a AccountInfo<'a>, ProgramError> {
    match authority {
        Some(authority) => {
            assert_signer(authority)?;
            Ok(authority)
        }
        None => Ok(payer),
    }
}
