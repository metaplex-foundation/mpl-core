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
            let serialized = collection.try_to_vec().unwrap();
            assert_eq!(serialized.len(), collection.len());
        }
    }
}
