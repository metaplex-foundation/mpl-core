use crate::{error::MplCoreError, state::Key, utils::load_key};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, instruction::Instruction, keccak, msg,
    program::invoke, program_error::ProgramError, pubkey::Pubkey,
};

use super::UpdateAuthority;

/// A trait for generic blobs of data that have size.
#[allow(clippy::len_without_is_empty)]
pub trait DataBlob: BorshSerialize + BorshDeserialize {
    /// Get the current length of the data blob.
    fn len(&self) -> usize;
}

/// A trait for Solana accounts.
pub trait SolanaAccount: BorshSerialize + BorshDeserialize {
    /// Get the discriminator key for the account.
    fn key() -> Key;

    /// Load the account from the given account info starting at the offset.
    fn load(account: &AccountInfo, offset: usize) -> Result<Self, ProgramError> {
        let key = load_key(account, offset)?;

        if key != Self::key() {
            return Err(MplCoreError::DeserializationError.into());
        }

        if account.owner != &crate::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let mut bytes: &[u8] = &(*account.data).borrow()[offset..];
        Self::deserialize(&mut bytes).map_err(|error| {
            msg!("Error: {}", error);
            MplCoreError::DeserializationError.into()
        })
    }

    /// Save the account to the given account info starting at the offset.
    fn save(&self, account: &AccountInfo, offset: usize) -> ProgramResult {
        borsh::to_writer(&mut account.data.borrow_mut()[offset..], self).map_err(|error| {
            msg!("Error: {}", error);
            MplCoreError::SerializationError.into()
        })
    }
}

/// A trait for data that can be compressed.
pub trait Compressible: BorshSerialize + BorshDeserialize {
    /// Get the hash of the compressed data.
    fn hash(&self) -> Result<[u8; 32], ProgramError> {
        let serialized_data = borsh::to_vec(self)?;
        Ok(keccak::hash(serialized_data.as_slice()).to_bytes())
    }
}

/// A trait for data that can be wrapped by the SPL Noop program.
pub trait Wrappable: BorshSerialize + BorshDeserialize {
    /// Write the data to ledger state by wrapping it in a noop instruction.
    fn wrap(&self) -> ProgramResult {
        let serialized_data = borsh::to_vec(self)?;
        invoke(
            &Instruction {
                program_id: crate::SPL_NOOP_ID,
                accounts: vec![],
                data: serialized_data,
            },
            &[],
        )
    }
}

/// A trait for core assets.
pub trait CoreAsset {
    /// Get the update authority of the asset.
    fn update_authority(&self) -> UpdateAuthority;

