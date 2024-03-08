use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::state::Authority;

use super::{PluginValidation, ValidationResult};

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
    percentage: u8,
    /// A list of creators to receive royalties.
    creators: Vec<Creator>,
    /// The rule set for the asset to enforce royalties.
    rule_set: RuleSet,
}

impl PluginValidation for Royalties {
    fn validate_create(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_update(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_burn(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_transfer(
        &self,
        authority_info: &AccountInfo,
        new_owner: &AccountInfo,
        _authority: &Authority,
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

    fn validate_compress(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_decompress(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }
}
