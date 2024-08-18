use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program::invoke, program_error::ProgramError, rent::Rent, system_instruction, sysvar::Sysvar,
};

use crate::error::MplCoreError;

use super::{
    abstain, Plugin, PluginType, PluginValidation, PluginValidationContext, ValidationResult,
};

/// The treasury plugin allows the creator to store a SOL treasury in the collection.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Default, Debug, PartialEq, Eq)]
pub struct Treasury {
    /// How much SOL has been withdrawn from the treasury, in lamports
    pub withdrawn: i64,
}

impl PluginValidation for Treasury {
    fn validate_create(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // Target plugin doesn't need to be populated for create, so we check if it exists, otherwise we pass.
        if let Some(Plugin::Treasury(target_plugin)) = ctx.target_plugin {
            // You can't create the treasury plugin on an asset.
            if ctx.asset_info.is_some() {
                Err(MplCoreError::PluginNotAllowedOnAsset.into())
            }
            // You can't create a treasury plugin with nonzero withdrawn amount.
            else if target_plugin.withdrawn > 0 {
                Err(MplCoreError::InvalidTreasuryWithdrawn.into())
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

        // You can't add the treasury plugin to an asset.
        if let Plugin::Treasury(target_plugin) = target_plugin {
            if ctx.asset_info.is_some() {
                Err(MplCoreError::PluginNotAllowedOnAsset.into())
            }
            // You can't add a treasury plugin with nonzero withdrawn amount.
            else if target_plugin.withdrawn > 0 {
                Err(MplCoreError::InvalidTreasuryWithdrawn.into())
            } else {
                abstain!()
            }
        } else {
            abstain!()
        }
    }

    fn validate_update_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(target_plugin) = ctx.target_plugin {
            if PluginType::from(target_plugin) == PluginType::Treasury {
                if let Plugin::Treasury(treasury) = target_plugin {
                    let collection = ctx.collection_info.ok_or(MplCoreError::MissingCollection)?;
                    // Withdrawing SOL from the treasury
                    if treasury.withdrawn > self.withdrawn {
                        let excess_rent = collection
                            .lamports()
                            .checked_sub(Rent::get()?.minimum_balance(collection.data_len()))
                            .ok_or(MplCoreError::NumericalOverflow)?;
                        let diff: u64 = treasury
                            .withdrawn
                            .checked_sub(self.withdrawn)
                            .ok_or(MplCoreError::NumericalOverflow)?
                            .try_into()
                            .map_err(|_| MplCoreError::NumericalOverflow)?;

                        if diff > excess_rent {
                            return Err(MplCoreError::CannotOverdraw.into());
                        }

                        let auth_starting_lamports = ctx.payer.lamports();
                        **ctx.payer.lamports.borrow_mut() =
                            auth_starting_lamports.checked_add(diff).unwrap();
                        **collection.lamports.borrow_mut() = collection
                            .lamports()
                            .checked_sub(diff)
                            .ok_or(MplCoreError::NumericalOverflow)?;
                    } else {
                        return Err(MplCoreError::InvalidPluginOperation.into());
                    }
                } else {
                    return Err(MplCoreError::InvalidPlugin.into());
                }
            }
        }
        abstain!()
    }
}
