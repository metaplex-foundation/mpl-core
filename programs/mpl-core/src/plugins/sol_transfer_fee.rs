use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program::invoke, program_error::ProgramError, system_instruction};

use crate::{error::MplCoreError, plugins::abstain};

use super::{PluginType, PluginValidation, PluginValidationContext, ValidationResult};

/// The SOL transfer fee plugin charges a fee for every transfer of the asset and stores it
/// on the collection.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Default, Debug, PartialEq, Eq)]
pub struct SolTransferFee {
    /// The amount of SOL to charge for every transfer, in lamports
    pub fee_amount: u64,
}

impl PluginValidation for SolTransferFee {
    fn validate_create(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // Transfer fees are stored on the collection, so we require a collection.
        if ctx.collection_info.is_none() {
            return Err(MplCoreError::PluginRequiresCollection.into());
        }

        // Target plugin doesn't need to be populated for create, so we check if it exists, otherwise we pass.
        if let Some(target_plugin) = ctx.target_plugin {
            // You can't create the SOL transfer fee plugin on a collection.
            if PluginType::from(target_plugin) == PluginType::SolTransferFee
                && ctx.asset_info.is_none()
            {
                Err(MplCoreError::PluginNotAllowedOnCollection.into())
            } else {
                abstain!()
            }
        } else {
            abstain!()
        }
    }

    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // Target plugin must be populated for add_plugin.
        let target_plugin = ctx.target_plugin.ok_or(MplCoreError::InvalidPlugin)?;

        // Transfer fees are stored on the collection, so we require a collection.
        if ctx.collection_info.is_none() {
            return Err(MplCoreError::PluginRequiresCollection.into());
        }

        // You can't add the SOL transfer fee plugin to a collection.
        if PluginType::from(target_plugin) == PluginType::SolTransferFee && ctx.asset_info.is_none()
        {
            Err(MplCoreError::PluginNotAllowedOnCollection.into())
        } else {
            abstain!()
        }
    }

    fn validate_transfer(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if self.fee_amount > 0 {
            let collection = ctx.collection_info.ok_or(MplCoreError::MissingCollection)?;

            invoke(
                &system_instruction::transfer(ctx.payer.key, collection.key, self.fee_amount),
                &[ctx.payer.clone(), collection.clone()],
            )?;
        }

        abstain!()
    }
}
