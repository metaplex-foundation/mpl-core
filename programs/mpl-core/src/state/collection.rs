use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    plugins::{CheckResult, Plugin, ValidationResult},
};

use super::{Authority, CoreAsset, DataBlob, Key, SolanaAccount, UpdateAuthority};

/// The representation of a collection of assets.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct Collection {
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

impl Collection {
    /// The base length of the collection account with an empty name and uri.
    pub const BASE_LENGTH: usize = 1 + 32 + 4 + 4 + 4 + 4;

    /// Create a new collection.
    pub fn new(
        update_authority: Pubkey,
        name: String,
        uri: String,
        num_minted: u32,
        current_size: u32,
    ) -> Self {
        Self {
            key: Key::Collection,
            update_authority,
            name,
            uri,
            num_minted,
            current_size,
        }
    }

    /// Check permissions for the add plugin lifecycle event.
    pub fn check_add_plugin() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the remove plugin lifecycle event.
    pub fn check_remove_plugin() -> CheckResult {
        CheckResult::CanApprove
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

    /// Validate the add plugin lifecycle event.
    pub fn validate_add_plugin(
        &self,
        authority_info: &AccountInfo,
        new_plugin: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        let new_plugin = match new_plugin {
            Some(plugin) => plugin,
            None => return Err(MplCoreError::InvalidPlugin.into()),
        };

        if *authority_info.key == self.update_authority
            && new_plugin.manager() == Authority::UpdateAuthority
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the remove plugin lifecycle event.
    pub fn validate_remove_plugin(
        &self,
        authority_info: &AccountInfo,
        plugin_to_remove: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin_to_remove = match plugin_to_remove {
            Some(plugin) => plugin,
            None => return Err(MplCoreError::InvalidPlugin.into()),
        };

        if *authority_info.key == self.update_authority
            && plugin_to_remove.manager() == Authority::UpdateAuthority
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the approve plugin authority lifecycle event.
    pub fn validate_approve_plugin_authority(
        &self,
        authority_info: &AccountInfo,
        plugin: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin = match plugin {
            Some(plugin) => plugin,
            None => return Err(MplCoreError::InvalidPlugin.into()),
        };

        if *authority_info.key == self.update_authority
            && plugin.manager() == Authority::UpdateAuthority
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the revoke plugin authority lifecycle event.
    pub fn validate_revoke_plugin_authority(
        &self,
        authority_info: &AccountInfo,
        plugin: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin = match plugin {
            Some(plugin) => plugin,
            None => return Err(MplCoreError::InvalidPlugin.into()),
        };

        if *authority_info.key == self.update_authority
            && plugin.manager() == Authority::UpdateAuthority
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the transfer lifecycle event.
    pub fn validate_transfer(
        &self,
        _authority_info: &AccountInfo,
        _: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the burn lifecycle event.
    pub fn validate_burn(
        &self,
        _authority_info: &AccountInfo,
        _: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the update lifecycle event.
    pub fn validate_update(
        &self,
        authority_info: &AccountInfo,
        _: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority_info.key == &self.update_authority {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the compress lifecycle event.
    pub fn validate_compress(
        &self,
        _authority_info: &AccountInfo,
        _: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the decompress lifecycle event.
    pub fn validate_decompress(
        &self,
        _authority_info: &AccountInfo,
        _: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }
}

impl DataBlob for Collection {
    fn get_initial_size() -> usize {
        Self::BASE_LENGTH
    }

    fn get_size(&self) -> usize {
        Self::BASE_LENGTH + self.name.len() + self.uri.len()
    }
}

impl SolanaAccount for Collection {
    fn key() -> Key {
        Key::Collection
    }
}

impl CoreAsset for Collection {
    fn update_authority(&self) -> UpdateAuthority {
        UpdateAuthority::Collection(self.update_authority)
    }

    fn owner(&self) -> &Pubkey {
        &self.update_authority
    }
}
