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
    /// The address of the creator.
    pub address: Pubkey, // 32
    /// The message of the creator.
    pub message: String, // 4 + len
}

impl AutographSignature {
    const BASE_LEN: usize = 32 // The address
    + 4; // The message length
}

impl DataBlob for AutographSignature {
    fn len(&self) -> usize {
        Self::BASE_LEN + self.message.len()
    }
}

/// Structure for an autograph book, often used in conjunction with the Royalties plugin for verified creators
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct Autograph {
    /// A list of signatures with option message
    pub signatures: Vec<AutographSignature>, // 4 + len * Autograph len
}

impl Autograph {
    const BASE_LEN: usize = 4; // The signatures length
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
    fn len(&self) -> usize {
        Self::BASE_LEN + self.signatures.iter().map(|sig| sig.len()).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autograph_signature_len() {
        let autograph_signature = AutographSignature {
            address: Pubkey::default(),
            message: "test".to_string(),
        };
        let serialized = autograph_signature.try_to_vec().unwrap();
        assert_eq!(serialized.len(), autograph_signature.len());
    }

    #[test]
    fn test_autograph_default_len() {
        let autograph = Autograph { signatures: vec![] };
        let serialized = autograph.try_to_vec().unwrap();
        assert_eq!(serialized.len(), autograph.len());
    }

    #[test]
    fn test_autograph_len() {
        let autograph = Autograph {
            signatures: vec![
                AutographSignature {
                    address: Pubkey::default(),
                    message: "test".to_string(),
                },
                AutographSignature {
                    address: Pubkey::default(),
                    message: "test2".to_string(),
                },
            ],
        };
        let serialized = autograph.try_to_vec().unwrap();
        assert_eq!(serialized.len(), autograph.len());
    }
}
