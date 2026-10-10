use std::collections::BTreeSet;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    plugins::{fetch_wrapped_plugin, reject, PluginType},
    state::{AssetV1, Authority, DataBlob, UpdateAuthority},
};

use crate::plugins::{
    abstain, approve, Plugin, PluginValidation, PluginValidationContext, ValidationResult,
};

/// This plugin manages additional permissions to burn.
/// Any authorities approved are given permission to burn the asset on behalf of the owner.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct UpdateDelegate {
    /// Additional update delegates.  Not currently available to be used.
    pub additional_delegates: Vec<Pubkey>, // 4 + len * 32
}

impl UpdateDelegate {
    const BASE_LEN: usize = 4; // The additional delegates length

    /// Initialize the UpdateDelegate plugin.
    pub fn new() -> Self {
        Self {
            additional_delegates: vec![],
        }
    }
}

impl Default for UpdateDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for UpdateDelegate {
    fn len(&self) -> usize {
        Self::BASE_LEN + self.additional_delegates.len() * 32
    }
}

impl PluginValidation for UpdateDelegate {
    fn validate_create(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(resolved_authorities) = ctx.resolved_authorities {
            if resolved_authorities.contains(ctx.self_authority) {
                return approve!();
            }
        }

        if self.additional_delegates.contains(ctx.authority_info.key) {
            return approve!();
        }

        abstain!()
    }

    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(new_plugin) = ctx.target_plugin {
            if ((ctx.resolved_authorities.is_some()
                && ctx
                    .resolved_authorities
                    .unwrap()
                    .contains(ctx.self_authority))
                || self.additional_delegates.contains(ctx.authority_info.key))
                && new_plugin.manager() == Authority::UpdateAuthority
            {
                approve!()
            } else {
                abstain!()
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    fn validate_remove_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(plugin_to_remove) = ctx.target_plugin {
            if ((ctx.resolved_authorities.is_some()
                && ctx
                    .resolved_authorities
                    .unwrap()
                    .contains(ctx.self_authority))
                || self.additional_delegates.contains(ctx.authority_info.key))
                && plugin_to_remove.manager() == Authority::UpdateAuthority
            {
                approve!()
            } else {
                abstain!()
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    // Validate the approve plugin authority lifecycle action.
    fn validate_approve_plugin_authority(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin = ctx.target_plugin.ok_or(MplCoreError::InvalidPlugin)?;

        // If the plugin authority is the authority signing.
        if ((ctx.resolved_authorities.is_some()
        && ctx
            .resolved_authorities
            .unwrap()
            .contains(ctx.self_authority))
            // Or the authority is one of the additional delegates.
            || self.additional_delegates.contains(ctx.authority_info.key))
            // And it's an authority-managed plugin.
            && plugin.manager() == Authority::UpdateAuthority
            // And the plugin is not an UpdateDelegate plugin, because we cannot change the authority of the UpdateDelegate plugin.
            && PluginType::from(plugin) != PluginType::UpdateDelegate
        {
            solana_program::msg!("UpdateDelegate: Approved");
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin = ctx.target_plugin.ok_or(MplCoreError::InvalidPlugin)?;

        // SECURITY FIX: Added explicit parentheses to fix operator precedence.
        // Previously, `&& plugin.manager() == Authority::UpdateAuthority` only bound
        // to the additional_delegates branch due to && having higher precedence than ||.
        // This allowed UpdateDelegate to revoke authority on owner-managed plugins
        // (FreezeDelegate, TransferDelegate) which it should not control.
        if ((ctx.resolved_authorities.is_some()
        && ctx
            .resolved_authorities
            .unwrap()
            .contains(ctx.self_authority))
            // Or the authority is one of the additional delegates.
            || (self.additional_delegates.contains(ctx.authority_info.key) && PluginType::from(plugin) != PluginType::UpdateDelegate))
            // And it's an authority-managed plugin (applies to BOTH branches).
            && plugin.manager() == Authority::UpdateAuthority
        {
            approve!()
        } else {
            abstain!()
        }
    }

    fn validate_update(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if (ctx.resolved_authorities.is_some()
            && ctx
                .resolved_authorities
                .unwrap()
                .contains(ctx.self_authority))
            || self.additional_delegates.contains(ctx.authority_info.key)
        {
            // The rules are:
            // - Collection UpdateDelegates should be able to add and remove assets from the collection
            // - Asset UpdateDelegates should not be able to add/remove assets from a collection
            // so:

            // If there is an asset
            if ctx.asset_info.is_some()
            // And it's part of a collection
                && ctx.collection_info.is_some()
                // And it's being removed from the collection.
                && ctx.new_asset_authority.is_some()
                && ctx.new_asset_authority.unwrap()
                    != &UpdateAuthority::Collection(*ctx.collection_info.unwrap().key)
                    // And the UpdateDelegate plugin is on the Asset, not the collection.
                && fetch_wrapped_plugin::<AssetV1>(
                    ctx.asset_info.unwrap(),
                    None,
                    PluginType::UpdateDelegate,
                )
                .is_ok()
            {
                // Then we reject.
                reject!()
            } else {
                // Otherwise, we approve.
                approve!()
            }
        } else {
            abstain!()
        }
    }

    fn validate_update_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin = ctx.target_plugin.ok_or(MplCoreError::InvalidPlugin)?;

        // If the plugin itself is being updated.
        if (ctx.resolved_authorities.is_some()
            && ctx
                .resolved_authorities
                .unwrap()
                .contains(ctx.self_authority))
            || self.additional_delegates.contains(ctx.authority_info.key)
        {
            if let Plugin::UpdateDelegate(update_delegate) = plugin {
                let existing: BTreeSet<_> = self.additional_delegates.iter().collect();
                let new: BTreeSet<_> = update_delegate.additional_delegates.iter().collect();

                if existing.difference(&new).collect::<Vec<_>>() == vec![&ctx.authority_info.key]
                    && new.difference(&existing).collect::<Vec<_>>().is_empty()
                {
                    return approve!();
                }
            }
            // UpdateDelegate has the same authority as UpdateAuthority, so if the target plugin authority is UpdateAuthority, we can approve.
            else if ctx.target_plugin_authority == Some(&Authority::UpdateAuthority) {
                return approve!();
            }
        }

        // Otherwise, abstain.
        abstain!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_delegate_default_len() {
        let update_delegate = UpdateDelegate::default();
        let serialized = borsh::to_vec(&update_delegate).unwrap();
        assert_eq!(serialized.len(), update_delegate.len());
    }

    #[test]
    fn test_update_delegate_len() {
        let update_delegates = vec![
            UpdateDelegate {
                additional_delegates: vec![Pubkey::default()],
            },
            UpdateDelegate {
                additional_delegates: vec![Pubkey::default(), Pubkey::default()],
            },
        ];
        for update_delegate in update_delegates {
            let serialized = borsh::to_vec(&update_delegate).unwrap();
            assert_eq!(serialized.len(), update_delegate.len());
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
                Attributes, FreezeDelegate, PluginHeaderV1, PluginRegistryV1, RegistryRecord,
            },
            state::Key,
        },
    };

    fn delegate(additional: &[Pubkey]) -> UpdateDelegate {
        UpdateDelegate {
            additional_delegates: additional.to_vec(),
        }
    }

    fn attributes() -> Plugin {
        Plugin::Attributes(Attributes {
            attribute_list: vec![],
        })
    }

    fn freeze() -> Plugin {
        Plugin::FreezeDelegate(FreezeDelegate { frozen: false })
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

    /// Serialized bytes of an `AssetV1` carrying one `UpdateDelegate` plugin,
    /// laid out the way `initialize_plugin` does.
    fn asset_with_update_delegate(owner: Pubkey) -> Vec<u8> {
        let asset = AssetV1 {
            key: Key::AssetV1,
            owner,
            update_authority: UpdateAuthority::Address(owner),
            name: "Test Asset".to_string(),
            uri: "https://example.com/test".to_string(),
            seq: None,
        };
        let mut data = borsh::to_vec(&asset).unwrap();
        let plugin_offset = data.len() + 9;
        let plugin_bytes = borsh::to_vec(&Plugin::UpdateDelegate(delegate(&[]))).unwrap();
        let registry = PluginRegistryV1 {
            key: Key::PluginRegistryV1,
            registry: vec![RegistryRecord {
                plugin_type: PluginType::UpdateDelegate,
                authority: Authority::UpdateAuthority,
                offset: plugin_offset,
            }],
            external_registry: vec![],
        };
        let header = PluginHeaderV1 {
            key: Key::PluginHeaderV1,
            plugin_registry_offset: plugin_offset + plugin_bytes.len(),
        };
        data.extend(borsh::to_vec(&header).unwrap());
        data.extend(plugin_bytes);
        data.extend(borsh::to_vec(&registry).unwrap());
        data
    }

    /// Serialized bytes of a bare `AssetV1` with no plugins.
    fn bare_asset(owner: Pubkey) -> Vec<u8> {
        borsh::to_vec(&AssetV1 {
            key: Key::AssetV1,
            owner,
            update_authority: UpdateAuthority::Address(owner),
            name: "Test Asset".to_string(),
            uri: "https://example.com/test".to_string(),
            seq: None,
        })
        .unwrap()
    }

    /// `validate_create` can only ever return `Approved` or `Pass`, and the
    /// create processors discard `Approved`, so the whole callback is inert
    /// (roadmap section 12, observation 3). Both branches are covered here.
    #[test]
    fn validate_create_is_inert_but_both_branches_work() {
        let signer_key = Pubkey::new_unique();
        let self_authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet_at(signer_key);
        let signer_info = signer.info();

        // The `resolved_authorities` branch, which no call site can reach.
        let resolved = [Authority::UpdateAuthority];
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        assert_eq!(
            delegate(&[]).validate_create(&ctx).unwrap(),
            ValidationResult::Approved
        );

        // The additional-delegates branch.
        let ctx = default_ctx(&[], &signer_info, &self_authority);
        assert_eq!(
            delegate(&[signer_key]).validate_create(&ctx).unwrap(),
            ValidationResult::Approved
        );

        // Neither.
        assert_eq!(
            delegate(&[]).validate_create(&ctx).unwrap(),
            ValidationResult::Pass
        );
    }

    #[test]
    fn add_and_remove_plugin_only_approve_authority_managed_targets() {
        let signer_key = Pubkey::new_unique();
        let self_authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet_at(signer_key);
        let signer_info = signer.info();
        let resolved = [Authority::UpdateAuthority];

        let ua_managed = attributes();
        let owner_managed = freeze();

        // As the plugin authority.
        for (target, expected) in [
            (&ua_managed, ValidationResult::Approved),
            (&owner_managed, ValidationResult::Pass),
        ] {
            let mut ctx = default_ctx(&[], &signer_info, &self_authority);
            ctx.resolved_authorities = Some(&resolved);
            ctx.target_plugin = Some(target);
            assert_eq!(delegate(&[]).validate_add_plugin(&ctx).unwrap(), expected);
            assert_eq!(
                delegate(&[]).validate_remove_plugin(&ctx).unwrap(),
                expected
            );
        }

        // As an additional delegate, with no resolved authority match.
        let unrelated = [Authority::Owner];
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&unrelated);
        ctx.target_plugin = Some(&ua_managed);
        assert_eq!(
            delegate(&[signer_key]).validate_add_plugin(&ctx).unwrap(),
            ValidationResult::Approved
        );
        assert_eq!(
            delegate(&[]).validate_add_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );

        // No target at all: unreachable from the processors.
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        assert_eq!(
            core_err(delegate(&[]).validate_add_plugin(&ctx)),
            MplCoreError::InvalidPlugin
        );
        assert_eq!(
            core_err(delegate(&[]).validate_remove_plugin(&ctx)),
            MplCoreError::InvalidPlugin
        );
    }

    #[test]
    fn approve_and_revoke_authority_exclude_the_update_delegate_itself() {
        let signer_key = Pubkey::new_unique();
        let self_authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet_at(signer_key);
        let signer_info = signer.info();
        let resolved = [Authority::UpdateAuthority];
        let unrelated = [Authority::Owner];

        let ua_managed = attributes();
        let owner_managed = freeze();
        let self_target = Plugin::UpdateDelegate(delegate(&[]));

        // Approve: the plugin authority may act on any UA-managed plugin
        // except an UpdateDelegate.
        for (target, expected) in [
            (&ua_managed, ValidationResult::Approved),
            (&owner_managed, ValidationResult::Pass),
            (&self_target, ValidationResult::Pass),
        ] {
            let mut ctx = default_ctx(&[], &signer_info, &self_authority);
            ctx.resolved_authorities = Some(&resolved);
            ctx.target_plugin = Some(target);
            assert_eq!(
                delegate(&[])
                    .validate_approve_plugin_authority(&ctx)
                    .unwrap(),
                expected
            );
        }

        // Revoke: after the precedence fix the manager check binds to both
        // branches, so an owner-managed target is never approved — not by the
        // plugin authority and not by an additional delegate.
        for (resolved_set, additional) in
            [(&resolved[..], vec![]), (&unrelated[..], vec![signer_key])]
        {
            let mut ctx = default_ctx(&[], &signer_info, &self_authority);
            ctx.resolved_authorities = Some(resolved_set);
            ctx.target_plugin = Some(&owner_managed);
            assert_eq!(
                delegate(&additional)
                    .validate_revoke_plugin_authority(&ctx)
                    .unwrap(),
                ValidationResult::Pass,
                "owner-managed targets must never be approved"
            );

            ctx.target_plugin = Some(&ua_managed);
            assert_eq!(
                delegate(&additional)
                    .validate_revoke_plugin_authority(&ctx)
                    .unwrap(),
                ValidationResult::Approved
            );
        }

        // An additional delegate may not revoke on the UpdateDelegate itself,
        // while the plugin authority may.
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&unrelated);
        ctx.target_plugin = Some(&self_target);
        assert_eq!(
            delegate(&[signer_key])
                .validate_revoke_plugin_authority(&ctx)
                .unwrap(),
            ValidationResult::Pass
        );
        ctx.resolved_authorities = Some(&resolved);
        assert_eq!(
            delegate(&[])
                .validate_revoke_plugin_authority(&ctx)
                .unwrap(),
            ValidationResult::Approved
        );

        // No target: unreachable from the processors.
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        assert_eq!(
            core_err(delegate(&[]).validate_approve_plugin_authority(&ctx)),
            MplCoreError::InvalidPlugin
        );
        assert_eq!(
            core_err(delegate(&[]).validate_revoke_plugin_authority(&ctx)),
            MplCoreError::InvalidPlugin
        );
    }

    #[test]
    fn validate_update_rejects_only_an_asset_level_delegate_re_parenting_the_asset() {
        let owner = Pubkey::new_unique();
        let signer_key = Pubkey::new_unique();
        let self_authority = Authority::UpdateAuthority;
        let resolved = [Authority::UpdateAuthority];
        let new_authority = UpdateAuthority::Address(owner);

        let mut signer = FakeAccount::wallet_at(signer_key);
        let mut collection = FakeAccount::wallet();
        let mut asset = FakeAccount::with_data(asset_with_update_delegate(owner));
        let signer_info = signer.info();
        let collection_info = collection.info();
        let asset_info = asset.info();

        // Asset-level delegate moving the asset out of its collection.
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.asset_info = Some(&asset_info);
        ctx.collection_info = Some(&collection_info);
        ctx.new_asset_authority = Some(&new_authority);
        assert_eq!(
            delegate(&[]).validate_update(&ctx).unwrap(),
            ValidationResult::Rejected
        );

        // The same move when the asset carries no UpdateDelegate of its own:
        // the plugin must live on the collection, so it approves.
        let mut bare = FakeAccount::with_data(bare_asset(owner));
        let bare_info = bare.info();
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.asset_info = Some(&bare_info);
        ctx.collection_info = Some(&collection_info);
        ctx.new_asset_authority = Some(&new_authority);
        assert_eq!(
            delegate(&[]).validate_update(&ctx).unwrap(),
            ValidationResult::Approved
        );

        // A plain rename (no new authority) is approved.
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.asset_info = Some(&asset_info);
        ctx.collection_info = Some(&collection_info);
        assert_eq!(
            delegate(&[]).validate_update(&ctx).unwrap(),
            ValidationResult::Approved
        );

        // Moving the asset to the collection it is already in is not a
        // re-parenting, so it is approved.
        let same_collection = UpdateAuthority::Collection(*collection_info.key);
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&resolved);
        ctx.asset_info = Some(&asset_info);
        ctx.collection_info = Some(&collection_info);
        ctx.new_asset_authority = Some(&same_collection);
        assert_eq!(
            delegate(&[]).validate_update(&ctx).unwrap(),
            ValidationResult::Approved
        );

        // A signer that is neither the plugin authority nor an additional
        // delegate abstains.
        let unrelated = [Authority::Owner];
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&unrelated);
        ctx.asset_info = Some(&asset_info);
        assert_eq!(
            delegate(&[]).validate_update(&ctx).unwrap(),
            ValidationResult::Pass
        );
    }

    #[test]
    fn validate_update_plugin_allows_self_removal_and_update_authority_targets() {
        let signer_key = Pubkey::new_unique();
        let other_key = Pubkey::new_unique();
        let self_authority = Authority::UpdateAuthority;
        let mut signer = FakeAccount::wallet_at(signer_key);
        let signer_info = signer.info();
        let unrelated = [Authority::Owner];
        let stored = delegate(&[signer_key, other_key]);

        // Removing only itself.
        let self_removed = Plugin::UpdateDelegate(delegate(&[other_key]));
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&unrelated);
        ctx.target_plugin = Some(&self_removed);
        assert_eq!(
            stored.validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Approved
        );

        // Removing somebody else.
        let other_removed = Plugin::UpdateDelegate(delegate(&[signer_key]));
        ctx.target_plugin = Some(&other_removed);
        assert_eq!(
            stored.validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );

        // Adding a key.
        let added =
            Plugin::UpdateDelegate(delegate(&[signer_key, other_key, Pubkey::new_unique()]));
        ctx.target_plugin = Some(&added);
        assert_eq!(
            stored.validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );

        // A different plugin whose authority is `UpdateAuthority`.
        let target = attributes();
        let ua = Authority::UpdateAuthority;
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&unrelated);
        ctx.target_plugin = Some(&target);
        ctx.target_plugin_authority = Some(&ua);
        assert_eq!(
            stored.validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Approved
        );

        // The same plugin delegated elsewhere.
        let elsewhere = Authority::Address {
            address: Pubkey::new_unique(),
        };
        ctx.target_plugin_authority = Some(&elsewhere);
        assert_eq!(
            stored.validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );

        // A signer that is neither delegate nor plugin authority.
        let mut stranger = FakeAccount::wallet();
        let stranger_info = stranger.info();
        let mut ctx = default_ctx(&[], &stranger_info, &self_authority);
        ctx.resolved_authorities = Some(&unrelated);
        ctx.target_plugin = Some(&target);
        ctx.target_plugin_authority = Some(&ua);
        assert_eq!(
            stored.validate_update_plugin(&ctx).unwrap(),
            ValidationResult::Pass
        );

        // No target: unreachable from the processors.
        let mut ctx = default_ctx(&[], &signer_info, &self_authority);
        ctx.resolved_authorities = Some(&unrelated);
        assert_eq!(
            core_err(stored.validate_update_plugin(&ctx)),
            MplCoreError::InvalidPlugin
        );
    }
}
