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
        let serialized = borsh::to_vec(&autograph_signature).unwrap();
        assert_eq!(serialized.len(), autograph_signature.len());
    }

    #[test]
    fn test_autograph_default_len() {
        let autograph = Autograph { signatures: vec![] };
        let serialized = borsh::to_vec(&autograph).unwrap();
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
        let serialized = borsh::to_vec(&autograph).unwrap();
        assert_eq!(serialized.len(), autograph.len());
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

    fn autograph(entries: &[(Pubkey, &str)]) -> Autograph {
        Autograph {
            signatures: entries
                .iter()
                .map(|(address, message)| AutographSignature {
                    address: *address,
                    message: message.to_string(),
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
    fn validate_autograph_covers_every_arm() {
        let owner = Pubkey::new_unique();
        let third_party = Pubkey::new_unique();
        let stored = autograph(&[(owner, "first")]);

        // An existing signature's message may never be rewritten, not even by
        // the plugin authority.
        for is_plugin_authority in [true, false] {
            assert_eq!(
                core_err(validate_autograph(
                    &autograph(&[(owner, "edited")]),
                    Some(&stored),
                    &owner,
                    is_plugin_authority,
                )),
                MplCoreError::InvalidPluginOperation
            );
        }

        // A new signature must belong to the signer.
        assert_eq!(
            core_err(validate_autograph(
                &autograph(&[(owner, "first"), (third_party, "theirs")]),
                Some(&stored),
                &owner,
                true,
            )),
            MplCoreError::MissingSigner
        );

        // Duplicate addresses.
        assert_eq!(
            core_err(validate_autograph(
                &autograph(&[(owner, "a"), (owner, "a")]),
                None,
                &owner,
                true,
            )),
            MplCoreError::InvalidPluginSetting
        );

        // Only the plugin authority may drop an existing signature.
        assert_eq!(
            core_err(validate_autograph(
                &autograph(&[(third_party, "theirs")]),
                Some(&autograph(&[(owner, "first"), (third_party, "theirs")])),
                &third_party,
                false,
            )),
            MplCoreError::MissingSigner
        );
        assert_eq!(
            validate_autograph(
                &autograph(&[(third_party, "theirs")]),
                Some(&autograph(&[(owner, "first"), (third_party, "theirs")])),
                &owner,
                true,
            )
            .unwrap(),
            ValidationResult::Pass
        );

        // A third party appending its own signature is fine even without
        // authority.
        assert_eq!(
            validate_autograph(
                &autograph(&[(owner, "first"), (third_party, "theirs")]),
                Some(&stored),
                &third_party,
                false,
            )
            .unwrap(),
            ValidationResult::Pass
        );
    }

    #[test]
    fn validate_add_and_update_plugin_route_by_target() {
        let owner = Pubkey::new_unique();
        let self_authority = Authority::Owner;
        let mut signer = FakeAccount::wallet_at(owner);
        let signer_info = signer.info();
        let stored = autograph(&[(owner, "first")]);

        // A different target abstains on both callbacks.
        let other = Plugin::FreezeDelegate(FreezeDelegate { frozen: false });
        let resolved = [Authority::Owner];
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.target_plugin = Some(&other);
        assert_eq!(
            stored.validate_add_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            stored.validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );

        // An Autograph target is validated and then explicitly approved.
        let target = Plugin::Autograph(autograph(&[(owner, "first")]));
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.target_plugin = Some(&target);
        assert_eq!(
            stored.validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Approved
        );
        assert_eq!(
            autograph(&[(owner, "mine")])
                .validate_add_plugin(&ctx)
                .unwrap(),
            ValidationResult::Approved
        );

        // Without resolved authorities `validate_update_plugin` errors; this
        // is unreachable from the processors.
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.target_plugin = Some(&target);
        assert_eq!(
            core_err(stored.validate_update_plugin(&ctx)),
            MplCoreError::InvalidAuthority
        );
    }

    #[test]
    fn validate_create_treats_the_creator_as_the_plugin_authority() {
        let owner = Pubkey::new_unique();
        let self_authority = Authority::Owner;
        let mut signer = FakeAccount::wallet_at(owner);
        let signer_info = signer.info();
        let ctx = default_ctx(&[], &signer_info, &self_authority);

        assert_eq!(
            autograph(&[(owner, "mine")]).validate_create(&ctx).unwrap(),
            ValidationResult::Pass
        );
        assert_eq!(
            core_err(autograph(&[(Pubkey::new_unique(), "theirs")]).validate_create(&ctx)),
            MplCoreError::MissingSigner
        );
    }
}
