use std::collections::{BTreeMap, HashSet};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::MplCoreError;

use super::{abstain, Plugin, PluginValidation, PluginValidationContext, ValidationResult};

/// The creator on an asset and whether or not they are verified.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Hash)]
pub struct VerifiedCreatorsSignature {
    address: Pubkey,
    verified: bool,
}

/// Structure for storing verified creators, often used in conjunction with the Royalties plugin
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct VerifiedCreators {
    /// A list of signatures
    signatures: Vec<VerifiedCreatorsSignature>,
}

struct SignatureChangeIndices {
    /// Indices of added signatures on new_verified_creators
    added: Vec<u8>,
    /// Indices of changed signatures on new_verified_creators
    changed: Vec<u8>,
    /// Indices of removed signatures on verified_creators
    removed: Vec<u8>,
}

fn calculate_signature_changes(
    new_verified_creators: &VerifiedCreators,
    verified_creators: Option<&VerifiedCreators>,
) -> Result<SignatureChangeIndices, ProgramError> {
    let existing_map = verified_creators.map_or_else(BTreeMap::new, |verified_creators| {
        verified_creators
            .signatures
            .iter()
            .map(|sig| (sig.address, sig))
            .collect::<BTreeMap<Pubkey, &VerifiedCreatorsSignature>>()
    });

    let new_signatures: HashSet<_> = new_verified_creators
        .signatures
        .iter()
        .map(|sig| sig.address)
        .collect();

    if new_verified_creators.signatures.len() != new_signatures.len() {
        // Ensure there are no duplicate signatures
        solana_program::msg!("Verified creators: Rejected");
        return Err(MplCoreError::InvalidPluginSetting.into());
    }

    let mut result = SignatureChangeIndices {
        added: Vec::new(),
        changed: Vec::new(),
        removed: Vec::new(),
    };

    for (i, sig) in new_verified_creators.signatures.iter().enumerate() {
        match existing_map.get(&sig.address) {
            Some(existing_sig) => {
                if existing_sig.verified != sig.verified {
                    result.changed.push(i as u8);
                }
            }
            None => {
                result.added.push(i as u8);
            }
        }
    }

    if let Some(verified_creators) = verified_creators {
        for (i, sig) in verified_creators.signatures.iter().enumerate() {
            if !new_signatures.contains(&sig.address) {
                result.removed.push(i as u8);
            }
        }
    }

    Ok(result)
}

fn validate_verified_creators_as_creator(
    new_verified_creators: &VerifiedCreators,
    verified_creators: &VerifiedCreators,
    authority: &Pubkey,
) -> Result<ValidationResult, ProgramError> {
    // Track any changes in verification status
    let changes = calculate_signature_changes(new_verified_creators, Some(verified_creators))?;

    if !changes.added.is_empty() || !changes.removed.is_empty() {
        // creators cannot add new allowable signatures or remove existing ones
        solana_program::msg!("Verified creators: Rejected");
        return Err(MplCoreError::MissingSigner.into());
    }

    for change in changes.changed.iter() {
        let sig = &new_verified_creators.signatures[*change as usize];
        if &sig.address != authority {
            // creators may only change their own verified status
            solana_program::msg!("Verified creators: Rejected");
            return Err(MplCoreError::MissingSigner.into());
        }
    }

    abstain!()
}

fn validate_verified_creators_as_plugin_authority(
    new_verified_creators: &VerifiedCreators,
    verified_creators: Option<&VerifiedCreators>,
    authority: &Pubkey,
) -> Result<ValidationResult, ProgramError> {
    // The plugin auth is allowed to: add/remove unverified creators, add self and sign for self.
    // The plugin auth cannot remove or unverify any existing creators other than self.
    // This is in line with legacy Token Metadata behaviour for verified creators

    let changes = calculate_signature_changes(new_verified_creators, verified_creators)?;

    for removal in changes.removed.iter() {
        let sig = &verified_creators.unwrap().signatures[*removal as usize];
        if sig.verified && &sig.address != authority {
            solana_program::msg!("Verified creators: Rejected");
            return Err(MplCoreError::InvalidPluginOperation.into());
        }
    }

    for change in changes.changed.iter() {
        let sig = &new_verified_creators.signatures[*change as usize];
        if &sig.address != authority {
            solana_program::msg!("Verified creators: Rejected");
            return Err(MplCoreError::InvalidPluginOperation.into());
        }
    }

    for addition in changes.added.iter() {
        let sig = &new_verified_creators.signatures[*addition as usize];
        if sig.verified && &sig.address != authority {
            solana_program::msg!("Verified creators: Rejected");
            return Err(MplCoreError::MissingSigner.into());
        }
    }

    abstain!()
}

impl PluginValidation for VerifiedCreators {
    fn validate_create(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        validate_verified_creators_as_plugin_authority(self, None, ctx.authority_info.key)
    }

    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match ctx.target_plugin {
            Some(Plugin::VerifiedCreators(_verified_creators)) => {
                validate_verified_creators_as_plugin_authority(self, None, ctx.authority_info.key)
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
            Some(Plugin::VerifiedCreators(verified_creators)) => {
                if resolved_authorities.contains(ctx.self_authority) {
                    validate_verified_creators_as_plugin_authority(
                        verified_creators,
                        Some(self),
                        ctx.authority_info.key,
                    )?;
                    Ok(ValidationResult::Approved)
                } else {
                    validate_verified_creators_as_creator(
                        verified_creators,
                        self,
                        ctx.authority_info.key,
                    )?;
                    Ok(ValidationResult::Approved)
                }
            }
            _ => abstain!(),
        }
    }
}
