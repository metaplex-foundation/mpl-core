use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    plugins::{abstain, approve, CheckResult, ExternalPluginAdapter, Plugin, ValidationResult},
};

use super::{Authority, CoreAsset, DataBlob, Key, SolanaAccount, UpdateAuthority};

/// The representation of a collection of assets.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct CollectionV1 {
    /// The account discriminator.
    pub key: Key, //1
    /// The update authority of the collection.
    pub update_authority: Pubkey, //32
    /// The name of the collection.
    pub name: String, //4
    /// The URI that links to what data to show for the collection.
    pub uri: String, //4
    /// The number of assets minted in the collection.
    pub num_minted: u32, //4
    /// The number of assets currently in the collection.
    pub current_size: u32, //4
}

impl CollectionV1 {
    /// The base length of the collection account with an empty name and uri.
    const BASE_LEN: usize = 1 // Key
                            + 32 // Update Authority
                            + 4 // Name Length
                            + 4 // URI Length
                            + 4 // num_minted
                            + 4; // current_size

    /// Create a new collection.
    pub fn new(
        update_authority: Pubkey,
        name: String,
        uri: String,
        num_minted: u32,
        current_size: u32,
    ) -> Self {
        Self {
            key: Key::CollectionV1,
            update_authority,
            name,
            uri,
            num_minted,
            current_size,
        }
    }

    /// Check permissions for the create lifecycle event.
    pub fn check_create() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the add plugin lifecycle event.
    pub fn check_add_plugin() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the remove plugin lifecycle event.
    pub fn check_remove_plugin() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the update plugin lifecycle event.
    pub fn check_update_plugin() -> CheckResult {
        CheckResult::None
    }

    /// Check permissions for the approve plugin authority lifecycle event.
    pub fn check_approve_plugin_authority() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the revoke plugin authority lifecycle event.
    pub fn check_revoke_plugin_authority() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the transfer lifecycle event.
    pub fn check_transfer() -> CheckResult {
        CheckResult::None
    }

    /// Check permissions for the burn lifecycle event.
    pub fn check_burn() -> CheckResult {
        CheckResult::None
    }

    /// Check permissions for the update lifecycle event.
    pub fn check_update() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the compress lifecycle event.
    pub fn check_compress() -> CheckResult {
        CheckResult::None
    }

    /// Check permissions for the decompress lifecycle event.
    pub fn check_decompress() -> CheckResult {
        CheckResult::None
    }

    /// Check permissions for the add external plugin adapter lifecycle event.
    pub fn check_add_external_plugin_adapter() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the remove external plugin adapter lifecycle event.
    pub fn check_remove_external_plugin_adapter() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the update external plugin adapter lifecycle event.
    pub fn check_update_external_plugin_adapter() -> CheckResult {
        CheckResult::None
    }

