use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use std::mem::size_of;

use crate::{
    error::MplCoreError,
    plugins::{abstain, approve, CheckResult, ExternalPluginAdapter, Plugin, ValidationResult},
    state::{Compressible, CompressionProof, DataBlob, Key, SolanaAccount},
};

use super::{Authority, CoreAsset, UpdateAuthority};

/// The Core Asset structure that exists at the beginning of every asset account.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount, Eq, PartialEq)]
pub struct AssetV1 {
    /// The account discriminator.
    pub key: Key, //1
    /// The owner of the asset.
    pub owner: Pubkey, //32
    /// The update authority of the asset.
    pub update_authority: UpdateAuthority, //33
    /// The name of the asset.
    pub name: String, //4
    /// The URI of the asset that points to the off-chain data.
    pub uri: String, //4
    /// The sequence number used for indexing with compression.
    pub seq: Option<u64>, //1
}

impl AssetV1 {
    /// The base length of the asset account with an empty name and uri and no seq.
    const BASE_LEN: usize = 1 // Key
                            + 32 // Owner
                            + 1 // Update Authority discriminator
                            + 4 // Name length
                            + 4 // URI length
                            + 1; // Seq option

    /// Create a new `Asset` with correct `Key` and `seq` of None.
    pub fn new(
        owner: Pubkey,
        update_authority: UpdateAuthority,
        name: String,
        uri: String,
    ) -> Self {
        Self {
            key: Key::AssetV1,
            owner,
            update_authority,
            name,
            uri,
            seq: None,
        }
    }

    /// If `asset.seq` is `Some(_)` then increment and save asset to account space.
    pub fn increment_seq_and_save(&mut self, account: &AccountInfo) -> ProgramResult {
        if let Some(seq) = &mut self.seq {
            *seq = seq.saturating_add(1);
            self.save(account, 0)?;
        };

        Ok(())
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
        CheckResult::CanApprove
    }