    /// Get the owner of the asset.
    fn owner(&self) -> &Pubkey;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        state::{AssetV1, CollectionV1, HashedAssetSchema, HashedAssetV1},
        utils::test_account::TestAccount,
    };

    fn deserialization_error() -> ProgramError {
        MplCoreError::DeserializationError.into()
    }

    fn asset() -> AssetV1 {
        AssetV1::new(
            Pubkey::new_unique(),
            UpdateAuthority::Address(Pubkey::new_unique()),
            "name".to_string(),
            "uri".to_string(),
        )
    }

    // ---------------------------------------------------------------------
    // `SolanaAccount::load`
    // ---------------------------------------------------------------------

    #[test]
    fn load_round_trips_at_offset_zero() {
        let asset = asset();
        let mut account = TestAccount::owned(borsh::to_vec(&asset).unwrap());

        assert_eq!(AssetV1::load(&account.info(), 0), Ok(asset));
    }

    /// `load` reads from `offset`, which is how the plugin header and registry
    /// are read out of the tail of an asset account.
    #[test]
    fn load_round_trips_at_a_non_zero_offset() {
        let asset = asset();
        let mut data = vec![Key::Uninitialized as u8];
        data.extend_from_slice(&borsh::to_vec(&asset).unwrap());
        let mut account = TestAccount::owned(data);

        assert_eq!(AssetV1::load(&account.info(), 1), Ok(asset));
    }

    /// `HashedAssetV1::key()` is only reached through `HashedAssetV1::load`.
    #[test]
    fn load_round_trips_a_hashed_asset() {
        let hashed = HashedAssetV1::new([7; 32]);
        let mut account = TestAccount::owned(borsh::to_vec(&hashed).unwrap());

        assert_eq!(HashedAssetV1::load(&account.info(), 0), Ok(hashed));
    }

    /// A valid discriminator for a *different* account type: the key check
    /// fires before Borsh is given the chance to misread the payload.
    #[test]
    fn load_rejects_a_mismatched_discriminator() {
        let collection = CollectionV1::new(
            Pubkey::new_unique(),
            "name".to_string(),
            "uri".to_string(),
            0,
            0,
        );
        let mut account = TestAccount::owned(borsh::to_vec(&collection).unwrap());

        assert_eq!(
            AssetV1::load(&account.info(), 0),
            Err(deserialization_error())
        );
    }

    /// A byte that is not a `Key` at all fails earlier, inside `load_key`,
    /// with the same error.
    #[test]
    fn load_rejects_an_unknown_discriminator() {
        let mut account = TestAccount::owned(vec![u8::MAX; 64]);

        assert_eq!(
            AssetV1::load(&account.info(), 0),
            Err(deserialization_error())
        );
    }

    /// The discriminator is right but the payload stops short: Borsh fails and
    /// the error is mapped to `DeserializationError`.
    #[test]
    fn load_rejects_a_truncated_payload() {
        let mut data = borsh::to_vec(&asset()).unwrap();
        data.truncate(10);
        assert_eq!(data[0], Key::AssetV1 as u8);
        let mut account = TestAccount::owned(data);

        assert_eq!(
            AssetV1::load(&account.info(), 0),
            Err(deserialization_error())
        );
    }

    /// An account holding a well-formed asset but owned by another program is
    /// rejected with the runtime's own error, not a program error.
    #[test]
    fn load_rejects_an_account_owned_by_another_program() {
        let asset = asset();
        let mut account = TestAccount::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            borsh::to_vec(&asset).unwrap(),
        );

        assert_eq!(
            AssetV1::load(&account.info(), 0),
            Err(ProgramError::InvalidAccountOwner)
        );
    }

    // ---------------------------------------------------------------------
    // `SolanaAccount::save`
    // ---------------------------------------------------------------------

    #[test]
    fn save_writes_the_payload_at_the_offset() {
        let asset = asset();
        let serialized = borsh::to_vec(&asset).unwrap();
        let mut account = TestAccount::owned(vec![0; serialized.len() + 1]);

        {
            let info = account.info();
            assert_eq!(asset.save(&info, 1), Ok(()));
        }

        assert_eq!(account.data()[0], 0);
        assert_eq!(&account.data()[1..], serialized.as_slice());
    }

    /// Borsh writes into a fixed-size slice, so a buffer that cannot hold the
    /// whole payload surfaces as `SerializationError`. No caller can reach this
    /// today (every one resizes first or rewrites an equal-length payload).
    #[test]
    fn save_rejects_a_buffer_that_is_too_small() {
        let asset = asset();
        let mut account = TestAccount::owned(vec![0; 1]);
        let info = account.info();

        assert_eq!(
            asset.save(&info, 0),
            Err(MplCoreError::SerializationError.into())
        );
    }

    // ---------------------------------------------------------------------
    // `Compressible::hash`
    // ---------------------------------------------------------------------

    #[test]
    fn compressible_hash_is_keccak_of_the_borsh_encoding() {
        let asset = asset();
        let expected = keccak::hash(borsh::to_vec(&asset).unwrap().as_slice()).to_bytes();
        assert_eq!(asset.hash(), Ok(expected));

        let schema = HashedAssetSchema {
            asset_hash: expected,
            plugin_hashes: vec![[1; 32], [2; 32]],
        };
        let expected = keccak::hash(borsh::to_vec(&schema).unwrap().as_slice()).to_bytes();
        assert_eq!(schema.hash(), Ok(expected));
    }

    /// The hash is over the serialized bytes, so any field change moves it.
    #[test]
    fn compressible_hash_changes_with_the_payload() {
        let asset = asset();
        let mut renamed = asset.clone();
        renamed.name = "other".to_string();

        assert_ne!(asset.hash().unwrap(), renamed.hash().unwrap());
    }
}
