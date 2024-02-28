use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        BurnAccounts, CompressAccounts, DecompressAccounts, TransferAccounts, UpdateAccounts,
    },
    plugins::{CheckResult, ValidationResult},
    state::{
        Compressible, CompressionProof, DataBlob, HashedAsset, HashedAssetSchema, Key,
        SolanaAccount,
    },
};

use super::CoreAsset;

/// The Core Asset structure that exists at the beginning of every asset account.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount, Eq, PartialEq)]
pub struct Asset {
    /// The account discriminator.
    pub key: Key, //1
    /// The update authority of the asset.
    pub update_authority: Pubkey, //32
    /// The owner of the asset.
    pub owner: Pubkey, //32
    /// The name of the asset.
    pub name: String, //4
    /// The URI of the asset that points to the off-chain data.
    pub uri: String, //4
}

impl Asset {
    /// The base length of the asset account with an empty name and uri.
    pub const BASE_LENGTH: usize = 1 + 32 + 32 + 4 + 4;

    /// Check that a compression proof results in same on-chain hash.
    pub fn verify_proof(
        hashed_asset: &AccountInfo,
        compression_proof: CompressionProof,
    ) -> Result<Asset, ProgramError> {
        let asset = Self::from(compression_proof);
        let asset_hash = asset.hash()?;

        // TODO: this currently assumes no plugins.
        let hashed_asset_schema = HashedAssetSchema {
            asset_hash,
            plugin_hashes: vec![],
        };

        let hashed_asset_schema_hash = hashed_asset_schema.hash()?;

        let current_account_hash = HashedAsset::load(hashed_asset, 0)?.hash;
        if hashed_asset_schema_hash != current_account_hash {
            return Err(MplCoreError::IncorrectAssetHash.into());
        }

        Ok(asset)
    }

    /// Check permissions for the transfer lifecycle event.
    pub fn check_transfer() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Check permissions for the update lifecycle event.
    pub fn check_update() -> CheckResult {
        CheckResult::CanApprove
    }

    /// Validate the update lifecycle event.
    pub fn validate_update(&self, ctx: &UpdateAccounts) -> Result<ValidationResult, ProgramError> {
        if ctx.authority.key == &self.update_authority {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the burn lifecycle event.
    pub fn validate_burn(&self, ctx: &BurnAccounts) -> Result<ValidationResult, ProgramError> {
        if ctx.authority.key == &self.owner {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the transfer lifecycle event.
    pub fn validate_transfer(
        &self,
        ctx: &TransferAccounts,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.authority.key == &self.owner {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the compress lifecycle event.
    pub fn validate_compress(
        &self,
        ctx: &CompressAccounts,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.owner.key == &self.owner {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the decompress lifecycle event.
    pub fn validate_decompress(
        &self,
        ctx: &DecompressAccounts,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.owner.key == &self.owner {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}

impl Compressible for Asset {}

impl DataBlob for Asset {
    fn get_initial_size() -> usize {
        Asset::BASE_LENGTH
    }

    fn get_size(&self) -> usize {
        Asset::BASE_LENGTH + self.name.len() + self.uri.len()
    }
}

impl SolanaAccount for Asset {
    fn key() -> Key {
        Key::Asset
    }
}

impl From<CompressionProof> for Asset {
    fn from(compression_proof: CompressionProof) -> Self {
        Self {
            key: Self::key(),
            update_authority: compression_proof.update_authority,
            owner: compression_proof.owner,
            name: compression_proof.name,
            uri: compression_proof.uri,
        }
    }
}

impl CoreAsset for Asset {
    fn update_authority(&self) -> &Pubkey {
        &self.update_authority
    }

    fn owner(&self) -> &Pubkey {
        &self.owner
    }
}
