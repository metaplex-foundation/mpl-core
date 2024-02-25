use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::{
    instruction::accounts::{
        BurnAccounts, CompressAccounts, CreateAccounts, DecompressAccounts, TransferAccounts,
        UpdateAccounts,
    },
    processor::{BurnArgs, CompressArgs, CreateArgs, DecompressArgs, TransferArgs, UpdateArgs},
    state::Authority,
};

use super::PluginValidation;

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Creator {
    address: Pubkey,
    verified: bool,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub enum RuleSet {
    ProgramAllowList(Vec<Pubkey>),
    ProgramDenyList(Vec<Pubkey>),
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct Royalties {
    seller_fee_basis_points: u16,
    creators: Vec<Creator>,
    rule_set: RuleSet,
}

impl PluginValidation for Royalties {
    fn validate_create(
        &self,
        _ctx: &CreateAccounts,
        _args: &CreateArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        todo!()
    }

    fn validate_update(
        &self,
        _ctx: &UpdateAccounts,
        _args: &UpdateArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        todo!()
    }

    fn validate_burn(
        &self,
        _ctx: &BurnAccounts,
        _args: &BurnArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        todo!()
    }

    fn validate_transfer(
        &self,
        _ctx: &TransferAccounts,
        _args: &TransferArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        todo!()
    }

    fn validate_compress(
        &self,
        _ctx: &CompressAccounts,
        _args: &CompressArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        todo!()
    }

    fn validate_decompress(
        &self,
        _ctx: &DecompressAccounts,
        _args: &DecompressArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        todo!()
    }
}
