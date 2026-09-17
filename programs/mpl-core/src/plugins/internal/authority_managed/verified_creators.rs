use std::collections::{BTreeMap, HashSet};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::MplCoreError;

use crate::plugins::{
    abstain, Plugin, PluginValidation, PluginValidationContext, ValidationResult,
};
use crate::state::DataBlob;

/// The creator on an asset and whether or not they are verified.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Hash)]
pub struct VerifiedCreatorsSignature {
    /// The address of the creator.
    pub address: Pubkey, // 32
    /// Whether or not the creator is verified.
    pub verified: bool, // 1
}

impl VerifiedCreatorsSignature {
    const BASE_LEN: usize = 32 // The address
    + 1; // The verified boolean
}

impl DataBlob for VerifiedCreatorsSignature {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

/// Structure for storing verified creators, often used in conjunction with the Royalties plugin
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct VerifiedCreators {
    /// A list of signatures
    pub signatures: Vec<VerifiedCreatorsSignature>, // 4 + len * VerifiedCreatorsSignature
}

impl VerifiedCreators {
    const BASE_LEN: usize = 4; // The signatures length
}

impl DataBlob for VerifiedCreators {
    fn len(&self) -> usize {
        Self::BASE_LEN + self.signatures.iter().map(|sig| sig.len()).sum::<usize>()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verified_creators_signature_len() {
        let verified_creators_signature = VerifiedCreatorsSignature {
            address: Pubkey::default(),
            verified: false,
        };
        let serialized = borsh::to_vec(&verified_creators_signature).unwrap();
        assert_eq!(serialized.len(), verified_creators_signature.len());
    }

    #[test]
    fn test_verified_creators_default_len() {
        let verified_creators = VerifiedCreators { signatures: vec![] };
        let serialized = borsh::to_vec(&verified_creators).unwrap();
        assert_eq!(serialized.len(), verified_creators.len());
    }

    #[test]
    fn test_verified_creators_len() {
        let verified_creators = VerifiedCreators {
            signatures: vec![
                VerifiedCreatorsSignature {
                    address: Pubkey::default(),
                    verified: false,
                },
                VerifiedCreatorsSignature {
                    address: Pubkey::default(),
                    verified: true,
                },
            ],
        };
        let serialized = borsh::to_vec(&verified_creators).unwrap();
        assert_eq!(serialized.len(), verified_creators.len());
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

    fn signatures(entries: &[(Pubkey, bool)]) -> VerifiedCreators {
        VerifiedCreators {
            signatures: entries
                .iter()
                .map(|(address, verified)| VerifiedCreatorsSignature {
                    address: *address,
                    verified: *verified,
                })
                .collect(),
        }
    }

    fn core_err<T>(result: Result<T, ProgramError>) -> MplCoreError {
        match result {
            Err(ProgramError::Custom(code)) => {
                num_traits::FromPrimitive::from_u32(code).expect("an MplCoreError code")
            }
            Err(other) => panic!("expected a custom program error, got {other:?}"),
            Ok(_) => panic!("expected an error"),
        }
    }

    #[test]
    fn calculate_signature_changes_reports_added_changed_and_removed() {
        let a = Pubkey::new_unique();
        let b = Pubkey::new_unique();
        let c = Pubkey::new_unique();

        // No stored plugin: everything is an addition.
        let changes =
            calculate_signature_changes(&signatures(&[(a, false), (b, true)]), None).unwrap();
        assert_eq!(changes.added, vec![0, 1]);
        assert!(changes.changed.is_empty());
        assert!(changes.removed.is_empty());

        // `a` keeps its flag, `b` flips, `c` is new and the stored `a2` is gone.
        let stored = signatures(&[(a, false), (b, false), (c, true)]);
        let new = signatures(&[(a, false), (b, true), (Pubkey::new_unique(), false)]);
        let changes = calculate_signature_changes(&new, Some(&stored)).unwrap();
        assert!(changes.added.contains(&2), "the new address is an addition");
        assert_eq!(changes.changed, vec![1], "only `b` flipped");
        assert_eq!(changes.removed, vec![2], "`c` was dropped");

        // Duplicates are refused outright.
        assert_eq!(
            core_err(calculate_signature_changes(
                &signatures(&[(a, false), (a, true)]),
                None
            )),
            MplCoreError::InvalidPluginSetting
        );
    }

    #[test]
    fn a_creator_may_only_flip_its_own_flag() {
        let creator = Pubkey::new_unique();
        let other = Pubkey::new_unique();
        let stored = signatures(&[(creator, false), (other, false)]);

        // Flipping own flag either way is fine.
        for verified in [true, false] {
            assert_eq!(
                validate_verified_creators_as_creator(
                    &signatures(&[(creator, verified), (other, false)]),
                    &stored,
                    &creator,
                )
                .unwrap(),
                ValidationResult::Pass
            );
        }

        // Flipping somebody else's flag.
        assert_eq!(
            core_err(validate_verified_creators_as_creator(
                &signatures(&[(creator, false), (other, true)]),
                &stored,
                &creator,
            )),
            MplCoreError::MissingSigner
        );

        // Adding an entry.
        assert_eq!(
            core_err(validate_verified_creators_as_creator(
                &signatures(&[
                    (creator, false),
                    (other, false),
                    (Pubkey::new_unique(), false)
                ]),
                &stored,
                &creator,
            )),
            MplCoreError::MissingSigner
        );

        // Removing an entry.
        assert_eq!(
            core_err(validate_verified_creators_as_creator(
                &signatures(&[(creator, false)]),
                &stored,
                &creator,
            )),
            MplCoreError::MissingSigner
        );
    }

    #[test]
    fn the_plugin_authority_may_only_touch_unverified_entries_and_itself() {
        let authority = Pubkey::new_unique();
        let other = Pubkey::new_unique();

        // Adding itself verified and third parties unverified is allowed.
        assert_eq!(
            validate_verified_creators_as_plugin_authority(
                &signatures(&[(authority, true), (other, false)]),
                None,
                &authority,
            )
            .unwrap(),
            ValidationResult::Pass
        );

        // Adding a verified third party is not.
        assert_eq!(
            core_err(validate_verified_creators_as_plugin_authority(
                &signatures(&[(other, true)]),
                None,
                &authority,
            )),
            MplCoreError::MissingSigner
        );

        // Removing an unverified third party is allowed; removing a verified
        // one is not.
        let stored_unverified = signatures(&[(other, false)]);
        assert_eq!(
            validate_verified_creators_as_plugin_authority(
                &signatures(&[]),
                Some(&stored_unverified),
                &authority,
            )
            .unwrap(),
            ValidationResult::Pass
        );
        let stored_verified = signatures(&[(other, true)]);
        assert_eq!(
            core_err(validate_verified_creators_as_plugin_authority(
                &signatures(&[]),
                Some(&stored_verified),
                &authority,
            )),
            MplCoreError::InvalidPluginOperation
        );

        // Unverifying a third party is refused; unverifying itself is fine.
        assert_eq!(
            core_err(validate_verified_creators_as_plugin_authority(
                &signatures(&[(other, false)]),
                Some(&stored_verified),
                &authority,
            )),
            MplCoreError::InvalidPluginOperation
        );
        let stored_self_verified = signatures(&[(authority, true)]);
        assert_eq!(
            validate_verified_creators_as_plugin_authority(
                &signatures(&[(authority, false)]),
                Some(&stored_self_verified),
                &authority,
            )
            .unwrap(),
            ValidationResult::Pass
        );
        // ... and so is removing itself entirely.
        assert_eq!(
            validate_verified_creators_as_plugin_authority(
                &signatures(&[]),
                Some(&stored_self_verified),
                &authority,
            )
            .unwrap(),
            ValidationResult::Pass
        );
    }

    #[test]
    fn validate_update_plugin_routes_by_whether_the_signer_is_the_plugin_authority() {
        let creator = Pubkey::new_unique();
        let self_authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet_at(creator);
        let signer_info = signer.info();
        let stored = signatures(&[(creator, false)]);

        // No resolved authorities: unreachable from the processors.
        let target = Plugin::VerifiedCreators(signatures(&[(creator, true)]));
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.target_plugin = Some(&target);
        assert_eq!(
            core_err(stored.validate_update_plugin(&ctx)),
            MplCoreError::InvalidAuthority
        );

        // As the plugin authority: the plugin-authority rules apply, so
        // verifying a third party is refused.
        let resolved = [Authority::UpdateAuthority];
        let third_party = Plugin::VerifiedCreators(signatures(&[
            (creator, false),
            (Pubkey::new_unique(), true),
        ]));
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.target_plugin = Some(&third_party);
        assert_eq!(
            core_err(stored.validate_update_plugin(&ctx)),
            MplCoreError::MissingSigner
        );

        // As a creator: flipping its own flag is approved.
        let resolved = [Authority::Owner];
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.target_plugin = Some(&target);
        assert_eq!(
            stored.validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Approved
        );

        // Roadmap section 12, observation 2: a no-op update from an unrelated
        // signer is `Approved`.
        let unchanged = Plugin::VerifiedCreators(stored.clone());
        let mut stranger = FakeAccount::wallet();
        let stranger_info = stranger.info();
        let mut ctx = default_ctx(&[], &stranger_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.target_plugin = Some(&unchanged);
        assert_eq!(
            stored.validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Approved
        );

        // A different target abstains.
        let other = Plugin::FreezeDelegate(FreezeDelegate { frozen: false });
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.target_plugin = Some(&other);
        assert_eq!(
            stored.validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );
    }

    #[test]
    fn validate_add_plugin_only_looks_at_a_verified_creators_target() {
        let authority = Pubkey::new_unique();
        let self_authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet_at(authority);
        let signer_info = signer.info();

        let other = Plugin::FreezeDelegate(FreezeDelegate { frozen: false });
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.target_plugin = Some(&other);
        assert_eq!(
            signatures(&[(Pubkey::new_unique(), true)])
                .validate_add_plugin(&ctx)
                .unwrap(),
            ValidationResult::Pass
        );

        // The self-validation path: `self` is the plugin being added, and it
        // is checked against an empty stored list.
        let target = Plugin::VerifiedCreators(signatures(&[]));
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.target_plugin = Some(&target);
        assert_eq!(
            core_err(signatures(&[(Pubkey::new_unique(), true)]).validate_add_plugin(&ctx)),
            MplCoreError::MissingSigner
        );
        assert_eq!(
            signatures(&[(authority, true)])
                .validate_add_plugin(&ctx)
                .unwrap(),
            ValidationResult::Pass
        );
    }
}
