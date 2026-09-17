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
        let serialized = borsh::to_vec(&creator).unwrap();
        assert_eq!(serialized.len(), creator.len());
    }

    #[test]
    fn test_rule_set_default_len() {
        let rule_set = RuleSet::None;
        let serialized = borsh::to_vec(&rule_set).unwrap();
        assert_eq!(serialized.len(), rule_set.len());
    }

    #[test]
    fn test_rule_set_len() {
        let rule_sets = vec![
            RuleSet::ProgramAllowList(vec![Pubkey::default()]),
            RuleSet::ProgramDenyList(vec![Pubkey::default(), Pubkey::default()]),
        ];
        for rule_set in rule_sets {
            let serialized = borsh::to_vec(&rule_set).unwrap();
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
            let serialized = borsh::to_vec(&royalty).unwrap();
            assert_eq!(serialized.len(), royalty.len());
        }
    }
}

#[cfg(test)]
mod validation_tests {
    use {
        super::*,
        crate::{
            plugins::{
                test_ctx::{default_ctx, FakeAccount},
                FreezeDelegate,
            },
            state::Authority,
        },
    };

    /// A `Royalties` plugin with the given basis points, creators and rule set.
    fn royalties(basis_points: u16, creators: &[(Pubkey, u8)], rule_set: RuleSet) -> Royalties {
        Royalties {
            basis_points,
            creators: creators
                .iter()
                .map(|(address, percentage)| Creator {
                    address: *address,
                    percentage: *percentage,
                })
                .collect(),
            rule_set,
        }
    }

    /// A valid one-creator `Royalties` with the given rule set.
    fn valid(rule_set: RuleSet) -> Royalties {
        royalties(500, &[(Pubkey::new_unique(), 100)], rule_set)
    }

    fn core_err(result: Result<ValidationResult, ProgramError>) -> MplCoreError {
        match result {
            Err(ProgramError::Custom(code)) => {
                num_traits::FromPrimitive::from_u32(code).expect("an MplCoreError code")
            }
            other => panic!("expected a custom program error, got {other:?}"),
        }
    }

    #[test]
    fn validate_royalties_rejects_invalid_settings() {
        let creator = Pubkey::new_unique();
        let other = Pubkey::new_unique();

        for candidate in [
            // Basis points above 100%.
            royalties(10001, &[(creator, 100)], RuleSet::None),
            // Percentages that do not add up to 100.
            royalties(500, &[(creator, 50), (other, 40)], RuleSet::None),
            // The same creator twice.
            royalties(500, &[(creator, 50), (creator, 50)], RuleSet::None),
            // No creators at all sums to 0, not 100.
            royalties(500, &[], RuleSet::None),
        ] {
            assert_eq!(
                core_err(validate_royalties(&candidate)),
                MplCoreError::InvalidPluginSetting,
                "{candidate:?} should be refused"
            );
        }

        // The percentage sum saturates, so 200 + 200 is 255, still not 100.
        assert_eq!(
            core_err(validate_royalties(&royalties(
                0,
                &[(creator, 200), (other, 200)],
                RuleSet::None
            ))),
            MplCoreError::InvalidPluginSetting
        );

        assert_eq!(
            validate_royalties(&valid(RuleSet::None)).unwrap(),
            ValidationResult::Pass
        );
        // Exactly 100% at the maximum basis points is valid.
        assert_eq!(
            validate_royalties(&royalties(10000, &[(creator, 100)], RuleSet::None)).unwrap(),
            ValidationResult::Pass
        );
    }

    #[test]
    fn validate_transfer_applies_the_rule_set_to_the_owning_programs() {
        let listed = Pubkey::new_unique();
        let authority = Authority::UpdateAuthority;

        // (rule set, signer's owning program, new owner's owning program, expected)
        let cases: Vec<(RuleSet, Pubkey, Pubkey, ValidationResult)> = vec![
            (
                RuleSet::None,
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                ValidationResult::Pass,
            ),
            // Both sides listed: allowed.
            (
                RuleSet::ProgramAllowList(vec![listed]),
                listed,
                listed,
                ValidationResult::Pass,
            ),
            // Only the signer listed: rejected.
            (
                RuleSet::ProgramAllowList(vec![listed]),
                listed,
                Pubkey::new_unique(),
                ValidationResult::Rejected,
            ),
            // Only the new owner listed: rejected.
            (
                RuleSet::ProgramAllowList(vec![listed]),
                Pubkey::new_unique(),
                listed,
                ValidationResult::Rejected,
            ),
            // The signer's program is denied.
            (
                RuleSet::ProgramDenyList(vec![listed]),
                listed,
                Pubkey::new_unique(),
                ValidationResult::Rejected,
            ),
            // The new owner's program is denied.
            (
                RuleSet::ProgramDenyList(vec![listed]),
                Pubkey::new_unique(),
                listed,
                ValidationResult::Rejected,
            ),
            // Neither is denied.
            (
                RuleSet::ProgramDenyList(vec![listed]),
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                ValidationResult::Pass,
            ),
            (
                RuleSet::ProgramDenyList(vec![]),
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                ValidationResult::Pass,
            ),
        ];

        for (rule_set, signer_owner, new_owner_owner, expected) in cases {
            let mut signer = FakeAccount::new(signer_owner);
            let mut new_owner = FakeAccount::new(new_owner_owner);
            let signer_info = signer.info();
            let new_owner_info = new_owner.info();
            let mut ctx = default_ctx(&[], &signer_info, &authority);
            ctx.new_owner = Some(&new_owner_info);

            assert_eq!(
                valid(rule_set.clone()).validate_transfer(&ctx).unwrap(),
                expected,
                "{rule_set:?} with signer owner {signer_owner} and new owner {new_owner_owner}"
            );
        }
    }

