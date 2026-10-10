mod account;
mod compression;

pub(crate) use account::*;
pub(crate) use compression::*;

use crate::{
    error::MplCoreError,
    plugins::{
        fetch_wrapped_plugin, validate_external_plugin_adapter_checks, validate_plugin_checks,
        CheckResult, ExternalCheckResultBits, ExternalPluginAdapter, ExternalPluginAdapterKey,
        ExternalRegistryRecord, HookableLifecycleEvent, Plugin, PluginHeaderV1, PluginRegistryV1,
        PluginType, PluginValidationContext, RegistryRecord, ValidationResult,
    },
    state::{
        AssetV1, Authority, CollectionV1, CoreAsset, DataBlob, GroupV1, Key, SolanaAccount,
        UpdateAuthority,
    },
};
use mpl_utils::assert_signer;
use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
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

/// Persist a mutated `GroupV1` as flat Borsh data only.
pub(crate) fn save_flat_group<'a>(
    group_info: &AccountInfo<'a>,
    group: &GroupV1,
    payer_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
) -> ProgramResult {
    let serialized_len = group.len();

    if serialized_len != group_info.data_len() {
        resize_or_reallocate_account(group_info, payer_info, system_program_info, serialized_len)?;
    }

    group.save(group_info, 0)?;

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

/// Returns true if the `authority_info` represents either the update authority of the asset
/// or a valid update delegate (defined by an `UpdateDelegate` plugin on the asset).
///
/// When the asset's update authority is `UpdateAuthority::Collection`, the signer is
/// validated against the collection's update authority (and its update delegates) by
/// locating the collection `AccountInfo` in `all_accounts`.  Callers must ensure the
/// collection account is present in the transaction when operating on collection-bound
/// assets.
pub fn is_valid_asset_authority<'a>(
    asset_info: &AccountInfo<'a>,
    authority_info: &AccountInfo<'a>,
    all_accounts: &'a [AccountInfo<'a>],
) -> Result<bool, ProgramError> {
    let asset_core = AssetV1::load(asset_info, 0)?;

    match &asset_core.update_authority {
        UpdateAuthority::Address(addr) => {
            if addr == authority_info.key {
                return Ok(true);
            }
        }
        UpdateAuthority::Collection(collection_addr) => {
            match all_accounts.iter().find(|a| a.key == collection_addr) {
                Some(collection_info) => {
                    if is_valid_collection_authority(collection_info, authority_info)? {
                        return Ok(true);
                    }
                }
                None => {
                    msg!(
                        "Asset has UpdateAuthority::Collection but the collection \
                         account {} was not provided in the transaction",
                        collection_addr
                    );
                }
            }
        }
        UpdateAuthority::None => {}
    }

    // Attempt to locate an UpdateDelegate plugin on the asset itself.
    match fetch_wrapped_plugin::<AssetV1>(asset_info, Some(&asset_core), PluginType::UpdateDelegate)
    {
        Ok((_plugin_authority, Plugin::UpdateDelegate(update_delegate))) => {
            if update_delegate
                .additional_delegates
                .contains(authority_info.key)
            {
                return Ok(true);
            }
        }
        Ok(_) => return Err(MplCoreError::InvalidPlugin.into()),
        Err(ProgramError::Custom(code))
            if code == MplCoreError::PluginNotFound as u32
                || code == MplCoreError::PluginsNotInitialized as u32 => {}
        Err(err) => return Err(err),
    }

    Ok(false)
}

/// Returns true if the `authority_info` represents the group's update authority.
pub fn is_valid_group_authority(
    group_info: &AccountInfo,
    authority_info: &AccountInfo,
) -> Result<bool, ProgramError> {
    let group_core = GroupV1::load(group_info, 0)?;
    Ok(authority_info.key == &group_core.update_authority)
}

/// Returns true if the `authority_info` represents either the update authority of the collection
/// or a valid update delegate (defined by an `UpdateDelegate` plugin on the collection).
pub fn is_valid_collection_authority(
    collection_info: &AccountInfo,
    authority_info: &AccountInfo,
) -> Result<bool, ProgramError> {
    let collection_core = CollectionV1::load(collection_info, 0)?;
    if authority_info.key == &collection_core.update_authority {
        return Ok(true);
    }

    // Attempt to locate an UpdateDelegate plugin on the collection.
    match fetch_wrapped_plugin::<CollectionV1>(
        collection_info,
        Some(&collection_core),
        PluginType::UpdateDelegate,
    ) {
        Ok((_plugin_authority, Plugin::UpdateDelegate(update_delegate))) => {
            if update_delegate
                .additional_delegates
                .contains(authority_info.key)
            {
                return Ok(true);
            }
        }
        Ok(_) => return Err(MplCoreError::InvalidPlugin.into()),
        Err(ProgramError::Custom(code))
            if code == MplCoreError::PluginNotFound as u32
                || code == MplCoreError::PluginsNotInitialized as u32 => {}
        Err(err) => return Err(err),
    }

    Ok(false)
}

