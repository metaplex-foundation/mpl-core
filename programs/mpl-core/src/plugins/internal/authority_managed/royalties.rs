use std::collections::HashSet;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::MplCoreError;

use crate::plugins::{
    abstain, reject, Plugin, PluginValidation, PluginValidationContext, ValidationResult,
};
use crate::state::DataBlob;

/// The creator on an asset and whether or not they are verified.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Creator {
    /// The address of the creator.
    pub address: Pubkey, // 32
    /// The percentage of royalties to be paid to the creator.
    pub percentage: u8, // 1
}

impl Creator {
    const BASE_LEN: usize = 32 // The address
    + 1; // The percentage
}

impl DataBlob for Creator {
    fn len(&self) -> usize {
        Self::BASE_LEN
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

impl RuleSet {
    const BASE_LEN: usize = 1; // The rule set discriminator
}

impl DataBlob for RuleSet {
    fn len(&self) -> usize {
        Self::BASE_LEN
            + match self {
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
    pub basis_points: u16, // 2
    /// A list of creators to receive royalties.
    pub creators: Vec<Creator>, // 4
    /// The rule set for the asset to enforce royalties.
    pub rule_set: RuleSet, // 1
}

impl DataBlob for Royalties {
    fn len(&self) -> usize {
        2 // basis_points
        + 4 // creators length
        + self.creators.iter().map(|creator| creator.len()).sum::<usize>()
        + self.rule_set.len() // rule_set
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
            RuleSet::None => abstain!(),
            RuleSet::ProgramAllowList(allow_list) => {
                if allow_list.contains(ctx.authority_info.owner)
                    && allow_list.contains(new_owner.owner)
                {
                    abstain!()
                } else {
                    reject!()
                }
            }
            RuleSet::ProgramDenyList(deny_list) => {
                if deny_list.contains(ctx.authority_info.owner)
                    || deny_list.contains(new_owner.owner)
                {
                    reject!()
                } else {
                    abstain!()
                }
            }
        }
    }

    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        match ctx.target_plugin {
            Some(Plugin::Royalties(_royalties)) => validate_royalties(self),
            _ => abstain!(),
        }
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
                abstain!()
            }
        } else {
            abstain!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creator_len() {
        let creator = Creator {
            address: Pubkey::default(),
            percentage: 100,
        };
        let serialized = creator.try_to_vec().unwrap();
        assert_eq!(serialized.len(), creator.len());
    }

    #[test]
    fn test_rule_set_default_len() {
        let rule_set = RuleSet::None;
        let serialized = rule_set.try_to_vec().unwrap();
        assert_eq!(serialized.len(), rule_set.len());
    }

    #[test]
    fn test_rule_set_len() {
        let rule_sets = vec![
            RuleSet::ProgramAllowList(vec![Pubkey::default()]),
            RuleSet::ProgramDenyList(vec![Pubkey::default(), Pubkey::default()]),
        ];
        for rule_set in rule_sets {
            let serialized = rule_set.try_to_vec().unwrap();
            assert_eq!(serialized.len(), rule_set.len());
        }
    }

    #[test]
    fn test_royalties_len() {
        let royalties = vec![
            Royalties {
                basis_points: 0,
                creators: vec![],
                rule_set: RuleSet::None,
            },
            Royalties {
                basis_points: 1,
                creators: vec![Creator {
                    address: Pubkey::default(),
                    percentage: 1,
                }],
                rule_set: RuleSet::ProgramAllowList(vec![]),
            },
            Royalties {
                basis_points: 2,
                creators: vec![
                    Creator {
                        address: Pubkey::default(),
                        percentage: 2,
                    },
                    Creator {
                        address: Pubkey::default(),
                        percentage: 3,
                    },
                ],
                rule_set: RuleSet::ProgramDenyList(vec![Pubkey::default()]),
            },
            Royalties {
                basis_points: 3,
                creators: vec![
                    Creator {
                        address: Pubkey::default(),
                        percentage: 3,
                    },
                    Creator {
                        address: Pubkey::default(),
                        percentage: 4,
                    },
                    Creator {
                        address: Pubkey::default(),
                        percentage: 5,
                    },
                ],
                rule_set: RuleSet::ProgramDenyList(vec![Pubkey::default(), Pubkey::default()]),
            },
        ];
        for royalty in royalties {
            let serialized = royalty.try_to_vec().unwrap();
            assert_eq!(serialized.len(), royalty.len());
        }
    }
}
