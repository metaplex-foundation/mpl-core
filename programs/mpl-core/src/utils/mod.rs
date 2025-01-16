mod account;
mod compression;

pub(crate) use account::*;
pub(crate) use compression::*;

use crate::{
    error::MplCoreError,
    plugins::{
        validate_external_plugin_adapter_checks, validate_plugin_checks, CheckResult,
        ExternalCheckResultBits, ExternalPluginAdapter, ExternalPluginAdapterKey,
        ExternalRegistryRecord, HookableLifecycleEvent, Plugin, PluginHeaderV1, PluginRegistryV1,
        PluginType, PluginValidationContext, RegistryRecord, ValidationResult,
    },
    state::{
        AssetV1, Authority, CollectionV1, CoreAsset, DataBlob, Key, SolanaAccount, UpdateAuthority,
    },
};
use mpl_utils::assert_signer;
use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use std::collections::BTreeMap;

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

    if asset.len() != account.data_len() {
        let plugin_header = PluginHeaderV1::load(account, asset.len())?;
        let plugin_registry =
            PluginRegistryV1::load(account, plugin_header.plugin_registry_offset)?;

        Ok((asset, Some(plugin_header), Some(plugin_registry)))
    } else {
        Ok((asset, None, None))
    }
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
    new_plugin_authority: Option<&Authority>,
    new_external_plugin_adapter: Option<&ExternalPluginAdapter>,
    new_external_plugin_adapter_authority: Option<&Authority>,
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
        new_plugin_authority,
        new_external_plugin_adapter,
        new_external_plugin_adapter_authority,
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
        new_plugin_authority,
        new_external_plugin_adapter,
        new_external_plugin_adapter_authority,
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
            new_plugin_authority,
            new_external_plugin_adapter,
            new_external_plugin_adapter_authority,
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
            new_plugin_authority,
            new_external_plugin_adapter,
            new_external_plugin_adapter_authority,
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
    new_plugin_authority: Option<&Authority>,
    new_external_plugin_adapter: Option<&ExternalPluginAdapter>,
    new_external_plugin_adapter_authority: Option<&Authority>,
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
        new_plugin_authority,
        new_external_plugin_adapter,
        new_external_plugin_adapter_authority,
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
            new_plugin_authority,
            new_external_plugin_adapter,
            new_external_plugin_adapter_authority,
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
