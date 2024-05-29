use std::collections::{BTreeMap, HashSet};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{error::MplCoreError, plugins::PluginType, state::Authority};

use super::{Plugin, PluginValidation, PluginValidationContext, ValidationResult};

/// The creator on an asset and whether or not they are verified.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Hash)]
pub struct AutographSignature {
    address: Pubkey,
    message: String,
}

/// Structure for an autograph book, often used in conjunction with the Royalties plugin for verified creators
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct Autograph {
    /// A list of signatures with option message
    signatures: Vec<AutographSignature>,
}

fn validate_autograph(
    new_autograph: &Autograph,
    autograph: Option<&Autograph>,
    authority: &Pubkey,
    is_plugin_authority: bool,
) -> Result<ValidationResult, ProgramError> {
    let mut existing_map: BTreeMap<Pubkey, &AutographSignature> = BTreeMap::new();
    if let Some(autograph) = autograph {
        for sig in autograph.signatures.iter() {
            existing_map.insert(sig.address, sig);
        }
    }

    for sig in new_autograph.signatures.iter() {
        // only the signing authority can add their own signature
        match existing_map.get(&sig.address) {
            Some(existing_sig) => {
                if existing_sig.message != sig.message {
                    solana_program::msg!("Autograph: Rejected");
                    return Err(MplCoreError::InvalidPluginOperation.into());
                }
            }
            None => {
                if &sig.address != authority {
                    solana_program::msg!("Autograph: Rejected");
                    return Err(MplCoreError::MissingSigner.into());
                }
            }
        }
    }

    let new_signatures: HashSet<_> = new_autograph
        .signatures
        .iter()
        .map(|sig| sig.address)
        .collect();

    if new_autograph.signatures.len() != new_signatures.len() {
        solana_program::msg!("Autograph: Rejected");
        return Err(MplCoreError::InvalidPluginSetting.into());
    }

    if !is_plugin_authority {
        // only the plugin authority can remove signatures
        for (key, _) in existing_map.iter() {
            if !new_signatures.contains(key) {
                solana_program::msg!("Autograph: Rejected");
                return Err(MplCoreError::MissingSigner.into());
            }
        }
    }

    Ok(ValidationResult::Pass)
}

impl PluginValidation for Autograph {
    fn validate_create(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        validate_autograph(self, None, ctx.authority_info.key, true)
    }

    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match ctx.target_plugin {
            Some(Plugin::Autograph(_autograph)) => {
                validate_autograph(self, None, ctx.authority_info.key, true)?;
                Ok(ValidationResult::Approved)
            }
            _ => Ok(ValidationResult::Pass),
        }
    }

    fn validate_update_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let resolved_authorities = ctx
            .resolved_authorities
            .ok_or(MplCoreError::InvalidAuthority)?;
        match ctx.target_plugin {
            Some(Plugin::Autograph(autograph)) => {
                validate_autograph(
                    autograph,
                    Some(self),
                    ctx.authority_info.key,
                    resolved_authorities.contains(ctx.self_authority),
                )?;
                solana_program::msg!("Autograph: Approved");
                Ok(ValidationResult::Approved)
            }
            _ => Ok(ValidationResult::Pass),
        }
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.self_authority
            == &(Authority::Address {
                address: *ctx.authority_info.key,
            })
            && ctx.target_plugin.is_some()
            && PluginType::from(ctx.target_plugin.unwrap()) == PluginType::Royalties
        {
            solana_program::msg!("Autograph: Approved");
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
