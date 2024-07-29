use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    instruction::accounts::{
        BurnV1Accounts, CompressV1Accounts, DecompressV1Accounts, TransferV1Accounts,
        UpdateV1Accounts,
    },
    plugins::{abstain, approve, reject, CheckResult, ValidationResult},
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

    /// Validate the update lifecycle event.
    pub fn validate_update(
        &self,
        ctx: &UpdateV1Accounts,
    ) -> Result<ValidationResult, ProgramError> {
        let authority = match self {
            Self::None => return reject!(),
            Self::Address(address) => address,
            Self::Collection(address) => address,
        };

        if ctx.authority.unwrap_or(ctx.payer).key == authority {
            approve!()
        } else {
            abstain!()
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
