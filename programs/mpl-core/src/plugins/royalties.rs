use std::collections::HashSet;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{error::MplCoreError, plugins::PluginType, state::Authority};

use super::{Plugin, PluginValidation, PluginValidationContext, ValidationResult};

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
        return Err(MplCoreError::InvalidPluginSetting.into());
    }
    if royalties
        .creators
        .iter()
        .fold(0u8, |acc, creator| acc.saturating_add(creator.percentage))
        != 100
    {
        // TODO propagate a more useful error
        return Err(MplCoreError::InvalidPluginSetting.into());
    }
    // check unique creators array
    let mut seen_addresses = HashSet::new();
    if !royalties
        .creators
        .iter()
        .all(|creator| seen_addresses.insert(creator.address))
    {
        // If `insert` returns false, it means the address was already in the set, indicating a duplicate
        return Err(MplCoreError::InvalidPluginSetting.into());
    }

    Ok(ValidationResult::Pass)
}

impl PluginValidation for Royalties {
    fn validate_create(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        validate_royalties(self)
    }

    fn validate_transfer(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let new_owner = ctx.new_owner.ok_or(MplCoreError::MissingNewOwner)?;
        match &self.rule_set {
            RuleSet::None => Ok(ValidationResult::Pass),
            RuleSet::ProgramAllowList(allow_list) => {
                if allow_list.contains(ctx.authority_info.owner)
                    && allow_list.contains(new_owner.owner)
                {
                    Ok(ValidationResult::Pass)
                } else {
                    solana_program::msg!("Royalties: Rejected");
                    Ok(ValidationResult::Rejected)
                }
            }
            RuleSet::ProgramDenyList(deny_list) => {
                if deny_list.contains(ctx.authority_info.owner)
                    || deny_list.contains(new_owner.owner)
                {
                    solana_program::msg!("Royalties: Rejected");
                    Ok(ValidationResult::Rejected)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
        }
    }

    fn validate_add_plugin(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        validate_royalties(self)
    }

    fn validate_update_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin_to_update = ctx.target_plugin.ok_or(MplCoreError::InvalidPlugin)?;
        let resolved_authorities = ctx
            .resolved_authorities
            .ok_or(MplCoreError::InvalidAuthority)?;

        // Perform validation on the new royalties plugin data.
        if let Plugin::Royalties(royalties) = plugin_to_update {
            if resolved_authorities.contains(ctx.self_authority) {
                validate_royalties(royalties)
            } else {
                Ok(ValidationResult::Pass)
            }
        } else {
            Ok(ValidationResult::Pass)
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
            solana_program::msg!("Royalties: Approved");
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
