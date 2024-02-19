use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::accounts::{MasterEdition, Metadata};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplAssetError,
    instruction::accounts::MigrateAccounts,
    state::{DataState, MigrationLevel},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct MigrateArgs {
    pub data_state: DataState,
    pub level: MigrationLevel,
}

pub(crate) fn migrate<'a>(accounts: &'a [AccountInfo<'a>], args: MigrateArgs) -> ProgramResult {
    let ctx = MigrateAccounts::context(accounts)?;

    // Assert that the owner is the one performing the migration.
    assert_signer(ctx.accounts.payer)?;
    let authority = if let Some(owner) = ctx.accounts.owner {
        assert_signer(owner)?;
        owner
    } else {
        ctx.accounts.payer
    };

    let metadata = Metadata::safe_deserialize(&ctx.accounts.metadata.data.borrow())?;

    match metadata.collection_details {
        // If this is a collection NFT then we need to initialize migration for the whole collection.
        Some(_) => {}
        // Otherwise, we need to migrate the NFT
        None => {
            // Assert that the NFT is not a print edition.
            if ctx.accounts.edition.data.borrow()[1]
                == mpl_token_metadata::types::Key::EditionV1 as u8
            {
                return Err(MplAssetError::CannotMigratePrints.into());
            }

            // Assert that the NFT is not a master edition with a nonzero supply.
            let master_edition =
                MasterEdition::safe_deserialize(&ctx.accounts.edition.data.borrow())?;
            if master_edition.max_supply != Some(0) {
                return Err(MplAssetError::CannotMigrateMasterWithSupply.into());
            }
        }
    }

    Ok(())
}
