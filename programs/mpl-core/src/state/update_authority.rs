use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        BurnAccounts, CompressAccounts, CreateAccounts, DecompressAccounts, TransferAccounts,
        UpdateAccounts,
    },
    plugins::{fetch_plugin, CheckResult, PluginType, ValidationResult},
    processor::CreateArgs,
    state::{Authority, CollectionData, SolanaAccount},
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
    pub fn validate_create(
        &self,
        ctx: &CreateAccounts,
        _args: &CreateArgs,
    ) -> Result<ValidationResult, ProgramError> {
        match (ctx.collection, self) {
            // If you're trying to add a collection, then check the authority.
            (Some(collection_info), UpdateAuthority::Collection(collection_address)) => {
                if collection_info.key != collection_address {
                    return Err(MplCoreError::InvalidCollection.into());
                }
                let collection = CollectionData::load(collection_info, 0)?;
                solana_program::msg!("Collection: {:?}", collection);

                let authority = match ctx.authority {
                    Some(authority) => {
                        assert_signer(authority)?;
                        authority
                    }
                    None => ctx.payer,
                };

                let maybe_update_delegate =
                    fetch_plugin(collection_info, PluginType::UpdateDelegate);

                if let Ok((mut authorities, _, _)) = maybe_update_delegate {
                    authorities.push(Authority::UpdateAuthority);
                    if assert_collection_authority(&collection, authority, &authorities).is_err() {
                        return Ok(ValidationResult::Rejected);
                    }
                } else if authority.key != &collection.update_authority {
                    return Ok(ValidationResult::Rejected);
                }

                Ok(ValidationResult::Pass)
            }
            // If you're not trying add a collection, then just pass.
            (_, UpdateAuthority::Address(_)) => Ok(ValidationResult::Pass),
            // Otherwise reject because you're doing something weird.
            _ => Ok(ValidationResult::Rejected),
        }

        // let authority = match self {
        //     Self::None => return Ok(ValidationResult::Rejected),
        //     Self::Address(address) => address,
        //     Self::Collection(address) => address,
        // };

        // if ctx.authority.key == authority {
        //     Ok(ValidationResult::Approved)
        // } else {
        //     Ok(ValidationResult::Pass)
        // }
    }

    /// Validate the update lifecycle event.
    pub fn validate_update(&self, ctx: &UpdateAccounts) -> Result<ValidationResult, ProgramError> {
        let authority = match self {
            Self::None => return Ok(ValidationResult::Rejected),
            Self::Address(address) => address,
            Self::Collection(address) => address,
        };

        if ctx.authority.key == authority {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the burn lifecycle event.
    pub fn validate_burn(&self, _ctx: &BurnAccounts) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the transfer lifecycle event.
    pub fn validate_transfer(
        &self,
        _ctx: &TransferAccounts,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the compress lifecycle event.
    pub fn validate_compress(
        &self,
        _ctx: &CompressAccounts,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the decompress lifecycle event.
    pub fn validate_decompress(
        &self,
        _ctx: &DecompressAccounts,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }
}