/// Test-only scaffolding: an owned buffer plus the metadata needed to build an
/// `AccountInfo` over it, so that functions taking `&AccountInfo` can be unit
/// tested without a runtime. Shared by the unit tests in `state` and `utils`.
#[cfg(test)]
pub(crate) mod test_account {
    use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

    /// An account backed by a local buffer.
    pub(crate) struct TestAccount {
        key: Pubkey,
        owner: Pubkey,
        lamports: u64,
        data: Vec<u8>,
    }

    impl TestAccount {
        /// An account with an explicit key, owner and payload.
        pub(crate) fn new(key: Pubkey, owner: Pubkey, data: Vec<u8>) -> Self {
            Self {
                key,
                owner,
                lamports: 1,
                data,
            }
        }

        /// An account owned by this program, under a fresh key.
        pub(crate) fn owned(data: Vec<u8>) -> Self {
            Self::new(Pubkey::new_unique(), crate::ID, data)
        }

        /// A data-less account standing in for a signer with the given key.
        pub(crate) fn signer(key: Pubkey) -> Self {
            Self::new(key, solana_system_interface::program::ID, Vec::new())
        }

        /// A data-less account standing in for a signer under a fresh key.
        pub(crate) fn stranger() -> Self {
            Self::signer(Pubkey::new_unique())
        }

        /// The current contents of the data buffer.
        pub(crate) fn data(&self) -> &[u8] {
            &self.data
        }