    /// `transfer.rs` always supplies a new owner, so this arm is unreachable
    /// from any instruction (roadmap section 12, dead-code table).
    #[test]
    fn validate_transfer_without_a_new_owner_errors() {
        let authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let ctx = default_ctx(&[], &signer_info, &authority);

        assert_eq!(
            core_err(valid(RuleSet::ProgramDenyList(vec![])).validate_transfer(&ctx)),
            MplCoreError::MissingNewOwner
        );
    }

    #[test]
    fn validate_add_plugin_only_looks_at_a_royalties_target() {
        let authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();

        // A different target: abstain, whatever `self` holds.
        let other = Plugin::FreezeDelegate(FreezeDelegate { frozen: false });
        let mut ctx = default_ctx(&[], &signer_info, &authority);
        ctx.target_plugin = Some(&other);
        assert_eq!(
            royalties(10001, &[], RuleSet::None)
                .validate_add_plugin(&ctx)
                .unwrap(),
            ValidationResult::Pass
        );

        // A Royalties target validates `self` rather than the target: with a
        // valid `self` the result is `Pass` even though the target is invalid.
        // See roadmap section 12, observation 1.
        let invalid_target = Plugin::Royalties(royalties(10001, &[], RuleSet::None));
        let mut ctx = default_ctx(&[], &signer_info, &authority);
        ctx.target_plugin = Some(&invalid_target);
        assert_eq!(
            valid(RuleSet::None).validate_add_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );
    }

    #[test]
    fn validate_update_plugin_requires_the_plugin_authority() {
        let self_authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();

        let invalid = Plugin::Royalties(royalties(10001, &[], RuleSet::None));
        let good = Plugin::Royalties(valid(RuleSet::None));
        let other = Plugin::FreezeDelegate(FreezeDelegate { frozen: false });

        // No target plugin at all (unreachable from the processors).
        let ctx = default_ctx(&[], &signer_info, &self_authority);
        assert_eq!(
            core_err(valid(RuleSet::None).validate_update_plugin(&ctx)),
            MplCoreError::InvalidPlugin
        );

        // No resolved authorities (unreachable from the processors).
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.target_plugin = Some(&good);
        assert_eq!(
            core_err(valid(RuleSet::None).validate_update_plugin(&ctx)),
            MplCoreError::InvalidAuthority
        );

        // The signer resolves to the plugin authority: the new data is checked.
        let resolved = [Authority::UpdateAuthority];
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.target_plugin = Some(&invalid);
        assert_eq!(
            core_err(valid(RuleSet::None).validate_update_plugin(&ctx)),
            MplCoreError::InvalidPluginSetting
        );
        ctx.target_plugin = Some(&good);
        assert_eq!(
            valid(RuleSet::None).validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );

        // The signer does not resolve to the plugin authority: abstain, even
        // for data that would be refused.
        let resolved = [Authority::Owner];
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.target_plugin = Some(&invalid);
        assert_eq!(
            valid(RuleSet::None).validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );

        // A different target: abstain.
        let resolved = [Authority::UpdateAuthority];
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.target_plugin = Some(&other);
        assert_eq!(
            valid(RuleSet::None).validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );
    }

    #[test]
    fn validate_create_checks_the_settings() {
        let authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet();
        let signer_info = signer.info();
        let ctx = default_ctx(&[], &signer_info, &authority);

        assert_eq!(
            valid(RuleSet::None).validate_create(&ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            core_err(royalties(10001, &[], RuleSet::None).validate_create(&ctx)),
            MplCoreError::InvalidPluginSetting
        );
    }
}
