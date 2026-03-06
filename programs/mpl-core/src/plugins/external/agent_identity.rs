use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::plugins::{
    abstain, reject, Authority, ExternalCheckResult, HookableLifecycleEvent, PluginValidation,
    PluginValidationContext, ValidationResult,
};

/// Agent Identity plugin that links to an ERC-8004 spec registration file via a URI.
/// This plugin can only be added to an asset, not a collection.  There can be at most
/// one AgentIdentity plugin per asset.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct AgentIdentity {
    /// URI pointing to the ERC-8004 agent registration JSON file.
    pub uri: String,
}

impl AgentIdentity {
    /// Updates the agent identity with the new info.
    pub fn update(&mut self, info: &AgentIdentityUpdateInfo) {
        if let Some(uri) = &info.uri {
            self.uri = uri.clone();
        }
    }
}

impl PluginValidation for AgentIdentity {
    fn validate_create(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // Reject if being added to a collection (asset_info is None for collections).
        if ctx.asset_info.is_none() {
            reject!()
        } else {
            abstain!()
        }
    }

    fn validate_add_external_plugin_adapter(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    fn validate_transfer(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    fn validate_burn(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    fn validate_update(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    fn validate_execute(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }
}

impl From<&AgentIdentityInitInfo> for AgentIdentity {
    fn from(init_info: &AgentIdentityInitInfo) -> Self {
        Self {
            uri: init_info.uri.clone(),
        }
    }
}

/// Agent Identity initialization info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct AgentIdentityInitInfo {
    /// URI pointing to the ERC-8004 agent registration JSON file.
    pub uri: String,
    /// Initial plugin authority.
    pub init_plugin_authority: Option<Authority>,
    /// The lifecycle events for which the external plugin adapter is active.
    pub lifecycle_checks: Vec<(HookableLifecycleEvent, ExternalCheckResult)>,
}

/// Agent Identity update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct AgentIdentityUpdateInfo {
    /// Updated URI pointing to the ERC-8004 agent registration JSON file.
    pub uri: Option<String>,
    /// The lifecycle events for which the external plugin adapter is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
}
