use std::collections::HashSet;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::MplCoreError;

use crate::plugins::{
    abstain, reject, AssetValidationCommon, AssetValidationContext, Plugin, PluginValidation,
    PluginValidationContext, ValidationResult,
};
use crate::state::DataBlob;

/// The creator on an asset and whether or not they are verified.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Creator {
    address: Pubkey, // 32
    percentage: u8,  // 1
}

impl DataBlob for Creator {
    fn get_initial_size() -> usize {
        33
    }

    fn get_size(&self) -> usize {
        33
    }
}

/// The rule set for an asset indicating where it is allowed to be transferred.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub enum RuleSet {
    /// No rules are enforced.
    None, // 1
    /// Allow list of programs that are allowed to transfer, receive, or send the asset.
    ProgramAllowList(Vec<Pubkey>), // 4
    /// Deny list of programs that are not allowed to transfer, receive, or send the asset.
    ProgramDenyList(Vec<Pubkey>), // 4
}

impl DataBlob for RuleSet {
    fn get_initial_size() -> usize {
        1
    }

    fn get_size(&self) -> usize {
        1 + match self {
            RuleSet::ProgramAllowList(allow_list) => 4 + allow_list.len() * 32,
            RuleSet::ProgramDenyList(deny_list) => 4 + deny_list.len() * 32,
            RuleSet::None => 0,
        }
    }
}

/// Traditional royalties structure for an asset.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct Royalties {
    /// The percentage of royalties to be paid to the creators.
    basis_points: u16, // 2
    /// A list of creators to receive royalties.
    creators: Vec<Creator>, // 4
    /// The rule set for the asset to enforce royalties.
    rule_set: RuleSet, // 1
}

impl DataBlob for Royalties {
    fn get_initial_size() -> usize {
        1
    }

    fn get_size(&self) -> usize {
        2 // basis_points
        + (4 + self.creators.len() * Creator::get_initial_size()) // creators
        + self.rule_set.get_size() // rule_set
    }
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

    abstain!()
}

impl PluginValidation for Royalties {
    fn validate_create(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        validate_royalties(self)
    }

    fn validate_transfer(
        &self,
        _plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::Transfer { new_owner, .. } = asset_ctx {
            match &self.rule_set {
                RuleSet::None => abstain!(),
                RuleSet::ProgramAllowList(allow_list) => {
                    if allow_list.contains(common.authority_info.owner)
                        && allow_list.contains(new_owner.owner)
                    {
                        abstain!()
                    } else {
                        reject!()
                    }
                }
                RuleSet::ProgramDenyList(deny_list) => {
                    if deny_list.contains(common.authority_info.owner)
                        || deny_list.contains(new_owner.owner)
                    {
                        reject!()
                    } else {
                        abstain!()
                    }
                }
            }
        } else {
            Err(MplCoreError::MissingNewOwner.into())
        }
    }

    fn validate_add_plugin(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::AddPlugin { new_plugin } = asset_ctx {
            match new_plugin {
                Plugin::Royalties(_royalties) => validate_royalties(self),
                _ => abstain!(),
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    fn validate_update_plugin(
        &self,
        plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::UpdatePlugin { new_plugin } = asset_ctx {
            let resolved_authorities = plugin_ctx
                .resolved_authorities
                .ok_or(MplCoreError::InvalidAuthority)?;

            // Perform validation on the new royalties plugin data.
            if let Plugin::Royalties(royalties) = new_plugin {
                if resolved_authorities.contains(plugin_ctx.self_authority) {
                    validate_royalties(royalties)
                } else {
                    abstain!()
                }
            } else {
                abstain!()
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }
}
