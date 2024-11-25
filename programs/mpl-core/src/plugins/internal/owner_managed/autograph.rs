use std::collections::{BTreeMap, HashSet};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    plugins::{
        abstain, approve, Plugin, PluginValidation, PluginValidationContext, ValidationResult,
    },
    state::DataBlob,
};

/// The creator on an asset and whether or not they are verified.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Hash)]
pub struct AutographSignature {
    address: Pubkey, // 32
    message: String, // 4 + len
}

impl DataBlob for AutographSignature {
    fn get_initial_size() -> usize {
        32 + 4
    }

    fn get_size(&self) -> usize {
        32 + 4 + self.message.len()
    }
}

/// Structure for an autograph book, often used in conjunction with the Royalties plugin for verified creators
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct Autograph {
    /// A list of signatures with option message
    signatures: Vec<AutographSignature>, // 4 + len * Autograph len
}

fn validate_autograph(
    new_autograph: &Autograph,
    autograph: Option<&Autograph>,
    authority: &Pubkey,
    is_plugin_authority: bool,
) -> Result<ValidationResult, ProgramError> {
    let existing_map = autograph.map_or_else(BTreeMap::new, |autograph| {
        autograph
            .signatures
            .iter()
            .map(|sig| (sig.address, sig))
            .collect::<BTreeMap<Pubkey, &AutographSignature>>()
    });

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

    abstain!()
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
                approve!()
            }
            _ => abstain!(),
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
                approve!()
            }
            _ => abstain!(),
        }
    }
}

impl DataBlob for Autograph {
    fn get_initial_size() -> usize {
        4
    }

    fn get_size(&self) -> usize {
        4 + self
            .signatures
            .iter()
            .fold(0, |acc, sig| acc + sig.get_size())
    }
}