        /// Borrow the account as an `AccountInfo`. The borrow is exclusive for
        /// the lifetime of the returned value, so scope it where the buffer has
        /// to be read back afterwards.
        pub(crate) fn info(&mut self) -> AccountInfo<'_> {
            AccountInfo::new(
                &self.key,
                false,
                true,
                &mut self.lamports,
                &mut self.data,
                &self.owner,
                false,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_account::TestAccount;
    use super::*;
    use crate::state::{AssetV1, CollectionV1, UpdateAuthority};

    fn invalid_authority() -> ProgramError {
        MplCoreError::InvalidAuthority.into()
    }

    fn asset(owner: Pubkey, update_authority: UpdateAuthority) -> AssetV1 {
        AssetV1::new(
            owner,
            update_authority,
            "name".to_string(),
            "uri".to_string(),
        )
    }

    fn collection(update_authority: Pubkey) -> CollectionV1 {
        CollectionV1::new(
            update_authority,
            "name".to_string(),
            "uri".to_string(),
            0,
            0,
        )
    }

    // ---------------------------------------------------------------------
    // `assert_authority`. This function has no callers anywhere in `src`
    // (roadmap section 13, finding 4); these tests pin its behaviour so that a
    // future caller cannot be wired up to something different by accident.
    // ---------------------------------------------------------------------

    #[test]
    fn assert_authority_owner_arm() {
        let owner = Pubkey::new_unique();
        let asset = asset(owner, UpdateAuthority::Address(Pubkey::new_unique()));

        let mut matching = TestAccount::signer(owner);
        assert_eq!(
            assert_authority(&asset, &matching.info(), &Authority::Owner),
            Ok(())
        );

        let mut other = TestAccount::stranger();
        assert_eq!(
            assert_authority(&asset, &other.info(), &Authority::Owner),
            Err(invalid_authority())
        );
    }

    #[test]
    fn assert_authority_update_authority_arm() {
        let update_authority = Pubkey::new_unique();
        let asset = asset(
            Pubkey::new_unique(),
            UpdateAuthority::Address(update_authority),
        );

        let mut matching = TestAccount::signer(update_authority);
        assert_eq!(
            assert_authority(&asset, &matching.info(), &Authority::UpdateAuthority),
            Ok(())
        );

        let mut other = TestAccount::stranger();
        assert_eq!(
            assert_authority(&asset, &other.info(), &Authority::UpdateAuthority),
            Err(invalid_authority())
        );
    }

    /// `UpdateAuthority::None.key()` is the system program id, so the
    /// `UpdateAuthority` arm would accept a system-program signer. That cannot
    /// happen on chain (the system program never signs), but the assertion
    /// documents the hazard (roadmap section 13, note 9).
    #[test]
    fn assert_authority_update_authority_none_is_system_program() {
        let asset = asset(Pubkey::new_unique(), UpdateAuthority::None);

        let mut system = TestAccount::signer(solana_system_interface::program::ID);
        assert_eq!(
            assert_authority(&asset, &system.info(), &Authority::UpdateAuthority),
            Ok(())
        );

        let mut other = TestAccount::stranger();
        assert_eq!(
            assert_authority(&asset, &other.info(), &Authority::UpdateAuthority),
            Err(invalid_authority())
        );
    }

    #[test]
    fn assert_authority_address_arm() {
        let delegate = Pubkey::new_unique();
        let asset = asset(Pubkey::new_unique(), UpdateAuthority::None);

        let mut matching = TestAccount::signer(delegate);
        assert_eq!(
            assert_authority(
                &asset,
                &matching.info(),
                &Authority::Address { address: delegate }
            ),
            Ok(())
        );

        let mut other = TestAccount::stranger();
        assert_eq!(
            assert_authority(
                &asset,
                &other.info(),
                &Authority::Address { address: delegate }
            ),
            Err(invalid_authority())
        );
    }

    /// `Authority::None` falls through to the error for every signer,
    /// including the owner and the update authority.
    #[test]
    fn assert_authority_none_arm_always_rejects() {
        let owner = Pubkey::new_unique();
        let update_authority = Pubkey::new_unique();
        let asset = asset(owner, UpdateAuthority::Address(update_authority));

        for key in [owner, update_authority, Pubkey::new_unique()] {
            let mut signer = TestAccount::signer(key);
            assert_eq!(
                assert_authority(&asset, &signer.info(), &Authority::None),
                Err(invalid_authority())
            );
        }
    }

    /// `CollectionV1::owner()` returns the update authority, so under
    /// `assert_authority` a collection's update authority satisfies an
    /// `Authority::Owner` record (roadmap section 13, note 4).
    #[test]
    fn assert_authority_collection_owner_is_update_authority() {
        let update_authority = Pubkey::new_unique();
        let collection = collection(update_authority);

        let mut ua = TestAccount::signer(update_authority);
        assert_eq!(
            assert_authority(&collection, &ua.info(), &Authority::Owner),
            Ok(())
        );
        let mut ua = TestAccount::signer(update_authority);
        assert_eq!(
            assert_authority(&collection, &ua.info(), &Authority::UpdateAuthority),
            Ok(())
        );

        let mut other = TestAccount::stranger();
        assert_eq!(
            assert_authority(&collection, &other.info(), &Authority::Owner),
            Err(invalid_authority())
        );
    }

    // ---------------------------------------------------------------------
    // `assert_collection_authority`. Single caller: `UpdateV2` when the new
    // collection carries an `UpdateDelegate` plugin (`processor/update.rs`).
    // ---------------------------------------------------------------------

    #[test]
    fn assert_collection_authority_update_authority_arm() {
        let update_authority = Pubkey::new_unique();
        let collection = collection(update_authority);

        let mut ua = TestAccount::signer(update_authority);
        assert_eq!(
            assert_collection_authority(&collection, &ua.info(), &Authority::UpdateAuthority),
            Ok(())
        );

        let mut other = TestAccount::stranger();
        assert_eq!(
            assert_collection_authority(&collection, &other.info(), &Authority::UpdateAuthority),
            Err(invalid_authority())
        );
    }

    #[test]
    fn assert_collection_authority_address_arm() {
        let delegate = Pubkey::new_unique();
        let collection = collection(Pubkey::new_unique());

        let mut matching = TestAccount::signer(delegate);
        assert_eq!(
            assert_collection_authority(
                &collection,
                &matching.info(),
                &Authority::Address { address: delegate }
            ),
            Ok(())
        );

        let mut other = TestAccount::stranger();
        assert_eq!(
            assert_collection_authority(
                &collection,
                &other.info(),
                &Authority::Address { address: delegate }
            ),
            Err(invalid_authority())
        );
    }

    /// Unlike `assert_authority`, the collection variant lumps `Owner` in with
    /// `None`: both fall through to `InvalidAuthority` even for the collection
    /// update authority.
    #[test]
    fn assert_collection_authority_none_and_owner_arms_always_reject() {
        let update_authority = Pubkey::new_unique();
        let collection = collection(update_authority);

        for authority in [Authority::None, Authority::Owner] {
            for key in [update_authority, Pubkey::new_unique()] {
                let mut signer = TestAccount::signer(key);
                assert_eq!(
                    assert_collection_authority(&collection, &signer.info(), &authority),
                    Err(invalid_authority())
                );
            }
        }
    }

    // ---------------------------------------------------------------------
    // `load_key`
    // ---------------------------------------------------------------------

    #[test]
    fn load_key_reads_the_discriminator_at_the_offset() {
        let mut account = TestAccount::owned(vec![
            Key::Uninitialized as u8,
            Key::AssetV1 as u8,
            Key::CollectionV1 as u8,
        ]);
        let info = account.info();

        assert_eq!(load_key(&info, 0), Ok(Key::Uninitialized));
        assert_eq!(load_key(&info, 1), Ok(Key::AssetV1));
        assert_eq!(load_key(&info, 2), Ok(Key::CollectionV1));
    }

    #[test]
    fn load_key_rejects_an_unknown_discriminator() {
        let mut account = TestAccount::owned(vec![u8::MAX]);
        let info = account.info();

        assert_eq!(
            load_key(&info, 0),
            Err(MplCoreError::DeserializationError.into())
        );
    }
}
