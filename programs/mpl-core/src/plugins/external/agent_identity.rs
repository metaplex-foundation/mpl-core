use borsh::{BorshDeserialize, BorshSerialize};
use mpl_agent_tools::accounts::ExecutionDelegateRecordV1;
use mpl_utils::{assert_derivation, assert_signer};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::MplCoreError,
    plugins::{
        abstain, approve, reject, Authority, ExternalCheckResult, HookableLifecycleEvent,
        PluginValidation, PluginValidationContext, ValidationResult,
    },
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

    /// Check that the agent identity program is signing off on the addition.
    pub fn verify_identity_registry(
        identity_account: &AccountInfo,
        asset: &Pubkey,
    ) -> ProgramResult {
        assert_signer(identity_account)?;
        let _ = assert_derivation(
            &mpl_agent_identity::ID,
            identity_account,
            &[b"agent_identity", asset.as_ref()],
            MplCoreError::AgentIdentityMustSign,
        )?;

        Ok(())
    }

    /// Verify that the Execution Delegate is valid for the asset.
    pub fn verify_execution_delegate(
        asset: &Pubkey,
        authority: &Pubkey,
        maybe_execution_delegate_record: &AccountInfo,
    ) -> Result<ValidationResult, ProgramError> {
        // If the account there is owned by the mpl-agent-tools program, then it's probably an execution delegate record.
        if maybe_execution_delegate_record.owner == &mpl_agent_tools::ID
            && maybe_execution_delegate_record.data_len() > 0
            && maybe_execution_delegate_record.data.borrow()[0]
                == mpl_agent_tools::types::Key::ExecutionDelegateRecordV1 as u8
        {
            let execution_delegate_record =
                ExecutionDelegateRecordV1::try_from(maybe_execution_delegate_record)?;
            if execution_delegate_record.agent_asset == *asset
                && execution_delegate_record.authority == *authority
            {
                return approve!();
            }
        }

        abstain!()
    }
}

impl PluginValidation for AgentIdentity {
    fn validate_create(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // Reject if being added to a collection (asset_info is None for collections).
        if let Some(asset_info) = ctx.asset_info {
            // Verify that the agent identity program is signing off on the addition.
            // Accounts cannot be empty so the unwrap is safe.
            Self::verify_identity_registry(ctx.accounts.last().unwrap(), asset_info.key)?;
            abstain!()
        } else {
            reject!()
        }
    }

    fn validate_add_external_plugin_adapter(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // Reject if being added to a collection (asset_info is None for collections).
        if let Some(asset_info) = ctx.asset_info {
            // Verify that the agent identity program is signing off on the addition.
            // Accounts cannot be empty so the unwrap is safe.
            Self::verify_identity_registry(ctx.accounts.last().unwrap(), asset_info.key)?;
            abstain!()
        } else {
            reject!()
        }
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
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.asset_info.is_some() && ctx.accounts.len() > 7 {
            Self::verify_execution_delegate(
                ctx.asset_info.unwrap().key,
                ctx.authority_info.key,
                ctx.accounts.get(7).unwrap(),
            )
        } else {
            abstain!()
        }
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