    /// Check permissions for the burn lifecycle event.
    pub fn check_burn() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the update lifecycle event.
    pub fn check_update() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the compress lifecycle event.
    pub fn check_compress() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the decompress lifecycle event.
    pub fn check_decompress() -> CheckResult {
        CheckResult::CanApprove
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
        _authority_info: &AccountInfo,
        _new_plugin: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        // If the asset is part of a collection, the collection must approve the create.
        match self.update_authority {
            UpdateAuthority::Collection(_) => abstain!(),
            _ => approve!(),
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

        // If it's an owner managed plugin or a UA managed plugin and the asset
        // is not in a collection, then it can be added.
        if (authority_info.key == &self.owner && new_plugin.manager() == Authority::Owner)
            || (UpdateAuthority::Address(*authority_info.key) == self.update_authority
                && new_plugin.manager() == Authority::UpdateAuthority)
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
        let plugin = match plugin_to_remove {
            Some(plugin) => plugin,
            None => return Err(MplCoreError::InvalidPlugin.into()),
        };

        if (plugin.manager() == Authority::UpdateAuthority
            && self.update_authority == UpdateAuthority::Address(*authority_info.key))
            || (plugin.manager() == Authority::Owner && authority_info.key == &self.owner)
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
        if let Some(plugin) = plugin {
            if (plugin.manager() == Authority::UpdateAuthority
                && self.update_authority == UpdateAuthority::Address(*authority_info.key))
                || (plugin.manager() == Authority::Owner && authority_info.key == &self.owner)
            {
                approve!()
            } else {
                abstain!()
            }
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
        if let Some(plugin) = plugin {
            if (plugin.manager() == Authority::UpdateAuthority
                && self.update_authority == UpdateAuthority::Address(*authority_info.key))
                || (plugin.manager() == Authority::Owner && authority_info.key == &self.owner)
            {
                approve!()
            } else {
                abstain!()
            }
        } else {
            abstain!()
        }
    }

    /// Validate the update lifecycle event.
    pub fn validate_update(
        &self,
        authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority_info.key == &self.update_authority.key() {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the burn lifecycle event.
    pub fn validate_burn(
        &self,
        authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority_info.key == &self.owner {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the transfer lifecycle event.
    pub fn validate_transfer(
        &self,
        authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority_info.key == &self.owner {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the compress lifecycle event.
    pub fn validate_compress(
        &self,
        authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority_info.key == &self.owner {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the decompress lifecycle event.
    pub fn validate_decompress(
        &self,
        authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority_info.key == &self.owner {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the execute lifecycle event.
    pub fn validate_execute(
        &self,
        authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority_info.key == &self.owner {
            approve!()
        } else {
            abstain!()
        }
    }

    /// Validate the add external plugin adapter lifecycle event.
    pub fn validate_add_external_plugin_adapter(
        &self,
        authority_info: &AccountInfo,
        _: Option<&Plugin>,
        _new_plugin: Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError> {
        // If it's not in a collection, then it can be added.
        if UpdateAuthority::Address(*authority_info.key) == self.update_authority {
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
        if self.update_authority == UpdateAuthority::Address(*authority_info.key) {
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
}

impl Compressible for AssetV1 {}

impl DataBlob for AssetV1 {
    fn len(&self) -> usize {
        let mut size = AssetV1::BASE_LEN + self.name.len() + self.uri.len();

        if let UpdateAuthority::Address(_) | UpdateAuthority::Collection(_) = self.update_authority
        {
            size += 32;
        }

        if self.seq.is_some() {
            size += size_of::<u64>();
        }
        size
    }
}

impl SolanaAccount for AssetV1 {
    fn key() -> Key {
        Key::AssetV1
    }
}

impl From<CompressionProof> for AssetV1 {
    fn from(compression_proof: CompressionProof) -> Self {
        Self {
            key: Self::key(),
            update_authority: compression_proof.update_authority,
            owner: compression_proof.owner,
            name: compression_proof.name,
            uri: compression_proof.uri,
            seq: Some(compression_proof.seq),
        }
    }
}

impl CoreAsset for AssetV1 {
    fn update_authority(&self) -> UpdateAuthority {
        self.update_authority.clone()
    }

    fn owner(&self) -> &Pubkey {
        &self.owner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_len() {
        let assets = vec![
            AssetV1 {
                key: Key::AssetV1,
                owner: Pubkey::default(),
                update_authority: UpdateAuthority::None,
                name: "".to_string(),
                uri: "".to_string(),
                seq: None,
            },
            AssetV1 {
                key: Key::AssetV1,
                owner: Pubkey::default(),
                update_authority: UpdateAuthority::Address(Pubkey::default()),
                name: "test".to_string(),
                uri: "test".to_string(),
                seq: None,
            },
            AssetV1 {
                key: Key::AssetV1,
                owner: Pubkey::default(),
                update_authority: UpdateAuthority::Collection(Pubkey::default()),
                name: "test2".to_string(),
                uri: "test2".to_string(),
                seq: Some(1),
            },
        ];
        for asset in assets {
            let serialized = borsh::to_vec(&asset).unwrap();
            assert_eq!(serialized.len(), asset.len());
        }
    }

    use crate::{
        plugins::{Attributes, FreezeDelegate},
        state::CollectionV1,
        utils::test_account::TestAccount,
    };

    fn asset(owner: Pubkey, update_authority: UpdateAuthority) -> AssetV1 {
        AssetV1::new(
            owner,
            update_authority,
            "name".to_string(),
            "uri".to_string(),
        )
    }

    /// An owner-managed plugin.
    fn owner_managed() -> Plugin {
        Plugin::FreezeDelegate(FreezeDelegate { frozen: false })
    }

    /// An authority-managed plugin.
    fn ua_managed() -> Plugin {
        Plugin::Attributes(Attributes::new())
    }

    fn invalid_plugin() -> ProgramError {
        MplCoreError::InvalidPlugin.into()
    }

    type Validator = fn(
        &AssetV1,
        &AccountInfo,
        Option<&Plugin>,
        Option<&ExternalPluginAdapter>,
    ) -> Result<ValidationResult, ProgramError>;

    // ---------------------------------------------------------------------
    // `increment_seq_and_save`
    // ---------------------------------------------------------------------

    /// `seq` is only ever `Some(_)` on an asset rebuilt from a compression
    /// proof, so this branch has no on-chain producer today.
    #[test]
    fn increment_seq_and_save_bumps_and_persists_a_some_seq() {
        let mut asset = asset(Pubkey::new_unique(), UpdateAuthority::None);
        asset.seq = Some(3);
        let mut account = TestAccount::owned(vec![0; borsh::to_vec(&asset).unwrap().len()]);

        {
            let info = account.info();
            assert_eq!(asset.increment_seq_and_save(&info), Ok(()));
        }

        assert_eq!(asset.seq, Some(4));
        let stored = AssetV1::try_from_slice(account.data()).unwrap();
        assert_eq!(stored, asset);
    }

    #[test]
    fn increment_seq_and_save_saturates_at_u64_max() {
        let mut asset = asset(Pubkey::new_unique(), UpdateAuthority::None);
        asset.seq = Some(u64::MAX);
        let mut account = TestAccount::owned(vec![0; borsh::to_vec(&asset).unwrap().len()]);

        {
            let info = account.info();
            assert_eq!(asset.increment_seq_and_save(&info), Ok(()));
        }

        assert_eq!(asset.seq, Some(u64::MAX));
    }

    /// With `seq: None` nothing is written at all, which is why the common
    /// path can be handed an account it must not touch.
    #[test]
    fn increment_seq_and_save_is_a_no_op_for_a_none_seq() {
        let mut asset = asset(Pubkey::new_unique(), UpdateAuthority::None);
        let mut account = TestAccount::owned(vec![0; borsh::to_vec(&asset).unwrap().len()]);

        {
            let info = account.info();
            assert_eq!(asset.increment_seq_and_save(&info), Ok(()));
        }

        assert_eq!(asset.seq, None);
        assert!(account.data().iter().all(|byte| *byte == 0));
    }

    // ---------------------------------------------------------------------
    // `From<CompressionProof>` and `CoreAsset`
    // ---------------------------------------------------------------------

    /// The rebuilt asset always carries the proof's `seq` as `Some(_)`, never
    /// the `None` an on-chain asset has.
    #[test]
    fn from_compression_proof_sets_key_and_seq() {
        let original = asset(
            Pubkey::new_unique(),
            UpdateAuthority::Address(Pubkey::new_unique()),
        );
        let proof = CompressionProof::new(original.clone(), 9, vec![]);

        let rebuilt = AssetV1::from(proof);
        assert_eq!(rebuilt.key, Key::AssetV1);
        assert_eq!(rebuilt.seq, Some(9));
        assert_eq!(
            rebuilt,
            AssetV1 {
                seq: Some(9),
                ..original
            }
        );
    }

    /// Reachable only through `assert_authority`, which has no callers
    /// (roadmap section 13, finding 4).
    #[test]
    fn core_asset_impl_returns_the_stored_roles() {
        let owner = Pubkey::new_unique();
        let update_authority = Pubkey::new_unique();
        let asset = asset(owner, UpdateAuthority::Address(update_authority));

        assert_eq!(asset.owner(), &owner);
        assert_eq!(
            CoreAsset::update_authority(&asset),
            UpdateAuthority::Address(update_authority)
        );

        let collection_asset = asset_in_collection(owner, update_authority);
        assert_eq!(
            CoreAsset::update_authority(&collection_asset),
            UpdateAuthority::Collection(update_authority)
        );
        assert_eq!(
            CoreAsset::update_authority(&collection_asset).key(),
            CollectionV1::new(update_authority, String::new(), String::new(), 0, 0)
                .update_authority
        );
    }

    fn asset_in_collection(owner: Pubkey, collection: Pubkey) -> AssetV1 {
        asset(owner, UpdateAuthority::Collection(collection))
    }

    // ---------------------------------------------------------------------
    // `check_*` matrix. A `None` check means `validate_asset_permissions`
    // never calls the matching validator.
    // ---------------------------------------------------------------------

    #[test]
    fn check_matrix() {
        assert_eq!(AssetV1::check_create(), CheckResult::CanApprove);
        assert_eq!(AssetV1::check_add_plugin(), CheckResult::CanApprove);
        assert_eq!(AssetV1::check_remove_plugin(), CheckResult::CanApprove);
        assert_eq!(
            AssetV1::check_approve_plugin_authority(),
            CheckResult::CanApprove
        );
        assert_eq!(
            AssetV1::check_revoke_plugin_authority(),
            CheckResult::CanApprove
        );
        assert_eq!(AssetV1::check_transfer(), CheckResult::CanApprove);
        assert_eq!(AssetV1::check_burn(), CheckResult::CanApprove);
        assert_eq!(AssetV1::check_update(), CheckResult::CanApprove);
        assert_eq!(AssetV1::check_compress(), CheckResult::CanApprove);
        assert_eq!(AssetV1::check_decompress(), CheckResult::CanApprove);
        assert_eq!(
            AssetV1::check_add_external_plugin_adapter(),
            CheckResult::CanApprove
        );
        assert_eq!(
            AssetV1::check_remove_external_plugin_adapter(),
            CheckResult::CanApprove
        );
        assert_eq!(AssetV1::check_execute(), CheckResult::CanApprove);

        // `None` => the paired validator is never reached.
        assert_eq!(AssetV1::check_update_plugin(), CheckResult::None);
        assert_eq!(
            AssetV1::check_update_external_plugin_adapter(),
            CheckResult::None
        );
    }

    // ---------------------------------------------------------------------
    // `validate_*`
    // ---------------------------------------------------------------------

    /// An asset in a collection defers the create decision to the collection.
    #[test]
    fn validate_create_abstains_only_for_a_collection_authority() {
        let owner = Pubkey::new_unique();
        let mut signer = TestAccount::stranger();

        for update_authority in [
            UpdateAuthority::None,
            UpdateAuthority::Address(Pubkey::new_unique()),
        ] {
            assert_eq!(
                asset(owner, update_authority).validate_create(&signer.info(), None, None),
                Ok(ValidationResult::Approved)
            );
        }

        assert_eq!(
            asset_in_collection(owner, Pubkey::new_unique()).validate_create(
                &signer.info(),
                None,
                None
            ),
            Ok(ValidationResult::Pass)
        );
    }

    /// The owner may add owner-managed plugins and the update authority may add
    /// authority-managed ones; crossing the two abstains, and an asset in a
    /// collection abstains either way so the collection decides.
    #[test]
    fn validate_add_and_remove_plugin_matrix() {
        let owner = Pubkey::new_unique();
        let update_authority = Pubkey::new_unique();
        let asset = asset(owner, UpdateAuthority::Address(update_authority));
        let collection_asset = asset_in_collection(owner, update_authority);

        let validators: [Validator; 2] = [
            AssetV1::validate_add_plugin,
            AssetV1::validate_remove_plugin,
        ];

        for validate in validators {
            let mut signer = TestAccount::signer(owner);
            assert_eq!(
                validate(&asset, &signer.info(), Some(&owner_managed()), None),
                Ok(ValidationResult::Approved)
            );

            let mut signer = TestAccount::signer(update_authority);
            assert_eq!(
                validate(&asset, &signer.info(), Some(&ua_managed()), None),
                Ok(ValidationResult::Approved)
            );

            // Owner cannot touch an authority-managed plugin...
            let mut signer = TestAccount::signer(owner);
            assert_eq!(
                validate(&asset, &signer.info(), Some(&ua_managed()), None),
                Ok(ValidationResult::Pass)
            );

            // ...nor the update authority an owner-managed one.
            let mut signer = TestAccount::signer(update_authority);
            assert_eq!(
                validate(&asset, &signer.info(), Some(&owner_managed()), None),
                Ok(ValidationResult::Pass)
            );

            let mut signer = TestAccount::stranger();
            assert_eq!(
                validate(&asset, &signer.info(), Some(&owner_managed()), None),
                Ok(ValidationResult::Pass)
            );

            // In a collection the asset abstains on the UA-managed plugin even
            // for the collection address, leaving the decision to the
            // collection's own validator.
            let mut signer = TestAccount::signer(update_authority);
            assert_eq!(
                validate(&collection_asset, &signer.info(), Some(&ua_managed()), None),
                Ok(ValidationResult::Pass)
            );

            // The owner keeps control of owner-managed plugins in a collection.
            let mut signer = TestAccount::signer(owner);
            assert_eq!(
                validate(
                    &collection_asset,
                    &signer.info(),
                    Some(&owner_managed()),
                    None
                ),
                Ok(ValidationResult::Approved)
            );

            // No processor passes `None`; the arm errors rather than abstains.
            let mut signer = TestAccount::signer(owner);
            assert_eq!(
                validate(&asset, &signer.info(), None, None),
                Err(invalid_plugin())
            );
        }
    }

    /// Approve and revoke use the same authority matrix as add/remove, but a
    /// missing plugin abstains here instead of erroring.
    #[test]
    fn validate_approve_and_revoke_plugin_authority_matrix() {
        let owner = Pubkey::new_unique();
        let update_authority = Pubkey::new_unique();
        let asset = asset(owner, UpdateAuthority::Address(update_authority));

        let validators: [Validator; 2] = [
            AssetV1::validate_approve_plugin_authority,
            AssetV1::validate_revoke_plugin_authority,
        ];

        for validate in validators {
            let mut signer = TestAccount::signer(owner);
            assert_eq!(
                validate(&asset, &signer.info(), Some(&owner_managed()), None),
                Ok(ValidationResult::Approved)
            );

            let mut signer = TestAccount::signer(update_authority);
            assert_eq!(
                validate(&asset, &signer.info(), Some(&ua_managed()), None),
                Ok(ValidationResult::Approved)
            );

            let mut signer = TestAccount::signer(owner);
            assert_eq!(
                validate(&asset, &signer.info(), Some(&ua_managed()), None),
                Ok(ValidationResult::Pass)
            );

            let mut signer = TestAccount::stranger();
            assert_eq!(
                validate(&asset, &signer.info(), Some(&owner_managed()), None),
                Ok(ValidationResult::Pass)
            );

            let mut signer = TestAccount::signer(owner);
            assert_eq!(
                validate(&asset, &signer.info(), None, None),
                Ok(ValidationResult::Pass)
            );
        }
    }

    /// Burn, transfer, compress, decompress and execute all approve exactly the
    /// owner and abstain for everybody else; abstention becomes `NoApprovals`
    /// unless a plugin approves.
    #[test]
    fn owner_gated_validators() {
        let owner = Pubkey::new_unique();
        let update_authority = Pubkey::new_unique();
        let asset = asset(owner, UpdateAuthority::Address(update_authority));

        let validators: [Validator; 5] = [
            AssetV1::validate_burn,
            AssetV1::validate_transfer,
            AssetV1::validate_compress,
            AssetV1::validate_decompress,
            AssetV1::validate_execute,
        ];

        for validate in validators {
            let mut signer = TestAccount::signer(owner);
            assert_eq!(
                validate(&asset, &signer.info(), None, None),
                Ok(ValidationResult::Approved)
            );

            // Not even the update authority.
            let mut signer = TestAccount::signer(update_authority);
            assert_eq!(
                validate(&asset, &signer.info(), None, None),
                Ok(ValidationResult::Pass)
            );

            let mut signer = TestAccount::stranger();
            assert_eq!(
                validate(&asset, &signer.info(), None, None),
                Ok(ValidationResult::Pass)
            );
        }
    }

    /// `validate_update` compares the signer against `update_authority.key()`.
    /// For `UpdateAuthority::None` that is the system program id, so the arm is
    /// safe only because the system program cannot sign (roadmap section 13,
    /// note 9). For `Collection(c)` it is the collection address, so an update
    /// signed by the collection *account address* would be approved by the
    /// asset itself.
    #[test]
    fn validate_update_compares_against_update_authority_key() {
        let owner = Pubkey::new_unique();
        let update_authority = Pubkey::new_unique();

        let addressed = asset(owner, UpdateAuthority::Address(update_authority));
        let mut signer = TestAccount::signer(update_authority);
        assert_eq!(
            addressed.validate_update(&signer.info(), None, None),
            Ok(ValidationResult::Approved)
        );
        let mut signer = TestAccount::signer(owner);
        assert_eq!(
            addressed.validate_update(&signer.info(), None, None),
            Ok(ValidationResult::Pass)
        );

        let collection = Pubkey::new_unique();
        let collection_asset = asset_in_collection(owner, collection);
        let mut signer = TestAccount::signer(collection);
        assert_eq!(
            collection_asset.validate_update(&signer.info(), None, None),
            Ok(ValidationResult::Approved)
        );

        let immutable = asset(owner, UpdateAuthority::None);
        let mut signer = TestAccount::signer(solana_system_interface::program::ID);
        assert_eq!(
            immutable.validate_update(&signer.info(), None, None),
            Ok(ValidationResult::Approved)
        );
        let mut signer = TestAccount::stranger();
        assert_eq!(
            immutable.validate_update(&signer.info(), None, None),
            Ok(ValidationResult::Pass)
        );
    }

    /// External adapters may be added or removed only by an `Address` update
    /// authority; an asset in a collection always abstains so the collection
    /// decides.
    #[test]
    fn validate_external_adapter_ops_require_an_address_update_authority() {
        let owner = Pubkey::new_unique();
        let update_authority = Pubkey::new_unique();
        let asset = asset(owner, UpdateAuthority::Address(update_authority));

        let validators: [Validator; 2] = [
            AssetV1::validate_add_external_plugin_adapter,
            AssetV1::validate_remove_external_plugin_adapter,
        ];

        for validate in validators {
            let mut signer = TestAccount::signer(update_authority);
            assert_eq!(
                validate(&asset, &signer.info(), None, None),
                Ok(ValidationResult::Approved)
            );

            let mut signer = TestAccount::signer(owner);
            assert_eq!(
                validate(&asset, &signer.info(), None, None),
                Ok(ValidationResult::Pass)
            );

            let collection = Pubkey::new_unique();
            let mut signer = TestAccount::signer(collection);
            assert_eq!(
                validate(
                    &asset_in_collection(owner, collection),
                    &signer.info(),
                    None,
                    None
                ),
                Ok(ValidationResult::Pass)
            );
        }
    }

    /// Both of these are dead: their paired `check_*` returns
    /// `CheckResult::None`, so `validate_asset_permissions` never calls them
    /// (roadmap section 13, finding 4).
    #[test]
    fn dead_validators_always_abstain() {
        let owner = Pubkey::new_unique();
        let update_authority = Pubkey::new_unique();
        let asset = asset(owner, UpdateAuthority::Address(update_authority));

        for key in [owner, update_authority, Pubkey::new_unique()] {
            let mut signer = TestAccount::signer(key);
            assert_eq!(
                asset.validate_update_plugin(&signer.info(), Some(&ua_managed()), None),
                Ok(ValidationResult::Pass)
            );

            let mut signer = TestAccount::signer(key);
            assert_eq!(
                asset.validate_update_external_plugin_adapter(&signer.info(), None, None),
                Ok(ValidationResult::Pass)
            );
        }
    }
}
