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
    //TODO: Fix this for dynamic size
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

    /// The base length of the asset account with an empty name and uri and no seq.
    pub const BASE_LENGTH: usize = 1 + 32 + 33 + 4 + 4 + 1;

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
    fn get_initial_size() -> usize {
        AssetV1::BASE_LENGTH
    }

    fn get_size(&self) -> usize {
        let mut size = AssetV1::BASE_LENGTH + self.name.len() + self.uri.len();
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
