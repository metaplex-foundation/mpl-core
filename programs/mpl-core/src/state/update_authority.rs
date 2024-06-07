use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        BurnV1Accounts, CompressV1Accounts, CreateV2Accounts, DecompressV1Accounts,
        TransferV1Accounts,
    },
    plugins::{
        abstain, fetch_plugin, reject, CheckResult, PluginType, UpdateDelegate, ValidationResult,
    },
    processor::CreateV2Args,
    state::{Authority, CollectionV1, SolanaAccount},
    utils::assert_collection_authority,
};

/// An enum representing the types of accounts that can update data on an asset.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub enum UpdateAuthority {
    /// No update authority, used for immutability.
    None,
    /// A standard address or PDA.
    Address(Pubkey),
    /// Authority delegated to a collection.
    Collection(Pubkey),
}

impl UpdateAuthority {
    /// Get the address of the update authority.
    pub fn key(&self) -> Pubkey {
        match self {
            Self::None => Pubkey::default(),
            Self::Address(address) => *address,
            Self::Collection(address) => *address,
        }
    }

    /// Check permissions for the create lifecycle event.
    pub fn check_create() -> CheckResult {
        CheckResult::CanReject
    }

    /// Check permissions for the update lifecycle event.
    pub fn check_update() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Validate the create lifecycle event.
    pub(crate) fn validate_create(
        &self,
        ctx: &CreateV2Accounts,
        _args: &CreateV2Args,
    ) -> Result<ValidationResult, ProgramError> {
        match (ctx.collection, self) {
            // If you're trying to add a collection, then check the authority.
            (Some(collection_info), UpdateAuthority::Collection(collection_address)) => {
                if collection_info.key != collection_address {
                    return Err(MplCoreError::InvalidCollection.into());
                }
                let collection = CollectionV1::load(collection_info, 0)?;

                let authority_info = match ctx.authority {
                    Some(authority) => {
                        assert_signer(authority)?;
                        authority
                    }
                    None => ctx.payer,
                };

                let maybe_update_delegate = fetch_plugin::<CollectionV1, UpdateDelegate>(
                    collection_info,
                    PluginType::UpdateDelegate,
                );

                if let Ok((authority, update_delegate_plugin, _)) = maybe_update_delegate {
                    if assert_collection_authority(&collection, authority_info, &authority).is_err()
                        && assert_collection_authority(
                            &collection,
                            authority_info,
                            &Authority::UpdateAuthority,
                        )
                        .is_err()
                        && !update_delegate_plugin
                            .additional_delegates
                            .contains(authority_info.key)
                    {
                        solana_program::msg!("UA: Rejected");
                        return reject!();
                    }
                } else if authority_info.key != &collection.update_authority {
                    solana_program::msg!("UA: Rejected");
                    return reject!();
                }

                abstain!()
            }
            // If you're not trying add a collection, then just pass.
            (_, UpdateAuthority::Address(_)) => abstain!(),
            // Otherwise reject because you're doing something weird.
            _ => reject!(),
        }
    }

    /// Validate the burn lifecycle event.
    pub fn validate_burn(&self, _ctx: &BurnV1Accounts) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the transfer lifecycle event.
    pub fn validate_transfer(
        &self,
        _ctx: &TransferV1Accounts,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the compress lifecycle event.
    pub fn validate_compress(
        &self,
        _ctx: &CompressV1Accounts,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    /// Validate the decompress lifecycle event.
    pub fn validate_decompress(
        &self,
        _ctx: &DecompressV1Accounts,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }
}