    /// Check permissions for the execute lifecycle event.
    pub fn check_execute() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Validate the create lifecycle event.
    pub fn validate_create(
        &self,
        authority_info: &AccountInfo,
        _new_plugin: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority_info.key == &self.update_authority {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the add plugin lifecycle event.
    pub fn validate_add_plugin(
        &self,
        authority_info: &AccountInfo,
        new_plugin: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        let new_plugin = match new_plugin {
            Some(plugin) => plugin,
            None => return Err(MplCoreError::InvalidPlugin.into()),
        };

        if *authority_info.key == self.update_authority
            && new_plugin.manager() == Authority::UpdateAuthority
        {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the remove plugin lifecycle event.
    pub fn validate_remove_plugin(
        &self,
        authority_info: &AccountInfo,
        plugin_to_remove: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin_to_remove = match plugin_to_remove {
            Some(plugin) => plugin,
            None => return Err(MplCoreError::InvalidPlugin.into()),
        };

        if *authority_info.key == self.update_authority
            && plugin_to_remove.manager() == Authority::UpdateAuthority
        {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the update plugin lifecycle event.
    pub fn validate_update_plugin(
        &self,
        _authority_info: &AccountInfo,
        _plugin: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the approve plugin authority lifecycle event.
    pub fn validate_approve_plugin_authority(
        &self,
        authority_info: &AccountInfo,
        plugin: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin = match plugin {
            Some(plugin) => plugin,
            None => return Err(MplCoreError::InvalidPlugin.into()),
        };

        if *authority_info.key == self.update_authority
            && plugin.manager() == Authority::UpdateAuthority
        {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the revoke plugin authority lifecycle event.
    pub fn validate_revoke_plugin_authority(
        &self,
        authority_info: &AccountInfo,
        plugin: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin = match plugin {
            Some(plugin) => plugin,
            None => return Err(MplCoreError::InvalidPlugin.into()),
        };

        if *authority_info.key == self.update_authority
            && plugin.manager() == Authority::UpdateAuthority
        {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the transfer lifecycle event.
    pub fn validate_transfer(
        &self,
        _authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the burn lifecycle event.
    pub fn validate_burn(
        &self,
        _: &AccountInfo,
        _: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the update lifecycle event.
    pub fn validate_update(
        &self,
        authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority_info.key == &self.update_authority {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the compress lifecycle event.
    pub fn validate_compress(
        &self,
        _authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the decompress lifecycle event.
    pub fn validate_decompress(
        &self,
        _authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the execute lifecycle event.
    pub fn validate_execute(
        &self,
        _authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the add external plugin adapter lifecycle event.
    pub fn validate_add_external_plugin_adapter(
        &self,
        authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _new_plugin: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        // Approve if the update authority matches the authority.
        if *authority_info.key == self.update_authority {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the remove external plugin adapter lifecycle event.
    pub fn validate_remove_external_plugin_adapter(
        &self,
        authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _plugin_to_remove: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        if self.update_authority == *authority_info.key {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the update external plugin adapter lifecycle event.
    pub fn validate_update_external_plugin_adapter(
        &self,
        _authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _plugin: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Increment number of minted items of the Collection
    pub fn increment_minted(&mut self) -> Result<(), ProgramError> {
        self.num_minted = self
            .num_minted
            .checked_add(1)
            .ok_or(MplCoreError::NumericalOverflowError)?;

        Ok(())
    }

    /// Increment current size of the Collection
    pub fn increment_size(&mut self) -> Result<(), ProgramError> {
        self.current_size = self
            .current_size
            .checked_add(1)
            .ok_or(MplCoreError::NumericalOverflowError)?;

        Ok(())
    }

    /// Decrement current size of the Collection
    pub fn decrement_size(&mut self) -> Result<(), ProgramError> {
        self.current_size = self
            .current_size
            .checked_sub(1)
            .ok_or(MplCoreError::NumericalOverflowError)?;

        Ok(())
    }
}

impl DataBlob for CollectionV1 {
    fn len(&self) -> usize {
        Self::BASE_LEN + self.name.len() + self.uri.len()
    }
}

impl SolanaAccount for CollectionV1 {
    fn key() -> Key {
        Key::CollectionV1
    }
}

impl CoreAsset for CollectionV1 {
    fn update_authority(&self) -> UpdateAuthority {
        UpdateAuthority::Collection(self.update_authority)
    }

    fn owner(&self) -> &Pubkey {
        &self.update_authority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_len() {
        let collections = vec![
            CollectionV1 {
                key: Key::CollectionV1,
                update_authority: Pubkey::default(),
                name: "".to_string(),
                uri: "".to_string(),
                num_minted: 0,
                current_size: 0,
            },
            CollectionV1 {
                key: Key::CollectionV1,
                update_authority: Pubkey::default(),
                name: "test".to_string(),
                uri: "test".to_string(),
                num_minted: 1,
                current_size: 1,
            },
        ];
        for collection in collections {
            let serialized = borsh::to_vec(&collection).unwrap();
            assert_eq!(serialized.len(), collection.len());
        }
    }

    use crate::{
        plugins::{Attributes, FreezeDelegate},
        utils::test_account::TestAccount,
    };

    fn collection(update_authority: Pubkey) -> CollectionV1 {
        CollectionV1::new(
            update_authority,
            "name".to_string(),
            "uri".to_string(),
            0,
            0,
        )
    }

    /// An authority-managed plugin: the collection update authority manages it.
    fn ua_managed() -> Plugin {
        Plugin::Attributes(Attributes::new())
    }

    /// An owner-managed plugin. Collections have no owner, so the collection
    /// validators never approve one.
    fn owner_managed() -> Plugin {
        Plugin::FreezeDelegate(FreezeDelegate { frozen: false })
    }

    fn overflow() -> ProgramError {
        MplCoreError::NumericalOverflowError.into()
    }

    fn invalid_plugin() -> ProgramError {
        MplCoreError::InvalidPlugin.into()
    }

    // ---------------------------------------------------------------------
    // Counters
    // ---------------------------------------------------------------------

    #[test]
    fn increment_minted_counts_up() {
        let mut collection = collection(Pubkey::new_unique());

        assert_eq!(collection.increment_minted(), Ok(()));
        assert_eq!(collection.num_minted, 1);
        assert_eq!(collection.increment_minted(), Ok(()));
        assert_eq!(collection.num_minted, 2);
        // `num_minted` is independent of `current_size`.
        assert_eq!(collection.current_size, 0);
    }

    #[test]
    fn increment_minted_overflows_at_u32_max() {
        let mut collection = collection(Pubkey::new_unique());
        collection.num_minted = u32::MAX;

        assert_eq!(collection.increment_minted(), Err(overflow()));
        assert_eq!(collection.num_minted, u32::MAX);
    }

    #[test]
    fn increment_size_counts_up() {
        let mut collection = collection(Pubkey::new_unique());

        assert_eq!(collection.increment_size(), Ok(()));
        assert_eq!(collection.current_size, 1);
        assert_eq!(collection.num_minted, 0);
    }

    #[test]
    fn increment_size_overflows_at_u32_max() {
        let mut collection = collection(Pubkey::new_unique());
        collection.current_size = u32::MAX;

        assert_eq!(collection.increment_size(), Err(overflow()));
        assert_eq!(collection.current_size, u32::MAX);
    }

    #[test]
    fn decrement_size_counts_down() {
        let mut collection = collection(Pubkey::new_unique());
        collection.current_size = 2;

        assert_eq!(collection.decrement_size(), Ok(()));
        assert_eq!(collection.current_size, 1);
        assert_eq!(collection.decrement_size(), Ok(()));
        assert_eq!(collection.current_size, 0);
    }

    /// A collection whose `current_size` reached zero while assets still
    /// reference it cannot burn or un-collection those assets: every
    /// `decrement_size` fails with `NumericalOverflowError` (roadmap section
    /// 13, note 7).
    #[test]
    fn decrement_size_underflows_at_zero() {
        let mut collection = collection(Pubkey::new_unique());

        assert_eq!(collection.current_size, 0);
        assert_eq!(collection.decrement_size(), Err(overflow()));
        assert_eq!(collection.current_size, 0);
    }

    // ---------------------------------------------------------------------
    // `CoreAsset`
    // ---------------------------------------------------------------------

    /// `owner()` returns the update authority, which is why
    /// `resolve_pubkey_to_authorities_collection` grants `Authority::Owner` to
    /// a collection's update authority (roadmap section 13, note 4).
    #[test]
    fn core_asset_impl_maps_both_roles_to_the_update_authority() {
        let update_authority = Pubkey::new_unique();
        let collection = collection(update_authority);

        assert_eq!(collection.owner(), &update_authority);
        assert_eq!(
            CoreAsset::update_authority(&collection),
            UpdateAuthority::Collection(update_authority)
        );
    }

    // ---------------------------------------------------------------------
    // `check_*` matrix. A `None` check means `validate_collection_permissions`
    // never calls the matching validator, so those validators are dead code.
    // ---------------------------------------------------------------------

    #[test]
    fn check_matrix() {
        assert_eq!(CollectionV1::check_create(), CheckResult::CanApprove);
        assert_eq!(CollectionV1::check_add_plugin(), CheckResult::CanApprove);
        assert_eq!(CollectionV1::check_remove_plugin(), CheckResult::CanApprove);
        assert_eq!(
            CollectionV1::check_approve_plugin_authority(),
            CheckResult::CanApprove
        );
        assert_eq!(
            CollectionV1::check_revoke_plugin_authority(),
            CheckResult::CanApprove
        );
        assert_eq!(CollectionV1::check_update(), CheckResult::CanApprove);
        assert_eq!(
            CollectionV1::check_add_external_plugin_adapter(),
            CheckResult::CanApprove
        );
        assert_eq!(
            CollectionV1::check_remove_external_plugin_adapter(),
            CheckResult::CanApprove
        );
        assert_eq!(CollectionV1::check_execute(), CheckResult::CanApprove);

        // `None` => the paired validator is never reached.
        assert_eq!(CollectionV1::check_update_plugin(), CheckResult::None);
        assert_eq!(CollectionV1::check_transfer(), CheckResult::None);
        assert_eq!(CollectionV1::check_burn(), CheckResult::None);
        assert_eq!(CollectionV1::check_compress(), CheckResult::None);
        assert_eq!(CollectionV1::check_decompress(), CheckResult::None);
        assert_eq!(
            CollectionV1::check_update_external_plugin_adapter(),
            CheckResult::None
        );
    }

    // ---------------------------------------------------------------------
    // `validate_*`
    // ---------------------------------------------------------------------

    #[test]
    fn validate_create_approves_only_the_update_authority() {
        let update_authority = Pubkey::new_unique();
        let collection = collection(update_authority);

        let mut ua = TestAccount::signer(update_authority);
        assert_eq!(
            collection.validate_create(&ua.info(), None, None),
            Ok(ValidationResult::Approved)
        );

        let mut other = TestAccount::stranger();
        assert_eq!(
            collection.validate_create(&other.info(), None, None),
            Ok(ValidationResult::Pass)
        );
    }

    #[test]
    fn validate_update_approves_only_the_update_authority() {
        let update_authority = Pubkey::new_unique();
        let collection = collection(update_authority);

        let mut ua = TestAccount::signer(update_authority);
        assert_eq!(
            collection.validate_update(&ua.info(), None, None),
            Ok(ValidationResult::Approved)
        );

        let mut other = TestAccount::stranger();
        assert_eq!(
            collection.validate_update(&other.info(), None, None),
            Ok(ValidationResult::Pass)
        );
    }

    /// Add, remove, approve and revoke all require the update authority *and*
    /// an authority-managed plugin; an owner-managed plugin only ever abstains,
    /// which is what turns `AddCollectionPluginV1` of `FreezeDelegate` into
    /// `InvalidAuthority`.
    #[test]
    fn validate_plugin_ops_require_ua_and_a_ua_managed_plugin() {
        let update_authority = Pubkey::new_unique();
        let collection = collection(update_authority);

        type Validator = fn(
            &CollectionV1,
            &AccountInfo,
            Option<&Plugin>,
            Option<&ExternalPluginAdapter>,
        ) -> Result<ValidationResult, ProgramError>;

        let validators: [Validator; 4] = [
            CollectionV1::validate_add_plugin,
            CollectionV1::validate_remove_plugin,
            CollectionV1::validate_approve_plugin_authority,
            CollectionV1::validate_revoke_plugin_authority,
        ];

        for validate in validators {
            let mut ua = TestAccount::signer(update_authority);
            assert_eq!(
                validate(&collection, &ua.info(), Some(&ua_managed()), None),
                Ok(ValidationResult::Approved)
            );

            let mut ua = TestAccount::signer(update_authority);
            assert_eq!(
                validate(&collection, &ua.info(), Some(&owner_managed()), None),
                Ok(ValidationResult::Pass)
            );

            let mut other = TestAccount::stranger();
            assert_eq!(
                validate(&collection, &other.info(), Some(&ua_managed()), None),
                Ok(ValidationResult::Pass)
            );

            // No processor passes `None`; the arm errors rather than abstains.
            let mut ua = TestAccount::signer(update_authority);
            assert_eq!(
                validate(&collection, &ua.info(), None, None),
                Err(invalid_plugin())
            );
        }
    }

    #[test]
    fn validate_external_adapter_ops_approve_only_the_update_authority() {
        let update_authority = Pubkey::new_unique();
        let collection = collection(update_authority);

        type Validator = fn(
            &CollectionV1,
            &AccountInfo,
            Option<&Plugin>,
            Option<&ExternalPluginAdapter>,
        ) -> Result<ValidationResult, ProgramError>;

        let validators: [Validator; 2] = [
            CollectionV1::validate_add_external_plugin_adapter,
            CollectionV1::validate_remove_external_plugin_adapter,
        ];

        for validate in validators {
            let mut ua = TestAccount::signer(update_authority);
            assert_eq!(
                validate(&collection, &ua.info(), None, None),
                Ok(ValidationResult::Approved)
            );

            let mut other = TestAccount::stranger();
            assert_eq!(
                validate(&collection, &other.info(), None, None),
                Ok(ValidationResult::Pass)
            );
        }
    }

    /// These validators are unconditional abstentions. Five of them are also
    /// unreachable, because their paired `check_*` returns `CheckResult::None`
    /// (roadmap section 13, finding 4); `validate_execute` is reachable and
    /// leaves the decision to the asset.
    #[test]
    fn validators_that_always_abstain() {
        let update_authority = Pubkey::new_unique();
        let collection = collection(update_authority);

        type Validator = fn(
            &CollectionV1,
            &AccountInfo,
            Option<&Plugin>,
            Option<&ExternalPluginAdapter>,
        ) -> Result<ValidationResult, ProgramError>;

        let validators: [Validator; 6] = [
            CollectionV1::validate_update_plugin,
            CollectionV1::validate_transfer,
            CollectionV1::validate_burn,
            CollectionV1::validate_compress,
            CollectionV1::validate_decompress,
            CollectionV1::validate_execute,
        ];

        for validate in validators {
            // Even the update authority, and even with a plugin supplied.
            let mut ua = TestAccount::signer(update_authority);
            assert_eq!(
                validate(&collection, &ua.info(), Some(&ua_managed()), None),
                Ok(ValidationResult::Pass)
            );
        }

        // `validate_update_external_plugin_adapter` has no `Plugin` argument
        // that matters either; it is dead for the same reason.
        let mut ua = TestAccount::signer(update_authority);
        assert_eq!(
            collection.validate_update_external_plugin_adapter(&ua.info(), None, None),
            Ok(ValidationResult::Pass)
        );
    }
}
