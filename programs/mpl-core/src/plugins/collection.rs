use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::pubkey::Pubkey;

use crate::{
    instruction::accounts::{
        BurnAccounts, CompressAccounts, CreateAccounts, DecompressAccounts, TransferAccounts,
        UpdateAccounts,
    },
    processor::{BurnArgs, CompressArgs, CreateArgs, DecompressArgs, TransferArgs, UpdateArgs},
    state::{Authority, CollectionData, DataBlob, SolanaAccount},
};

use super::{PluginValidation, ValidationResult};

/// This plugin manages a collection or grouping of assets.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct Collection {
    /// A pointer to the collection which the asset is a part of.
    collection_address: Pubkey, // 32
    /// This flag indicates if the collection is required when operating on the asset.
    /// Managed collections use the Collection as a parent which can store plugins that
    /// are applied to all assets in the collection by default. Plugins on the asset itself
    /// can override the collection plugins.
    managed: bool, // 1
}

impl DataBlob for Collection {
    fn get_initial_size() -> usize {
        33
    }

    fn get_size(&self) -> usize {
        33
    }
}

impl PluginValidation for Collection {
    fn validate_create(
        &self,
        ctx: &CreateAccounts,
        _args: &CreateArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        match ctx.collection {
            Some(collection) => {
                let collection = CollectionData::load(collection, 0)?;
                solana_program::msg!("Collection: {:?}", collection);
                // Check that the collection update authority is a signer.
                let authority = match ctx.update_authority {
                    Some(authority) => authority,
                    None => ctx.payer,
                };

                assert_signer(authority)?;

                if authority.key != &collection.update_authority {
                    return Ok(ValidationResult::Rejected);
                }

                Ok(ValidationResult::Pass)
            }
            None => {
                solana_program::msg!("No collection provided");
                Ok(ValidationResult::Rejected)
            }
        }
    }

    fn validate_update(
        &self,
        _ctx: &UpdateAccounts,
        _args: &UpdateArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_burn(
        &self,
        _ctx: &BurnAccounts,
        _args: &BurnArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_transfer(
        &self,
        _ctx: &TransferAccounts,
        _args: &TransferArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_compress(
        &self,
        _ctx: &CompressAccounts,
        _args: &CompressArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_decompress(
        &self,
        _ctx: &DecompressAccounts,
        _args: &DecompressArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }
}
