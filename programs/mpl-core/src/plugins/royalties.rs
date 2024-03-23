use std::collections::HashSet;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::{plugins::PluginType, state::Authority};

use super::{Plugin, PluginValidation, ValidationResult};

/// The creator on an asset and whether or not they are verified.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Creator {
    address: Pubkey,
    percentage: u8,
}

/// The rule set for an asset indicating where it is allowed to be transferred.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub enum RuleSet {
    /// No rules are enforced.
    None,
    /// Allow list of programs that are allowed to transfer, receive, or send the asset.
    ProgramAllowList(Vec<Pubkey>),
    /// Deny list of programs that are not allowed to transfer, receive, or send the asset.
    ProgramDenyList(Vec<Pubkey>),
}

/// Traditional royalties structure for an asset.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct Royalties {
    /// The percentage of royalties to be paid to the creators.
    basis_points: u16,
    /// A list of creators to receive royalties.
    creators: Vec<Creator>,
    /// The rule set for the asset to enforce royalties.
    rule_set: RuleSet,
}

fn validate_royalties(royalties: &Royalties) -> Result<ValidationResult, ProgramError> {
    if royalties.basis_points > 10000 {
        // TODO propagate a more useful error
        return Err(ProgramError::InvalidArgument);
    }
    if royalties
        .creators
        .iter()
        .fold(0u8, |acc, creator| acc.saturating_add(creator.percentage))
        != 100
    {
        // TODO propagate a more useful error
        return Err(ProgramError::InvalidArgument);
    }
    // check unique creators array
    let mut seen_addresses = HashSet::new();
    if !royalties
        .creators
        .iter()
        .all(|creator| seen_addresses.insert(creator.address))
    {
        // If `insert` returns false, it means the address was already in the set, indicating a duplicate
        return Err(ProgramError::InvalidArgument);
    }

    Ok(ValidationResult::Pass)
}

impl PluginValidation for Royalties {
    fn validate_create(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        validate_royalties(self)
    }

    fn validate_transfer(
        &self,
        authority_info: &AccountInfo,
        new_owner: &AccountInfo,
        _authority: &Authority,
        _resolved_authorities: Option<&[Authority]>,
    ) -> Result<ValidationResult, ProgramError> {
        match &self.rule_set {
            RuleSet::None => Ok(ValidationResult::Pass),
            RuleSet::ProgramAllowList(allow_list) => {
                solana_program::msg!("Evaluating royalties");
                if allow_list.contains(authority_info.owner) || allow_list.contains(new_owner.owner)
                {
                    Ok(ValidationResult::Pass)
                } else {
                    Ok(ValidationResult::Rejected)
                }
            }
            RuleSet::ProgramDenyList(deny_list) => {
                if deny_list.contains(authority_info.owner) || deny_list.contains(new_owner.owner) {
                    Ok(ValidationResult::Rejected)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
        }
    }

    fn validate_add_plugin(
        &self,
        _authority: &AccountInfo,
        _authorities: &Authority,
        _new_plugin: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        validate_royalties(self)
    }

    fn validate_update_plugin<T: crate::state::CoreAsset>(
        &self,
        _core_asset: &T,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        validate_royalties(self)
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        authority_info: &AccountInfo,
        authority: &Authority,
        plugin_to_revoke: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        solana_program::msg!("authority_info: {:?}", authority_info.key);
        solana_program::msg!("authority: {:?}", authority);
        if authority
            == &(Authority::Address {
                address: *authority_info.key,
            })
            && plugin_to_revoke.is_some()
            && PluginType::from(plugin_to_revoke.unwrap()) == PluginType::Royalties
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
