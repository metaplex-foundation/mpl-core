#![feature(prelude_import)]
#![deny(missing_docs)]
//! # MPL Core
//!
//! Metaplex Core (“Core”) sheds the complexity and technical debt of previous
//! standards and provides a clean and simple core spec for digital assets.
//! This makes the bare minimum use case easy and understandable for users just
//! starting out with Digital Assets.
//!
//!However, it's designed with a flexible plugin system that allows for users
//! to extend the program itself without relying on Metaplex to add support to
//! a rigid standard like Token Metadata. The plugin system is so powerful that
//! it could even allow users to contribute third party plugins after the core
//! program is made immutable.
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
/// Standard Solana entrypoint.
pub mod entrypoint {
    use solana_program::{
        account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
        program_error::PrintProgramError, pubkey::Pubkey,
    };
    use crate::{error::MplCoreError, processor};
    /// # Safety
    #[no_mangle]
    pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
        let (program_id, accounts, instruction_data) = unsafe {
            ::solana_program::entrypoint::deserialize(input)
        };
        match process_instruction(&program_id, &accounts, &instruction_data) {
            Ok(()) => ::solana_program::entrypoint::SUCCESS,
            Err(error) => error.into(),
        }
    }
    /// Entrypoint function
    fn process_instruction<'a>(
        program_id: &'a Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> ProgramResult {
        if let Err(error) = processor::process_instruction(
            program_id,
            accounts,
            instruction_data,
        ) {
            error.print::<MplCoreError>();
            return Err(error);
        }
        Ok(())
    }
}
/// Error types for MPL Core.
pub mod error {
    use num_derive::FromPrimitive;
    use solana_program::{
        decode_error::DecodeError, msg, program_error::{PrintProgramError, ProgramError},
    };
    use thiserror::Error;
    /// Errors that may be returned by the Mpl Core program.
    pub enum MplCoreError {
        /// 0 - Invalid System Program
        #[error("Invalid System Program")]
        InvalidSystemProgram,
        /// 1 - Error deserializing account
        #[error("Error deserializing account")]
        DeserializationError,
        /// 2 - Error serializing account
        #[error("Error serializing account")]
        SerializationError,
        /// 3 - Plugins not initialized
        #[error("Plugins not initialized")]
        PluginsNotInitialized,
        /// 4 - Plugin not found
        #[error("Plugin not found")]
        PluginNotFound,
        /// 5 - Numerical Overflow
        #[error("Numerical Overflow")]
        NumericalOverflow,
        /// 6 - Incorrect account
        #[error("Incorrect account")]
        IncorrectAccount,
        /// 7 - Provided data does not match asset hash.
        #[error("Incorrect asset hash")]
        IncorrectAssetHash,
        /// 8 - Invalid Plugin
        #[error("Invalid Plugin")]
        InvalidPlugin,
        /// 9 - Invalid Authority
        #[error("Invalid Authority")]
        InvalidAuthority,
        /// 10 - Asset is frozen
        #[error("Cannot transfer a frozen asset")]
        AssetIsFrozen,
        /// 11 - Missing compression proof
        #[error("Missing compression proof")]
        MissingCompressionProof,
        /// 12 - Cannot migrate a master edition used for prints
        #[error("Cannot migrate a master edition used for prints")]
        CannotMigrateMasterWithSupply,
        /// 13 - Cannot migrate a print edition
        #[error("Cannot migrate a print edition")]
        CannotMigratePrints,
        /// 14 - Cannot burn a collection NFT
        #[error("Cannot burn a collection NFT")]
        CannotBurnCollection,
        /// 15 - Plugin already exists
        #[error("Plugin already exists")]
        PluginAlreadyExists,
        /// 16 - Numerical overflow
        #[error("Numerical overflow")]
        NumericalOverflowError,
        /// 17 - Already compressed account
        #[error("Already compressed account")]
        AlreadyCompressed,
        /// 18 - Already decompressed
        #[error("Already decompressed account")]
        AlreadyDecompressed,
        /// 19 - Invalid Collection passed in
        #[error("Invalid Collection passed in")]
        InvalidCollection,
        /// 20 - Missing update authority
        #[error("Missing update authority")]
        MissingUpdateAuthority,
        /// 21 - Missing new owner
        #[error("Missing new owner")]
        MissingNewOwner,
        /// 22 - Missing system program
        #[error("Missing system program")]
        MissingSystemProgram,
        /// 23 - Feature not available
        #[error("Feature not available")]
        NotAvailable,
        /// 24 - Invalid Asset passed in
        #[error("Invalid Asset passed in")]
        InvalidAsset,
        /// 25 - Missing collection
        #[error("Missing collection")]
        MissingCollection,
        /// 26 - Neither the asset or any plugins have approved this operation.
        #[error("Neither the asset or any plugins have approved this operation")]
        NoApprovals,
    }
    #[allow(unused_qualifications)]
    impl std::error::Error for MplCoreError {}
    #[allow(unused_qualifications)]
    impl ::core::fmt::Display for MplCoreError {
        fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
            match self {
                MplCoreError::InvalidSystemProgram {} => {
                    __formatter.write_fmt(format_args!("Invalid System Program"))
                }
                MplCoreError::DeserializationError {} => {
                    __formatter.write_fmt(format_args!("Error deserializing account"))
                }
                MplCoreError::SerializationError {} => {
                    __formatter.write_fmt(format_args!("Error serializing account"))
                }
                MplCoreError::PluginsNotInitialized {} => {
                    __formatter.write_fmt(format_args!("Plugins not initialized"))
                }
                MplCoreError::PluginNotFound {} => {
                    __formatter.write_fmt(format_args!("Plugin not found"))
                }
                MplCoreError::NumericalOverflow {} => {
                    __formatter.write_fmt(format_args!("Numerical Overflow"))
                }
                MplCoreError::IncorrectAccount {} => {
                    __formatter.write_fmt(format_args!("Incorrect account"))
                }
                MplCoreError::IncorrectAssetHash {} => {
                    __formatter.write_fmt(format_args!("Incorrect asset hash"))
                }
                MplCoreError::InvalidPlugin {} => {
                    __formatter.write_fmt(format_args!("Invalid Plugin"))
                }
                MplCoreError::InvalidAuthority {} => {
                    __formatter.write_fmt(format_args!("Invalid Authority"))
                }
                MplCoreError::AssetIsFrozen {} => {
                    __formatter.write_fmt(format_args!("Cannot transfer a frozen asset"))
                }
                MplCoreError::MissingCompressionProof {} => {
                    __formatter.write_fmt(format_args!("Missing compression proof"))
                }
                MplCoreError::CannotMigrateMasterWithSupply {} => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Cannot migrate a master edition used for prints",
                            ),
                        )
                }
                MplCoreError::CannotMigratePrints {} => {
                    __formatter.write_fmt(format_args!("Cannot migrate a print edition"))
                }
                MplCoreError::CannotBurnCollection {} => {
                    __formatter.write_fmt(format_args!("Cannot burn a collection NFT"))
                }
                MplCoreError::PluginAlreadyExists {} => {
                    __formatter.write_fmt(format_args!("Plugin already exists"))
                }
                MplCoreError::NumericalOverflowError {} => {
                    __formatter.write_fmt(format_args!("Numerical overflow"))
                }
                MplCoreError::AlreadyCompressed {} => {
                    __formatter.write_fmt(format_args!("Already compressed account"))
                }
                MplCoreError::AlreadyDecompressed {} => {
                    __formatter.write_fmt(format_args!("Already decompressed account"))
                }
                MplCoreError::InvalidCollection {} => {
                    __formatter.write_fmt(format_args!("Invalid Collection passed in"))
                }
                MplCoreError::MissingUpdateAuthority {} => {
                    __formatter.write_fmt(format_args!("Missing update authority"))
                }
                MplCoreError::MissingNewOwner {} => {
                    __formatter.write_fmt(format_args!("Missing new owner"))
                }
                MplCoreError::MissingSystemProgram {} => {
                    __formatter.write_fmt(format_args!("Missing system program"))
                }
                MplCoreError::NotAvailable {} => {
                    __formatter.write_fmt(format_args!("Feature not available"))
                }
                MplCoreError::InvalidAsset {} => {
                    __formatter.write_fmt(format_args!("Invalid Asset passed in"))
                }
                MplCoreError::MissingCollection {} => {
                    __formatter.write_fmt(format_args!("Missing collection"))
                }
                MplCoreError::NoApprovals {} => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Neither the asset or any plugins have approved this operation",
                            ),
                        )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for MplCoreError {
        #[inline]
        fn clone(&self) -> MplCoreError {
            match self {
                MplCoreError::InvalidSystemProgram => MplCoreError::InvalidSystemProgram,
                MplCoreError::DeserializationError => MplCoreError::DeserializationError,
                MplCoreError::SerializationError => MplCoreError::SerializationError,
                MplCoreError::PluginsNotInitialized => {
                    MplCoreError::PluginsNotInitialized
                }
                MplCoreError::PluginNotFound => MplCoreError::PluginNotFound,
                MplCoreError::NumericalOverflow => MplCoreError::NumericalOverflow,
                MplCoreError::IncorrectAccount => MplCoreError::IncorrectAccount,
                MplCoreError::IncorrectAssetHash => MplCoreError::IncorrectAssetHash,
                MplCoreError::InvalidPlugin => MplCoreError::InvalidPlugin,
                MplCoreError::InvalidAuthority => MplCoreError::InvalidAuthority,
                MplCoreError::AssetIsFrozen => MplCoreError::AssetIsFrozen,
                MplCoreError::MissingCompressionProof => {
                    MplCoreError::MissingCompressionProof
                }
                MplCoreError::CannotMigrateMasterWithSupply => {
                    MplCoreError::CannotMigrateMasterWithSupply
                }
                MplCoreError::CannotMigratePrints => MplCoreError::CannotMigratePrints,
                MplCoreError::CannotBurnCollection => MplCoreError::CannotBurnCollection,
                MplCoreError::PluginAlreadyExists => MplCoreError::PluginAlreadyExists,
                MplCoreError::NumericalOverflowError => {
                    MplCoreError::NumericalOverflowError
                }
                MplCoreError::AlreadyCompressed => MplCoreError::AlreadyCompressed,
                MplCoreError::AlreadyDecompressed => MplCoreError::AlreadyDecompressed,
                MplCoreError::InvalidCollection => MplCoreError::InvalidCollection,
                MplCoreError::MissingUpdateAuthority => {
                    MplCoreError::MissingUpdateAuthority
                }
                MplCoreError::MissingNewOwner => MplCoreError::MissingNewOwner,
                MplCoreError::MissingSystemProgram => MplCoreError::MissingSystemProgram,
                MplCoreError::NotAvailable => MplCoreError::NotAvailable,
                MplCoreError::InvalidAsset => MplCoreError::InvalidAsset,
                MplCoreError::MissingCollection => MplCoreError::MissingCollection,
                MplCoreError::NoApprovals => MplCoreError::NoApprovals,
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for MplCoreError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    MplCoreError::InvalidSystemProgram => "InvalidSystemProgram",
                    MplCoreError::DeserializationError => "DeserializationError",
                    MplCoreError::SerializationError => "SerializationError",
                    MplCoreError::PluginsNotInitialized => "PluginsNotInitialized",
                    MplCoreError::PluginNotFound => "PluginNotFound",
                    MplCoreError::NumericalOverflow => "NumericalOverflow",
                    MplCoreError::IncorrectAccount => "IncorrectAccount",
                    MplCoreError::IncorrectAssetHash => "IncorrectAssetHash",
                    MplCoreError::InvalidPlugin => "InvalidPlugin",
                    MplCoreError::InvalidAuthority => "InvalidAuthority",
                    MplCoreError::AssetIsFrozen => "AssetIsFrozen",
                    MplCoreError::MissingCompressionProof => "MissingCompressionProof",
                    MplCoreError::CannotMigrateMasterWithSupply => {
                        "CannotMigrateMasterWithSupply"
                    }
                    MplCoreError::CannotMigratePrints => "CannotMigratePrints",
                    MplCoreError::CannotBurnCollection => "CannotBurnCollection",
                    MplCoreError::PluginAlreadyExists => "PluginAlreadyExists",
                    MplCoreError::NumericalOverflowError => "NumericalOverflowError",
                    MplCoreError::AlreadyCompressed => "AlreadyCompressed",
                    MplCoreError::AlreadyDecompressed => "AlreadyDecompressed",
                    MplCoreError::InvalidCollection => "InvalidCollection",
                    MplCoreError::MissingUpdateAuthority => "MissingUpdateAuthority",
                    MplCoreError::MissingNewOwner => "MissingNewOwner",
                    MplCoreError::MissingSystemProgram => "MissingSystemProgram",
                    MplCoreError::NotAvailable => "NotAvailable",
                    MplCoreError::InvalidAsset => "InvalidAsset",
                    MplCoreError::MissingCollection => "MissingCollection",
                    MplCoreError::NoApprovals => "NoApprovals",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for MplCoreError {}
    #[automatically_derived]
    impl ::core::cmp::Eq for MplCoreError {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for MplCoreError {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for MplCoreError {
        #[inline]
        fn eq(&self, other: &MplCoreError) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[allow(non_upper_case_globals, unused_qualifications)]
    const _IMPL_NUM_FromPrimitive_FOR_MplCoreError: () = {
        #[allow(clippy::useless_attribute)]
        #[allow(rust_2018_idioms)]
        extern crate num_traits as _num_traits;
        impl _num_traits::FromPrimitive for MplCoreError {
            #[allow(trivial_numeric_casts)]
            #[inline]
            fn from_i64(n: i64) -> Option<Self> {
                if n == MplCoreError::InvalidSystemProgram as i64 {
                    Some(MplCoreError::InvalidSystemProgram)
                } else if n == MplCoreError::DeserializationError as i64 {
                    Some(MplCoreError::DeserializationError)
                } else if n == MplCoreError::SerializationError as i64 {
                    Some(MplCoreError::SerializationError)
                } else if n == MplCoreError::PluginsNotInitialized as i64 {
                    Some(MplCoreError::PluginsNotInitialized)
                } else if n == MplCoreError::PluginNotFound as i64 {
                    Some(MplCoreError::PluginNotFound)
                } else if n == MplCoreError::NumericalOverflow as i64 {
                    Some(MplCoreError::NumericalOverflow)
                } else if n == MplCoreError::IncorrectAccount as i64 {
                    Some(MplCoreError::IncorrectAccount)
                } else if n == MplCoreError::IncorrectAssetHash as i64 {
                    Some(MplCoreError::IncorrectAssetHash)
                } else if n == MplCoreError::InvalidPlugin as i64 {
                    Some(MplCoreError::InvalidPlugin)
                } else if n == MplCoreError::InvalidAuthority as i64 {
                    Some(MplCoreError::InvalidAuthority)
                } else if n == MplCoreError::AssetIsFrozen as i64 {
                    Some(MplCoreError::AssetIsFrozen)
                } else if n == MplCoreError::MissingCompressionProof as i64 {
                    Some(MplCoreError::MissingCompressionProof)
                } else if n == MplCoreError::CannotMigrateMasterWithSupply as i64 {
                    Some(MplCoreError::CannotMigrateMasterWithSupply)
                } else if n == MplCoreError::CannotMigratePrints as i64 {
                    Some(MplCoreError::CannotMigratePrints)
                } else if n == MplCoreError::CannotBurnCollection as i64 {
                    Some(MplCoreError::CannotBurnCollection)
                } else if n == MplCoreError::PluginAlreadyExists as i64 {
                    Some(MplCoreError::PluginAlreadyExists)
                } else if n == MplCoreError::NumericalOverflowError as i64 {
                    Some(MplCoreError::NumericalOverflowError)
                } else if n == MplCoreError::AlreadyCompressed as i64 {
                    Some(MplCoreError::AlreadyCompressed)
                } else if n == MplCoreError::AlreadyDecompressed as i64 {
                    Some(MplCoreError::AlreadyDecompressed)
                } else if n == MplCoreError::InvalidCollection as i64 {
                    Some(MplCoreError::InvalidCollection)
                } else if n == MplCoreError::MissingUpdateAuthority as i64 {
                    Some(MplCoreError::MissingUpdateAuthority)
                } else if n == MplCoreError::MissingNewOwner as i64 {
                    Some(MplCoreError::MissingNewOwner)
                } else if n == MplCoreError::MissingSystemProgram as i64 {
                    Some(MplCoreError::MissingSystemProgram)
                } else if n == MplCoreError::NotAvailable as i64 {
                    Some(MplCoreError::NotAvailable)
                } else if n == MplCoreError::InvalidAsset as i64 {
                    Some(MplCoreError::InvalidAsset)
                } else if n == MplCoreError::MissingCollection as i64 {
                    Some(MplCoreError::MissingCollection)
                } else if n == MplCoreError::NoApprovals as i64 {
                    Some(MplCoreError::NoApprovals)
                } else {
                    None
                }
            }
            #[inline]
            fn from_u64(n: u64) -> Option<Self> {
                Self::from_i64(n as i64)
            }
        }
    };
    impl PrintProgramError for MplCoreError {
        fn print<E>(&self) {
            ::solana_program::log::sol_log(&self.to_string());
        }
    }
    impl From<MplCoreError> for ProgramError {
        fn from(e: MplCoreError) -> Self {
            ProgramError::Custom(e as u32)
        }
    }
    impl<T> DecodeError<T> for MplCoreError {
        fn type_of() -> &'static str {
            "Mpl Core Error"
        }
    }
}
/// Main enum for managing instructions on MPL Core.
pub mod instruction {
    #![allow(missing_docs)]
    use borsh::{BorshDeserialize, BorshSerialize};
    use shank::{ShankContext, ShankInstruction};
    use crate::processor::{
        AddCollectionPluginArgs, AddPluginArgs, ApproveCollectionPluginAuthorityArgs,
        ApprovePluginAuthorityArgs, BurnArgs, BurnCollectionArgs, CompressArgs,
        CreateArgs, CreateCollectionArgs, DecompressArgs, RemoveCollectionPluginArgs,
        RemovePluginArgs, RevokeCollectionPluginAuthorityArgs, RevokePluginAuthorityArgs,
        TransferArgs, UpdateArgs, UpdateCollectionArgs, UpdateCollectionPluginArgs,
        UpdatePluginArgs,
    };
    /// Instructions supported by the mpl-core program.
    #[rustfmt::skip]
    pub(crate) enum MplAssetInstruction {
        /// Create a new mpl-core Asset.
        /// This function creates the initial Asset, with or without plugins.
        #[account(
            0,
            writable,
            signer,
            name = "asset",
            desc = "The address of the new asset"
        )]
        #[account(
            1,
            optional,
            writable,
            name = "collection",
            desc = "The collection to which the asset belongs"
        )]
        #[account(
            2,
            optional,
            signer,
            name = "authority",
            desc = "The authority signing for creation"
        )]
        #[account(
            3,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            4,
            optional,
            name = "owner",
            desc = "The owner of the new asset. Defaults to the authority if not present."
        )]
        #[account(
            5,
            optional,
            name = "update_authority",
            desc = "The authority on the new asset"
        )]
        #[account(6, name = "system_program", desc = "The system program")]
        #[account(7, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        Create(CreateArgs),
        /// Create a new mpl-core Collection.
        /// This function creates the initial Collection, with or without plugins.
        #[account(
            0,
            writable,
            signer,
            name = "collection",
            desc = "The address of the new asset"
        )]
        #[account(
            1,
            optional,
            name = "update_authority",
            desc = "The authority of the new asset"
        )]
        #[account(
            2,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(3, name = "system_program", desc = "The system program")]
        CreateCollection(CreateCollectionArgs),
        /// Add a plugin to an mpl-core.
        #[account(0, writable, name = "asset", desc = "The address of the asset")]
        #[account(
            1,
            optional,
            writable,
            name = "collection",
            desc = "The collection to which the asset belongs"
        )]
        #[account(
            2,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            3,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(4, name = "system_program", desc = "The system program")]
        #[account(5, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        AddPlugin(AddPluginArgs),
        /// Add a plugin to an mpl-core Collection.
        #[account(0, writable, name = "collection", desc = "The address of the asset")]
        #[account(
            1,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            2,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(3, name = "system_program", desc = "The system program")]
        #[account(4, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        AddCollectionPlugin(AddCollectionPluginArgs),
        /// Remove a plugin from an mpl-core.
        #[account(0, writable, name = "asset", desc = "The address of the asset")]
        #[account(
            1,
            optional,
            writable,
            name = "collection",
            desc = "The collection to which the asset belongs"
        )]
        #[account(
            2,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            3,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(4, name = "system_program", desc = "The system program")]
        #[account(5, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        RemovePlugin(RemovePluginArgs),
        /// Remove a plugin from an mpl-core Collection.
        #[account(0, writable, name = "collection", desc = "The address of the asset")]
        #[account(
            1,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            2,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(3, name = "system_program", desc = "The system program")]
        #[account(4, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        RemoveCollectionPlugin(RemoveCollectionPluginArgs),
        /// Update a plugin of an mpl-core.
        #[account(0, writable, name = "asset", desc = "The address of the asset")]
        #[account(
            1,
            optional,
            writable,
            name = "collection",
            desc = "The collection to which the asset belongs"
        )]
        #[account(
            2,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            3,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(4, name = "system_program", desc = "The system program")]
        #[account(5, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        UpdatePlugin(UpdatePluginArgs),
        /// Update a plugin of an mpl-core Collection.
        #[account(0, writable, name = "collection", desc = "The address of the asset")]
        #[account(
            1,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            2,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(3, name = "system_program", desc = "The system program")]
        #[account(4, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        UpdateCollectionPlugin(UpdateCollectionPluginArgs),
        /// Approve an authority to an mpl-core plugin.
        #[account(0, writable, name = "asset", desc = "The address of the asset")]
        #[account(
            1,
            optional,
            writable,
            name = "collection",
            desc = "The collection to which the asset belongs"
        )]
        #[account(
            2,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            3,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(4, name = "system_program", desc = "The system program")]
        #[account(5, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        ApprovePluginAuthority(ApprovePluginAuthorityArgs),
        /// Approve an authority to an mpl-core plugin.
        #[account(0, writable, name = "collection", desc = "The address of the asset")]
        #[account(
            1,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            2,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(3, name = "system_program", desc = "The system program")]
        #[account(4, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        ApproveCollectionPluginAuthority(ApproveCollectionPluginAuthorityArgs),
        /// Revoke an authority from an mpl-core plugin.
        #[account(0, writable, name = "asset", desc = "The address of the asset")]
        #[account(
            1,
            optional,
            writable,
            name = "collection",
            desc = "The collection to which the asset belongs"
        )]
        #[account(
            2,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            3,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(4, name = "system_program", desc = "The system program")]
        #[account(5, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        RevokePluginAuthority(RevokePluginAuthorityArgs),
        /// Revoke an authority from an mpl-core plugin.
        #[account(0, writable, name = "collection", desc = "The address of the asset")]
        #[account(
            1,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            2,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(3, name = "system_program", desc = "The system program")]
        #[account(4, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        RevokeCollectionPluginAuthority(RevokeCollectionPluginAuthorityArgs),
        /// Burn an mpl-core.
        #[account(0, writable, name = "asset", desc = "The address of the asset")]
        #[account(
            1,
            optional,
            writable,
            name = "collection",
            desc = "The collection to which the asset belongs"
        )]
        #[account(
            2,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            3,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(4, optional, name = "system_program", desc = "The system program")]
        #[account(5, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        Burn(BurnArgs),
        /// Burn an mpl-core.
        #[account(0, writable, name = "collection", desc = "The address of the asset")]
        #[account(
            1,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            2,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(3, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        BurnCollection(BurnCollectionArgs),
        /// Transfer an asset by changing its owner.
        #[account(0, writable, name = "asset", desc = "The address of the asset")]
        #[account(
            1,
            optional,
            name = "collection",
            desc = "The collection to which the asset belongs"
        )]
        #[account(
            2,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            3,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(
            4,
            name = "new_owner",
            desc = "The new owner to which to transfer the asset"
        )]
        #[account(5, optional, name = "system_program", desc = "The system program")]
        #[account(6, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        Transfer(TransferArgs),
        /// Update an mpl-core.
        #[account(0, writable, name = "asset", desc = "The address of the asset")]
        #[account(
            1,
            optional,
            name = "collection",
            desc = "The collection to which the asset belongs"
        )]
        #[account(
            2,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            3,
            optional,
            signer,
            name = "authority",
            desc = "The update authority or update authority delegate of the asset"
        )]
        #[account(
            4,
            optional,
            name = "new_update_authority",
            desc = "The new update authority of the asset"
        )]
        #[account(5, name = "system_program", desc = "The system program")]
        #[account(6, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        Update(UpdateArgs),
        /// Update an mpl-core.
        #[account(0, writable, name = "collection", desc = "The address of the asset")]
        #[account(
            1,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            2,
            optional,
            signer,
            name = "authority",
            desc = "The update authority or update authority delegate of the asset"
        )]
        #[account(
            3,
            optional,
            name = "new_update_authority",
            desc = "The new update authority of the asset"
        )]
        #[account(4, name = "system_program", desc = "The system program")]
        #[account(5, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        UpdateCollection(UpdateCollectionArgs),
        /// Compress an mpl-core.
        #[account(0, writable, name = "asset", desc = "The address of the asset")]
        #[account(
            1,
            optional,
            name = "collection",
            desc = "The collection to which the asset belongs"
        )]
        #[account(
            2,
            writable,
            signer,
            name = "payer",
            desc = "The account receiving the storage fees"
        )]
        #[account(
            3,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(4, name = "system_program", desc = "The system program")]
        #[account(5, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        Compress(CompressArgs),
        /// Decompress an mpl-core.
        #[account(0, writable, name = "asset", desc = "The address of the asset")]
        #[account(
            1,
            optional,
            name = "collection",
            desc = "The collection to which the asset belongs"
        )]
        #[account(
            2,
            writable,
            signer,
            name = "payer",
            desc = "The account paying for the storage fees"
        )]
        #[account(
            3,
            optional,
            signer,
            name = "authority",
            desc = "The owner or delegate of the asset"
        )]
        #[account(4, name = "system_program", desc = "The system program")]
        #[account(5, optional, name = "log_wrapper", desc = "The SPL Noop Program")]
        Decompress(DecompressArgs),
        /// Collect
        /// This function creates the initial mpl-core
        #[account(
            0,
            writable,
            name = "recipient",
            desc = "The address of the recipient"
        )]
        Collect,
    }
    impl borsh::de::BorshDeserialize for MplAssetInstruction
    where
        CreateArgs: borsh::BorshDeserialize,
        CreateCollectionArgs: borsh::BorshDeserialize,
        AddPluginArgs: borsh::BorshDeserialize,
        AddCollectionPluginArgs: borsh::BorshDeserialize,
        RemovePluginArgs: borsh::BorshDeserialize,
        RemoveCollectionPluginArgs: borsh::BorshDeserialize,
        UpdatePluginArgs: borsh::BorshDeserialize,
        UpdateCollectionPluginArgs: borsh::BorshDeserialize,
        ApprovePluginAuthorityArgs: borsh::BorshDeserialize,
        ApproveCollectionPluginAuthorityArgs: borsh::BorshDeserialize,
        RevokePluginAuthorityArgs: borsh::BorshDeserialize,
        RevokeCollectionPluginAuthorityArgs: borsh::BorshDeserialize,
        BurnArgs: borsh::BorshDeserialize,
        BurnCollectionArgs: borsh::BorshDeserialize,
        TransferArgs: borsh::BorshDeserialize,
        UpdateArgs: borsh::BorshDeserialize,
        UpdateCollectionArgs: borsh::BorshDeserialize,
        CompressArgs: borsh::BorshDeserialize,
        DecompressArgs: borsh::BorshDeserialize,
    {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let tag = <u8 as borsh::de::BorshDeserialize>::deserialize_reader(reader)?;
            <Self as borsh::de::EnumExt>::deserialize_variant(reader, tag)
        }
    }
    impl borsh::de::EnumExt for MplAssetInstruction
    where
        CreateArgs: borsh::BorshDeserialize,
        CreateCollectionArgs: borsh::BorshDeserialize,
        AddPluginArgs: borsh::BorshDeserialize,
        AddCollectionPluginArgs: borsh::BorshDeserialize,
        RemovePluginArgs: borsh::BorshDeserialize,
        RemoveCollectionPluginArgs: borsh::BorshDeserialize,
        UpdatePluginArgs: borsh::BorshDeserialize,
        UpdateCollectionPluginArgs: borsh::BorshDeserialize,
        ApprovePluginAuthorityArgs: borsh::BorshDeserialize,
        ApproveCollectionPluginAuthorityArgs: borsh::BorshDeserialize,
        RevokePluginAuthorityArgs: borsh::BorshDeserialize,
        RevokeCollectionPluginAuthorityArgs: borsh::BorshDeserialize,
        BurnArgs: borsh::BorshDeserialize,
        BurnCollectionArgs: borsh::BorshDeserialize,
        TransferArgs: borsh::BorshDeserialize,
        UpdateArgs: borsh::BorshDeserialize,
        UpdateCollectionArgs: borsh::BorshDeserialize,
        CompressArgs: borsh::BorshDeserialize,
        DecompressArgs: borsh::BorshDeserialize,
    {
        fn deserialize_variant<R: borsh::maybestd::io::Read>(
            reader: &mut R,
            variant_idx: u8,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let mut return_value = match variant_idx {
                0u8 => {
                    MplAssetInstruction::Create(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                1u8 => {
                    MplAssetInstruction::CreateCollection(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                2u8 => {
                    MplAssetInstruction::AddPlugin(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                3u8 => {
                    MplAssetInstruction::AddCollectionPlugin(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                4u8 => {
                    MplAssetInstruction::RemovePlugin(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                5u8 => {
                    MplAssetInstruction::RemoveCollectionPlugin(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                6u8 => {
                    MplAssetInstruction::UpdatePlugin(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                7u8 => {
                    MplAssetInstruction::UpdateCollectionPlugin(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                8u8 => {
                    MplAssetInstruction::ApprovePluginAuthority(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                9u8 => {
                    MplAssetInstruction::ApproveCollectionPluginAuthority(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                10u8 => {
                    MplAssetInstruction::RevokePluginAuthority(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                11u8 => {
                    MplAssetInstruction::RevokeCollectionPluginAuthority(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                12u8 => {
                    MplAssetInstruction::Burn(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                13u8 => {
                    MplAssetInstruction::BurnCollection(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                14u8 => {
                    MplAssetInstruction::Transfer(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                15u8 => {
                    MplAssetInstruction::Update(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                16u8 => {
                    MplAssetInstruction::UpdateCollection(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                17u8 => {
                    MplAssetInstruction::Compress(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                18u8 => {
                    MplAssetInstruction::Decompress(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                19u8 => MplAssetInstruction::Collect,
                _ => {
                    return Err(
                        borsh::maybestd::io::Error::new(
                            borsh::maybestd::io::ErrorKind::InvalidInput,
                            {
                                let res = ::alloc::fmt::format(
                                    format_args!("Unexpected variant index: {0:?}", variant_idx),
                                );
                                res
                            },
                        ),
                    );
                }
            };
            Ok(return_value)
        }
    }
    impl borsh::ser::BorshSerialize for MplAssetInstruction
    where
        CreateArgs: borsh::ser::BorshSerialize,
        CreateCollectionArgs: borsh::ser::BorshSerialize,
        AddPluginArgs: borsh::ser::BorshSerialize,
        AddCollectionPluginArgs: borsh::ser::BorshSerialize,
        RemovePluginArgs: borsh::ser::BorshSerialize,
        RemoveCollectionPluginArgs: borsh::ser::BorshSerialize,
        UpdatePluginArgs: borsh::ser::BorshSerialize,
        UpdateCollectionPluginArgs: borsh::ser::BorshSerialize,
        ApprovePluginAuthorityArgs: borsh::ser::BorshSerialize,
        ApproveCollectionPluginAuthorityArgs: borsh::ser::BorshSerialize,
        RevokePluginAuthorityArgs: borsh::ser::BorshSerialize,
        RevokeCollectionPluginAuthorityArgs: borsh::ser::BorshSerialize,
        BurnArgs: borsh::ser::BorshSerialize,
        BurnCollectionArgs: borsh::ser::BorshSerialize,
        TransferArgs: borsh::ser::BorshSerialize,
        UpdateArgs: borsh::ser::BorshSerialize,
        UpdateCollectionArgs: borsh::ser::BorshSerialize,
        CompressArgs: borsh::ser::BorshSerialize,
        DecompressArgs: borsh::ser::BorshSerialize,
    {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            let variant_idx: u8 = match self {
                MplAssetInstruction::Create(..) => 0u8,
                MplAssetInstruction::CreateCollection(..) => 1u8,
                MplAssetInstruction::AddPlugin(..) => 2u8,
                MplAssetInstruction::AddCollectionPlugin(..) => 3u8,
                MplAssetInstruction::RemovePlugin(..) => 4u8,
                MplAssetInstruction::RemoveCollectionPlugin(..) => 5u8,
                MplAssetInstruction::UpdatePlugin(..) => 6u8,
                MplAssetInstruction::UpdateCollectionPlugin(..) => 7u8,
                MplAssetInstruction::ApprovePluginAuthority(..) => 8u8,
                MplAssetInstruction::ApproveCollectionPluginAuthority(..) => 9u8,
                MplAssetInstruction::RevokePluginAuthority(..) => 10u8,
                MplAssetInstruction::RevokeCollectionPluginAuthority(..) => 11u8,
                MplAssetInstruction::Burn(..) => 12u8,
                MplAssetInstruction::BurnCollection(..) => 13u8,
                MplAssetInstruction::Transfer(..) => 14u8,
                MplAssetInstruction::Update(..) => 15u8,
                MplAssetInstruction::UpdateCollection(..) => 16u8,
                MplAssetInstruction::Compress(..) => 17u8,
                MplAssetInstruction::Decompress(..) => 18u8,
                MplAssetInstruction::Collect => 19u8,
            };
            writer.write_all(&variant_idx.to_le_bytes())?;
            match self {
                MplAssetInstruction::Create(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::CreateCollection(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::AddPlugin(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::AddCollectionPlugin(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::RemovePlugin(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::RemoveCollectionPlugin(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::UpdatePlugin(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::UpdateCollectionPlugin(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::ApprovePluginAuthority(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::ApproveCollectionPluginAuthority(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::RevokePluginAuthority(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::RevokeCollectionPluginAuthority(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::Burn(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::BurnCollection(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::Transfer(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::Update(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::UpdateCollection(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::Compress(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::Decompress(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                MplAssetInstruction::Collect => {}
            }
            Ok(())
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for MplAssetInstruction {
        #[inline]
        fn clone(&self) -> MplAssetInstruction {
            match self {
                MplAssetInstruction::Create(__self_0) => {
                    MplAssetInstruction::Create(::core::clone::Clone::clone(__self_0))
                }
                MplAssetInstruction::CreateCollection(__self_0) => {
                    MplAssetInstruction::CreateCollection(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::AddPlugin(__self_0) => {
                    MplAssetInstruction::AddPlugin(::core::clone::Clone::clone(__self_0))
                }
                MplAssetInstruction::AddCollectionPlugin(__self_0) => {
                    MplAssetInstruction::AddCollectionPlugin(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::RemovePlugin(__self_0) => {
                    MplAssetInstruction::RemovePlugin(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::RemoveCollectionPlugin(__self_0) => {
                    MplAssetInstruction::RemoveCollectionPlugin(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::UpdatePlugin(__self_0) => {
                    MplAssetInstruction::UpdatePlugin(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::UpdateCollectionPlugin(__self_0) => {
                    MplAssetInstruction::UpdateCollectionPlugin(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::ApprovePluginAuthority(__self_0) => {
                    MplAssetInstruction::ApprovePluginAuthority(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::ApproveCollectionPluginAuthority(__self_0) => {
                    MplAssetInstruction::ApproveCollectionPluginAuthority(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::RevokePluginAuthority(__self_0) => {
                    MplAssetInstruction::RevokePluginAuthority(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::RevokeCollectionPluginAuthority(__self_0) => {
                    MplAssetInstruction::RevokeCollectionPluginAuthority(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::Burn(__self_0) => {
                    MplAssetInstruction::Burn(::core::clone::Clone::clone(__self_0))
                }
                MplAssetInstruction::BurnCollection(__self_0) => {
                    MplAssetInstruction::BurnCollection(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::Transfer(__self_0) => {
                    MplAssetInstruction::Transfer(::core::clone::Clone::clone(__self_0))
                }
                MplAssetInstruction::Update(__self_0) => {
                    MplAssetInstruction::Update(::core::clone::Clone::clone(__self_0))
                }
                MplAssetInstruction::UpdateCollection(__self_0) => {
                    MplAssetInstruction::UpdateCollection(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::Compress(__self_0) => {
                    MplAssetInstruction::Compress(::core::clone::Clone::clone(__self_0))
                }
                MplAssetInstruction::Decompress(__self_0) => {
                    MplAssetInstruction::Decompress(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                MplAssetInstruction::Collect => MplAssetInstruction::Collect,
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for MplAssetInstruction {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                MplAssetInstruction::Create(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Create",
                        &__self_0,
                    )
                }
                MplAssetInstruction::CreateCollection(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "CreateCollection",
                        &__self_0,
                    )
                }
                MplAssetInstruction::AddPlugin(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "AddPlugin",
                        &__self_0,
                    )
                }
                MplAssetInstruction::AddCollectionPlugin(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "AddCollectionPlugin",
                        &__self_0,
                    )
                }
                MplAssetInstruction::RemovePlugin(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "RemovePlugin",
                        &__self_0,
                    )
                }
                MplAssetInstruction::RemoveCollectionPlugin(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "RemoveCollectionPlugin",
                        &__self_0,
                    )
                }
                MplAssetInstruction::UpdatePlugin(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "UpdatePlugin",
                        &__self_0,
                    )
                }
                MplAssetInstruction::UpdateCollectionPlugin(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "UpdateCollectionPlugin",
                        &__self_0,
                    )
                }
                MplAssetInstruction::ApprovePluginAuthority(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ApprovePluginAuthority",
                        &__self_0,
                    )
                }
                MplAssetInstruction::ApproveCollectionPluginAuthority(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ApproveCollectionPluginAuthority",
                        &__self_0,
                    )
                }
                MplAssetInstruction::RevokePluginAuthority(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "RevokePluginAuthority",
                        &__self_0,
                    )
                }
                MplAssetInstruction::RevokeCollectionPluginAuthority(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "RevokeCollectionPluginAuthority",
                        &__self_0,
                    )
                }
                MplAssetInstruction::Burn(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Burn",
                        &__self_0,
                    )
                }
                MplAssetInstruction::BurnCollection(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "BurnCollection",
                        &__self_0,
                    )
                }
                MplAssetInstruction::Transfer(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Transfer",
                        &__self_0,
                    )
                }
                MplAssetInstruction::Update(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Update",
                        &__self_0,
                    )
                }
                MplAssetInstruction::UpdateCollection(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "UpdateCollection",
                        &__self_0,
                    )
                }
                MplAssetInstruction::Compress(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Compress",
                        &__self_0,
                    )
                }
                MplAssetInstruction::Decompress(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Decompress",
                        &__self_0,
                    )
                }
                MplAssetInstruction::Collect => {
                    ::core::fmt::Formatter::write_str(f, "Collect")
                }
            }
        }
    }
    pub mod accounts {
        use super::*;
        pub struct Context<'a, T> {
            pub accounts: T,
            pub remaining_accounts: &'a [solana_program::account_info::AccountInfo<'a>],
        }
        pub struct CreateAccounts<'a> {
            pub asset: &'a solana_program::account_info::AccountInfo<'a>,
            pub collection: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub owner: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub update_authority: Option<
                &'a solana_program::account_info::AccountInfo<'a>,
            >,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> CreateAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 8usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        asset: &accounts[0usize],
                        collection: if accounts[1usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[1usize])
                        },
                        authority: if accounts[2usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[2usize])
                        },
                        payer: &accounts[3usize],
                        owner: if accounts[4usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[4usize])
                        },
                        update_authority: if accounts[5usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[5usize])
                        },
                        system_program: &accounts[6usize],
                        log_wrapper: if accounts[7usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[7usize])
                        },
                    },
                    remaining_accounts: &accounts[8usize..],
                })
            }
        }
        pub struct CreateCollectionAccounts<'a> {
            pub collection: &'a solana_program::account_info::AccountInfo<'a>,
            pub update_authority: Option<
                &'a solana_program::account_info::AccountInfo<'a>,
            >,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
        }
        impl<'a> CreateCollectionAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 4usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        collection: &accounts[0usize],
                        update_authority: if accounts[1usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[1usize])
                        },
                        payer: &accounts[2usize],
                        system_program: &accounts[3usize],
                    },
                    remaining_accounts: &accounts[4usize..],
                })
            }
        }
        pub struct AddPluginAccounts<'a> {
            pub asset: &'a solana_program::account_info::AccountInfo<'a>,
            pub collection: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> AddPluginAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 6usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        asset: &accounts[0usize],
                        collection: if accounts[1usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[1usize])
                        },
                        payer: &accounts[2usize],
                        authority: if accounts[3usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[3usize])
                        },
                        system_program: &accounts[4usize],
                        log_wrapper: if accounts[5usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[5usize])
                        },
                    },
                    remaining_accounts: &accounts[6usize..],
                })
            }
        }
        pub struct AddCollectionPluginAccounts<'a> {
            pub collection: &'a solana_program::account_info::AccountInfo<'a>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> AddCollectionPluginAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 5usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        collection: &accounts[0usize],
                        payer: &accounts[1usize],
                        authority: if accounts[2usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[2usize])
                        },
                        system_program: &accounts[3usize],
                        log_wrapper: if accounts[4usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[4usize])
                        },
                    },
                    remaining_accounts: &accounts[5usize..],
                })
            }
        }
        pub struct RemovePluginAccounts<'a> {
            pub asset: &'a solana_program::account_info::AccountInfo<'a>,
            pub collection: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> RemovePluginAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 6usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        asset: &accounts[0usize],
                        collection: if accounts[1usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[1usize])
                        },
                        payer: &accounts[2usize],
                        authority: if accounts[3usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[3usize])
                        },
                        system_program: &accounts[4usize],
                        log_wrapper: if accounts[5usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[5usize])
                        },
                    },
                    remaining_accounts: &accounts[6usize..],
                })
            }
        }
        pub struct RemoveCollectionPluginAccounts<'a> {
            pub collection: &'a solana_program::account_info::AccountInfo<'a>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> RemoveCollectionPluginAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 5usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        collection: &accounts[0usize],
                        payer: &accounts[1usize],
                        authority: if accounts[2usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[2usize])
                        },
                        system_program: &accounts[3usize],
                        log_wrapper: if accounts[4usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[4usize])
                        },
                    },
                    remaining_accounts: &accounts[5usize..],
                })
            }
        }
        pub struct UpdatePluginAccounts<'a> {
            pub asset: &'a solana_program::account_info::AccountInfo<'a>,
            pub collection: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> UpdatePluginAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 6usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        asset: &accounts[0usize],
                        collection: if accounts[1usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[1usize])
                        },
                        payer: &accounts[2usize],
                        authority: if accounts[3usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[3usize])
                        },
                        system_program: &accounts[4usize],
                        log_wrapper: if accounts[5usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[5usize])
                        },
                    },
                    remaining_accounts: &accounts[6usize..],
                })
            }
        }
        pub struct UpdateCollectionPluginAccounts<'a> {
            pub collection: &'a solana_program::account_info::AccountInfo<'a>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> UpdateCollectionPluginAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 5usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        collection: &accounts[0usize],
                        payer: &accounts[1usize],
                        authority: if accounts[2usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[2usize])
                        },
                        system_program: &accounts[3usize],
                        log_wrapper: if accounts[4usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[4usize])
                        },
                    },
                    remaining_accounts: &accounts[5usize..],
                })
            }
        }
        pub struct ApprovePluginAuthorityAccounts<'a> {
            pub asset: &'a solana_program::account_info::AccountInfo<'a>,
            pub collection: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> ApprovePluginAuthorityAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 6usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        asset: &accounts[0usize],
                        collection: if accounts[1usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[1usize])
                        },
                        payer: &accounts[2usize],
                        authority: if accounts[3usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[3usize])
                        },
                        system_program: &accounts[4usize],
                        log_wrapper: if accounts[5usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[5usize])
                        },
                    },
                    remaining_accounts: &accounts[6usize..],
                })
            }
        }
        pub struct ApproveCollectionPluginAuthorityAccounts<'a> {
            pub collection: &'a solana_program::account_info::AccountInfo<'a>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> ApproveCollectionPluginAuthorityAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 5usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        collection: &accounts[0usize],
                        payer: &accounts[1usize],
                        authority: if accounts[2usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[2usize])
                        },
                        system_program: &accounts[3usize],
                        log_wrapper: if accounts[4usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[4usize])
                        },
                    },
                    remaining_accounts: &accounts[5usize..],
                })
            }
        }
        pub struct RevokePluginAuthorityAccounts<'a> {
            pub asset: &'a solana_program::account_info::AccountInfo<'a>,
            pub collection: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> RevokePluginAuthorityAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 6usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        asset: &accounts[0usize],
                        collection: if accounts[1usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[1usize])
                        },
                        payer: &accounts[2usize],
                        authority: if accounts[3usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[3usize])
                        },
                        system_program: &accounts[4usize],
                        log_wrapper: if accounts[5usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[5usize])
                        },
                    },
                    remaining_accounts: &accounts[6usize..],
                })
            }
        }
        pub struct RevokeCollectionPluginAuthorityAccounts<'a> {
            pub collection: &'a solana_program::account_info::AccountInfo<'a>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> RevokeCollectionPluginAuthorityAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 5usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        collection: &accounts[0usize],
                        payer: &accounts[1usize],
                        authority: if accounts[2usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[2usize])
                        },
                        system_program: &accounts[3usize],
                        log_wrapper: if accounts[4usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[4usize])
                        },
                    },
                    remaining_accounts: &accounts[5usize..],
                })
            }
        }
        pub struct BurnAccounts<'a> {
            pub asset: &'a solana_program::account_info::AccountInfo<'a>,
            pub collection: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: Option<
                &'a solana_program::account_info::AccountInfo<'a>,
            >,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> BurnAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 6usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        asset: &accounts[0usize],
                        collection: if accounts[1usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[1usize])
                        },
                        payer: &accounts[2usize],
                        authority: if accounts[3usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[3usize])
                        },
                        system_program: if accounts[4usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[4usize])
                        },
                        log_wrapper: if accounts[5usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[5usize])
                        },
                    },
                    remaining_accounts: &accounts[6usize..],
                })
            }
        }
        pub struct BurnCollectionAccounts<'a> {
            pub collection: &'a solana_program::account_info::AccountInfo<'a>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> BurnCollectionAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 4usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        collection: &accounts[0usize],
                        payer: &accounts[1usize],
                        authority: if accounts[2usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[2usize])
                        },
                        log_wrapper: if accounts[3usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[3usize])
                        },
                    },
                    remaining_accounts: &accounts[4usize..],
                })
            }
        }
        pub struct TransferAccounts<'a> {
            pub asset: &'a solana_program::account_info::AccountInfo<'a>,
            pub collection: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub new_owner: &'a solana_program::account_info::AccountInfo<'a>,
            pub system_program: Option<
                &'a solana_program::account_info::AccountInfo<'a>,
            >,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> TransferAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 7usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        asset: &accounts[0usize],
                        collection: if accounts[1usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[1usize])
                        },
                        payer: &accounts[2usize],
                        authority: if accounts[3usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[3usize])
                        },
                        new_owner: &accounts[4usize],
                        system_program: if accounts[5usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[5usize])
                        },
                        log_wrapper: if accounts[6usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[6usize])
                        },
                    },
                    remaining_accounts: &accounts[7usize..],
                })
            }
        }
        pub struct UpdateAccounts<'a> {
            pub asset: &'a solana_program::account_info::AccountInfo<'a>,
            pub collection: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub new_update_authority: Option<
                &'a solana_program::account_info::AccountInfo<'a>,
            >,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> UpdateAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 7usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        asset: &accounts[0usize],
                        collection: if accounts[1usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[1usize])
                        },
                        payer: &accounts[2usize],
                        authority: if accounts[3usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[3usize])
                        },
                        new_update_authority: if accounts[4usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[4usize])
                        },
                        system_program: &accounts[5usize],
                        log_wrapper: if accounts[6usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[6usize])
                        },
                    },
                    remaining_accounts: &accounts[7usize..],
                })
            }
        }
        pub struct UpdateCollectionAccounts<'a> {
            pub collection: &'a solana_program::account_info::AccountInfo<'a>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub new_update_authority: Option<
                &'a solana_program::account_info::AccountInfo<'a>,
            >,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> UpdateCollectionAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 6usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        collection: &accounts[0usize],
                        payer: &accounts[1usize],
                        authority: if accounts[2usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[2usize])
                        },
                        new_update_authority: if accounts[3usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[3usize])
                        },
                        system_program: &accounts[4usize],
                        log_wrapper: if accounts[5usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[5usize])
                        },
                    },
                    remaining_accounts: &accounts[6usize..],
                })
            }
        }
        pub struct CompressAccounts<'a> {
            pub asset: &'a solana_program::account_info::AccountInfo<'a>,
            pub collection: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> CompressAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 6usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        asset: &accounts[0usize],
                        collection: if accounts[1usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[1usize])
                        },
                        payer: &accounts[2usize],
                        authority: if accounts[3usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[3usize])
                        },
                        system_program: &accounts[4usize],
                        log_wrapper: if accounts[5usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[5usize])
                        },
                    },
                    remaining_accounts: &accounts[6usize..],
                })
            }
        }
        pub struct DecompressAccounts<'a> {
            pub asset: &'a solana_program::account_info::AccountInfo<'a>,
            pub collection: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub payer: &'a solana_program::account_info::AccountInfo<'a>,
            pub authority: Option<&'a solana_program::account_info::AccountInfo<'a>>,
            pub system_program: &'a solana_program::account_info::AccountInfo<'a>,
            pub log_wrapper: Option<&'a solana_program::account_info::AccountInfo<'a>>,
        }
        impl<'a> DecompressAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 6usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        asset: &accounts[0usize],
                        collection: if accounts[1usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[1usize])
                        },
                        payer: &accounts[2usize],
                        authority: if accounts[3usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[3usize])
                        },
                        system_program: &accounts[4usize],
                        log_wrapper: if accounts[5usize].key == &crate::ID {
                            None
                        } else {
                            Some(&accounts[5usize])
                        },
                    },
                    remaining_accounts: &accounts[6usize..],
                })
            }
        }
        pub struct CollectAccounts<'a> {
            pub recipient: &'a solana_program::account_info::AccountInfo<'a>,
        }
        impl<'a> CollectAccounts<'a> {
            pub fn context(
                accounts: &'a [solana_program::account_info::AccountInfo<'a>],
            ) -> Result<
                Context<'a, Self>,
                solana_program::sysvar::slot_history::ProgramError,
            > {
                if accounts.len() < 1usize {
                    return Err(
                        solana_program::sysvar::slot_history::ProgramError::NotEnoughAccountKeys,
                    );
                }
                Ok(Context {
                    accounts: Self {
                        recipient: &accounts[0usize],
                    },
                    remaining_accounts: &accounts[1usize..],
                })
            }
        }
    }
}
/// Module for managing plugins.
pub mod plugins {
    mod attributes {
        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::{account_info::AccountInfo, program_error::ProgramError};
        use crate::state::{Authority, CoreAsset, DataBlob};
        use super::{PluginValidation, ValidationResult};
        /// The Attribute type which represent a Key Value pair.
        #[repr(C)]
        pub struct Attribute {
            /// The Key of the attribute.
            pub key: String,
            /// The Value of the attribute.
            pub value: String,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Attribute {
            #[inline]
            fn clone(&self) -> Attribute {
                Attribute {
                    key: ::core::clone::Clone::clone(&self.key),
                    value: ::core::clone::Clone::clone(&self.value),
                }
            }
        }
        impl borsh::ser::BorshSerialize for Attribute
        where
            String: borsh::ser::BorshSerialize,
            String: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.key, writer)?;
                borsh::BorshSerialize::serialize(&self.value, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for Attribute
        where
            String: borsh::BorshDeserialize,
            String: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    key: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    value: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Attribute {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "Attribute",
                    "key",
                    &self.key,
                    "value",
                    &&self.value,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Attribute {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Attribute {
            #[inline]
            fn eq(&self, other: &Attribute) -> bool {
                self.key == other.key && self.value == other.value
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for Attribute {}
        #[automatically_derived]
        impl ::core::cmp::Eq for Attribute {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<String>;
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for Attribute {
            #[inline]
            fn default() -> Attribute {
                Attribute {
                    key: ::core::default::Default::default(),
                    value: ::core::default::Default::default(),
                }
            }
        }
        /// The Attributes plugin allows the authority to add arbitrary Key-Value pairs to the asset.
        #[repr(C)]
        pub struct Attributes {
            /// A vector of Key-Value pairs.
            pub attribute_list: Vec<Attribute>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Attributes {
            #[inline]
            fn clone(&self) -> Attributes {
                Attributes {
                    attribute_list: ::core::clone::Clone::clone(&self.attribute_list),
                }
            }
        }
        impl borsh::ser::BorshSerialize for Attributes
        where
            Vec<Attribute>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.attribute_list, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for Attributes
        where
            Vec<Attribute>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    attribute_list: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Attributes {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "Attributes",
                    "attribute_list",
                    &&self.attribute_list,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Attributes {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Attributes {
            #[inline]
            fn eq(&self, other: &Attributes) -> bool {
                self.attribute_list == other.attribute_list
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for Attributes {}
        #[automatically_derived]
        impl ::core::cmp::Eq for Attributes {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Vec<Attribute>>;
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for Attributes {
            #[inline]
            fn default() -> Attributes {
                Attributes {
                    attribute_list: ::core::default::Default::default(),
                }
            }
        }
        impl Attributes {
            /// Initialize the Attributes plugin, unfrozen by default.
            pub fn new() -> Self {
                Self::default()
            }
        }
        impl DataBlob for Attributes {
            fn get_initial_size() -> usize {
                4
            }
            fn get_size(&self) -> usize {
                4
            }
        }
        impl PluginValidation for Attributes {
            fn validate_update_plugin<T: CoreAsset>(
                &self,
                core_asset: &T,
                authority_info: &AccountInfo,
                authority: &Authority,
            ) -> Result<ValidationResult, ProgramError> {
                if authority_info.key == &core_asset.update_authority().key()
                    && authority == (&Authority::UpdateAuthority)
                    || authority
                        == (&Authority::Pubkey {
                            address: *authority_info.key,
                        })
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
        }
    }
    mod burn {
        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::{account_info::AccountInfo, program_error::ProgramError};
        use crate::{plugins::PluginType, state::{Authority, DataBlob}};
        use super::{Plugin, PluginValidation, ValidationResult};
        /// This plugin manages additional permissions to burn.
        /// Any authorities approved are given permission to burn the asset on behalf of the owner.
        #[repr(C)]
        pub struct Burn {}
        #[automatically_derived]
        impl ::core::clone::Clone for Burn {
            #[inline]
            fn clone(&self) -> Burn {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Burn {}
        impl borsh::ser::BorshSerialize for Burn {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for Burn {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {})
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Burn {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "Burn")
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Burn {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Burn {
            #[inline]
            fn eq(&self, other: &Burn) -> bool {
                true
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for Burn {}
        #[automatically_derived]
        impl ::core::cmp::Eq for Burn {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        impl Burn {
            /// Initialize the Burn plugin.
            pub fn new() -> Self {
                Self {}
            }
        }
        impl Default for Burn {
            fn default() -> Self {
                Self::new()
            }
        }
        impl DataBlob for Burn {
            fn get_initial_size() -> usize {
                0
            }
            fn get_size(&self) -> usize {
                0
            }
        }
        impl PluginValidation for Burn {
            fn validate_burn(
                &self,
                authority_info: &AccountInfo,
                authority: &Authority,
                _resolved_authority: Option<&Authority>,
            ) -> Result<super::ValidationResult, ProgramError> {
                if authority
                    == (&Authority::Pubkey {
                        address: *authority_info.key,
                    })
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the revoke plugin authority lifecycle action.
            fn validate_revoke_plugin_authority(
                &self,
                authority_info: &AccountInfo,
                authority: &Authority,
                plugin_to_revoke: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                ::solana_program::log::sol_log(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("authority_info: {0:?}", authority_info.key),
                        );
                        res
                    },
                );
                ::solana_program::log::sol_log(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("authority: {0:?}", authority),
                        );
                        res
                    },
                );
                if authority
                    == &(Authority::Pubkey {
                        address: *authority_info.key,
                    }) && plugin_to_revoke.is_some()
                    && PluginType::from(plugin_to_revoke.unwrap()) == PluginType::Burn
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
        }
    }
    mod freeze {
        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::{account_info::AccountInfo, program_error::ProgramError};
        use crate::state::{Authority, CoreAsset, DataBlob};
        use super::{Plugin, PluginValidation, ValidationResult};
        /// The freeze plugin allows any authority to lock the asset so it's no longer transferable.
        /// The default authority for this plugin is the owner.
        #[repr(C)]
        pub struct Freeze {
            /// The current state of the asset and whether or not it's transferable.
            pub frozen: bool,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Freeze {
            #[inline]
            fn clone(&self) -> Freeze {
                let _: ::core::clone::AssertParamIsClone<bool>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Freeze {}
        impl borsh::ser::BorshSerialize for Freeze
        where
            bool: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.frozen, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for Freeze
        where
            bool: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    frozen: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Freeze {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "Freeze",
                    "frozen",
                    &&self.frozen,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Freeze {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Freeze {
            #[inline]
            fn eq(&self, other: &Freeze) -> bool {
                self.frozen == other.frozen
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for Freeze {}
        #[automatically_derived]
        impl ::core::cmp::Eq for Freeze {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<bool>;
            }
        }
        impl Freeze {
            /// Initialize the Freeze plugin, unfrozen by default.
            pub fn new() -> Self {
                Self { frozen: false }
            }
        }
        impl Default for Freeze {
            fn default() -> Self {
                Self::new()
            }
        }
        impl DataBlob for Freeze {
            fn get_initial_size() -> usize {
                1
            }
            fn get_size(&self) -> usize {
                1
            }
        }
        impl PluginValidation for Freeze {
            fn validate_update_plugin<T: CoreAsset>(
                &self,
                core_asset: &T,
                authority_info: &AccountInfo,
                authority: &Authority,
            ) -> Result<ValidationResult, ProgramError> {
                if (authority_info.key != core_asset.owner()
                    && (authority_info.key == &core_asset.update_authority().key()
                        && authority == (&Authority::UpdateAuthority))
                    || authority
                        == (&Authority::Pubkey {
                            address: *authority_info.key,
                        }))
                    || (authority_info.key == core_asset.owner()
                        && authority == (&Authority::Owner))
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            fn validate_burn(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
                _resolved_authority: Option<&Authority>,
            ) -> Result<super::ValidationResult, ProgramError> {
                if self.frozen {
                    Ok(ValidationResult::Rejected)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            fn validate_transfer(
                &self,
                _authority_info: &AccountInfo,
                _new_owner: &AccountInfo,
                _authority: &Authority,
                _resolved_authority: Option<&Authority>,
            ) -> Result<super::ValidationResult, ProgramError> {
                if self.frozen {
                    Ok(ValidationResult::Rejected)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            fn validate_approve_plugin_authority(
                &self,
                _authority: &AccountInfo,
                _authorities: &Authority,
                plugin_to_approve: Option<&super::Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if let Some(Plugin::Freeze(freeze)) = plugin_to_approve {
                    if freeze.frozen {
                        return Ok(ValidationResult::Rejected);
                    }
                }
                Ok(ValidationResult::Pass)
            }
            /// Validate the revoke plugin authority lifecycle action.
            fn validate_revoke_plugin_authority(
                &self,
                authority_info: &AccountInfo,
                authority: &Authority,
                plugin_to_revoke: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if let Some(Plugin::Freeze(freeze)) = plugin_to_revoke {
                    if freeze.frozen {
                        return Ok(ValidationResult::Rejected);
                    } else if authority
                        == &(Authority::Pubkey {
                            address: *authority_info.key,
                        })
                    {
                        return Ok(ValidationResult::Approved);
                    }
                }
                Ok(ValidationResult::Pass)
            }
        }
    }
    mod lifecycle {
        use std::collections::BTreeMap;
        use solana_program::{account_info::AccountInfo, program_error::ProgramError};
        use crate::{error::MplCoreError, state::{Authority, CoreAsset, Key}};
        use super::{Plugin, PluginType, RegistryRecord};
        /// Lifecycle permissions
        /// Plugins use this field to indicate their permission to approve or deny
        /// a lifecycle action.
        pub enum CheckResult {
            /// A plugin is permitted to approve a lifecycle action.
            CanApprove,
            /// A plugin is permitted to reject a lifecycle action.
            CanReject,
            /// A plugin is not permitted to approve or reject a lifecycle action.
            None,
            /// Certain plugins can force approve a lifecycle action.
            CanForceApprove,
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for CheckResult {}
        #[automatically_derived]
        impl ::core::cmp::Eq for CheckResult {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for CheckResult {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for CheckResult {
            #[inline]
            fn eq(&self, other: &CheckResult) -> bool {
                let __self_tag = ::core::intrinsics::discriminant_value(self);
                let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                __self_tag == __arg1_tag
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for CheckResult {}
        #[automatically_derived]
        impl ::core::clone::Clone for CheckResult {
            #[inline]
            fn clone(&self) -> CheckResult {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for CheckResult {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        CheckResult::CanApprove => "CanApprove",
                        CheckResult::CanReject => "CanReject",
                        CheckResult::None => "None",
                        CheckResult::CanForceApprove => "CanForceApprove",
                    },
                )
            }
        }
        impl PluginType {
            /// Check permissions for the add plugin lifecycle event.
            pub fn check_add_plugin(plugin_type: &PluginType) -> CheckResult {
                match plugin_type {
                    PluginType::Royalties => CheckResult::CanReject,
                    PluginType::UpdateDelegate => CheckResult::CanApprove,
                    PluginType::PermanentFreeze => CheckResult::CanReject,
                    PluginType::PermanentTransfer => CheckResult::CanReject,
                    _ => CheckResult::None,
                }
            }
            /// Check permissions for the remove plugin lifecycle event.
            pub fn check_remove_plugin(plugin_type: &PluginType) -> CheckResult {
                #[allow(clippy::match_single_binding)]
                match plugin_type {
                    PluginType::UpdateDelegate => CheckResult::CanApprove,
                    _ => CheckResult::None,
                }
            }
            /// Check permissions for the approve plugin authority lifecycle event.
            pub fn check_approve_plugin_authority(
                plugin_type: &PluginType,
            ) -> CheckResult {
                #[allow(clippy::match_single_binding)]
                match plugin_type {
                    _ => CheckResult::CanApprove,
                }
            }
            /// Check permissions for the revoke plugin authority lifecycle event.
            pub fn check_revoke_plugin_authority(
                plugin_type: &PluginType,
            ) -> CheckResult {
                #[allow(clippy::match_single_binding)]
                match plugin_type {
                    _ => CheckResult::CanApprove,
                }
            }
            /// Check if a plugin is permitted to approve or deny a create action.
            pub fn check_create(plugin_type: &PluginType) -> CheckResult {
                #[allow(clippy::match_single_binding)]
                match plugin_type {
                    PluginType::Royalties => CheckResult::CanReject,
                    _ => CheckResult::None,
                }
            }
            /// Check if a plugin is permitted to approve or deny an update action.
            pub fn check_update(plugin_type: &PluginType) -> CheckResult {
                #[allow(clippy::match_single_binding)]
                match plugin_type {
                    _ => CheckResult::None,
                }
            }
            /// Check if a plugin is permitted to approve or deny a burn action.
            pub fn check_burn(plugin_type: &PluginType) -> CheckResult {
                match plugin_type {
                    PluginType::Freeze => CheckResult::CanReject,
                    PluginType::Burn => CheckResult::CanApprove,
                    PluginType::PermanentFreeze => CheckResult::CanReject,
                    PluginType::PermanentBurn => CheckResult::CanApprove,
                    _ => CheckResult::None,
                }
            }
            /// Check if a plugin is permitted to approve or deny a transfer action.
            pub fn check_transfer(plugin_type: &PluginType) -> CheckResult {
                match plugin_type {
                    PluginType::Royalties => CheckResult::CanReject,
                    PluginType::Freeze => CheckResult::CanReject,
                    PluginType::Transfer => CheckResult::CanApprove,
                    PluginType::PermanentFreeze => CheckResult::CanReject,
                    PluginType::PermanentTransfer => CheckResult::CanApprove,
                    _ => CheckResult::None,
                }
            }
            /// Check if a plugin is permitted to approve or deny a compress action.
            pub fn check_compress(plugin_type: &PluginType) -> CheckResult {
                #[allow(clippy::match_single_binding)]
                match plugin_type {
                    _ => CheckResult::None,
                }
            }
            /// Check if a plugin is permitted to approve or deny a decompress action.
            pub fn check_decompress(plugin_type: &PluginType) -> CheckResult {
                #[allow(clippy::match_single_binding)]
                match plugin_type {
                    _ => CheckResult::None,
                }
            }
        }
        impl Plugin {
            /// Validate the add plugin lifecycle event.
            pub fn validate_add_plugin(
                plugin: &Plugin,
                authority_info: &AccountInfo,
                _: Option<&AccountInfo>,
                authority: &Authority,
                new_plugin: Option<&Plugin>,
                _: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                match plugin {
                    Plugin::Royalties(royalties) => {
                        royalties
                            .validate_add_plugin(authority_info, authority, new_plugin)
                    }
                    Plugin::Freeze(freeze) => {
                        freeze.validate_add_plugin(authority_info, authority, new_plugin)
                    }
                    Plugin::Burn(burn) => {
                        burn.validate_add_plugin(authority_info, authority, new_plugin)
                    }
                    Plugin::Transfer(transfer) => {
                        transfer
                            .validate_add_plugin(authority_info, authority, new_plugin)
                    }
                    Plugin::UpdateDelegate(update_delegate) => {
                        update_delegate
                            .validate_add_plugin(authority_info, authority, new_plugin)
                    }
                    Plugin::PermanentFreeze(permanent_freeze) => {
                        permanent_freeze
                            .validate_add_plugin(authority_info, authority, new_plugin)
                    }
                    Plugin::Attributes(attributes) => {
                        attributes
                            .validate_add_plugin(authority_info, authority, new_plugin)
                    }
                    Plugin::PermanentTransfer(permanent_transfer) => {
                        permanent_transfer
                            .validate_add_plugin(authority_info, authority, new_plugin)
                    }
                    Plugin::PermanentBurn(permanent_burn) => {
                        permanent_burn
                            .validate_add_plugin(authority_info, authority, new_plugin)
                    }
                }
            }
            /// Validate the remove plugin lifecycle event.
            pub fn validate_remove_plugin(
                plugin: &Plugin,
                authority_info: &AccountInfo,
                _: Option<&AccountInfo>,
                authority: &Authority,
                plugin_to_remove: Option<&Plugin>,
                _: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                match plugin {
                    Plugin::Royalties(royalties) => {
                        royalties
                            .validate_remove_plugin(
                                authority_info,
                                authority,
                                plugin_to_remove,
                            )
                    }
                    Plugin::Freeze(freeze) => {
                        freeze
                            .validate_remove_plugin(
                                authority_info,
                                authority,
                                plugin_to_remove,
                            )
                    }
                    Plugin::Burn(burn) => {
                        burn.validate_remove_plugin(
                            authority_info,
                            authority,
                            plugin_to_remove,
                        )
                    }
                    Plugin::Transfer(transfer) => {
                        transfer
                            .validate_remove_plugin(
                                authority_info,
                                authority,
                                plugin_to_remove,
                            )
                    }
                    Plugin::UpdateDelegate(update_delegate) => {
                        update_delegate
                            .validate_remove_plugin(
                                authority_info,
                                authority,
                                plugin_to_remove,
                            )
                    }
                    Plugin::PermanentFreeze(permanent_freeze) => {
                        permanent_freeze
                            .validate_remove_plugin(
                                authority_info,
                                authority,
                                plugin_to_remove,
                            )
                    }
                    Plugin::Attributes(attributes) => {
                        attributes
                            .validate_remove_plugin(
                                authority_info,
                                authority,
                                plugin_to_remove,
                            )
                    }
                    Plugin::PermanentTransfer(permanent_transfer) => {
                        permanent_transfer
                            .validate_remove_plugin(
                                authority_info,
                                authority,
                                plugin_to_remove,
                            )
                    }
                    Plugin::PermanentBurn(permanent_burn) => {
                        permanent_burn
                            .validate_remove_plugin(
                                authority_info,
                                authority,
                                plugin_to_remove,
                            )
                    }
                }
            }
            /// Validate the approve plugin authority lifecycle event.
            pub fn validate_approve_plugin_authority(
                plugin: &Plugin,
                authority_info: &AccountInfo,
                _: Option<&AccountInfo>,
                authority: &Authority,
                plugin_to_approve: Option<&Plugin>,
                _: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                match plugin {
                    Plugin::Royalties(royalties) => {
                        royalties
                            .validate_approve_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_approve,
                            )
                    }
                    Plugin::Freeze(freeze) => {
                        freeze
                            .validate_approve_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_approve,
                            )
                    }
                    Plugin::Burn(burn) => {
                        burn.validate_approve_plugin_authority(
                            authority_info,
                            authority,
                            plugin_to_approve,
                        )
                    }
                    Plugin::Transfer(transfer) => {
                        transfer
                            .validate_approve_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_approve,
                            )
                    }
                    Plugin::UpdateDelegate(update_delegate) => {
                        update_delegate
                            .validate_approve_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_approve,
                            )
                    }
                    Plugin::PermanentFreeze(permanent_freeze) => {
                        permanent_freeze
                            .validate_approve_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_approve,
                            )
                    }
                    Plugin::Attributes(attributes) => {
                        attributes
                            .validate_approve_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_approve,
                            )
                    }
                    Plugin::PermanentTransfer(permanent_transfer) => {
                        permanent_transfer
                            .validate_approve_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_approve,
                            )
                    }
                    Plugin::PermanentBurn(permanent_burn) => {
                        permanent_burn
                            .validate_approve_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_approve,
                            )
                    }
                }
            }
            /// Validate the revoke plugin authority lifecycle event.
            pub fn validate_revoke_plugin_authority(
                plugin: &Plugin,
                authority_info: &AccountInfo,
                _: Option<&AccountInfo>,
                authority: &Authority,
                plugin_to_revoke: Option<&Plugin>,
                _: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                if authority == &Authority::None {
                    return Ok(ValidationResult::Rejected);
                }
                match plugin {
                    Plugin::Royalties(royalties) => {
                        royalties
                            .validate_revoke_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_revoke,
                            )
                    }
                    Plugin::Freeze(freeze) => {
                        freeze
                            .validate_revoke_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_revoke,
                            )
                    }
                    Plugin::Burn(burn) => {
                        burn.validate_revoke_plugin_authority(
                            authority_info,
                            authority,
                            plugin_to_revoke,
                        )
                    }
                    Plugin::Transfer(transfer) => {
                        transfer
                            .validate_revoke_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_revoke,
                            )
                    }
                    Plugin::UpdateDelegate(update_delegate) => {
                        update_delegate
                            .validate_revoke_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_revoke,
                            )
                    }
                    Plugin::PermanentFreeze(permanent_freeze) => {
                        permanent_freeze
                            .validate_revoke_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_revoke,
                            )
                    }
                    Plugin::Attributes(attributes) => {
                        attributes
                            .validate_revoke_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_revoke,
                            )
                    }
                    Plugin::PermanentTransfer(permanent_transfer) => {
                        permanent_transfer
                            .validate_revoke_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_revoke,
                            )
                    }
                    Plugin::PermanentBurn(permanent_burn) => {
                        permanent_burn
                            .validate_revoke_plugin_authority(
                                authority_info,
                                authority,
                                plugin_to_revoke,
                            )
                    }
                }
            }
            /// Route the validation of the create action to the appropriate plugin.
            pub(crate) fn validate_create(
                plugin: &Plugin,
                authority_info: &AccountInfo,
                _: Option<&AccountInfo>,
                authority: &Authority,
                _: Option<&Plugin>,
                _: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                match plugin {
                    Plugin::Royalties(royalties) => {
                        royalties.validate_create(authority_info, authority)
                    }
                    Plugin::Freeze(freeze) => {
                        freeze.validate_create(authority_info, authority)
                    }
                    Plugin::Burn(burn) => burn.validate_create(authority_info, authority),
                    Plugin::Transfer(transfer) => {
                        transfer.validate_create(authority_info, authority)
                    }
                    Plugin::UpdateDelegate(update_delegate) => {
                        update_delegate.validate_create(authority_info, authority)
                    }
                    Plugin::PermanentFreeze(permanent_freeze) => {
                        permanent_freeze.validate_create(authority_info, authority)
                    }
                    Plugin::Attributes(attributes) => {
                        attributes.validate_create(authority_info, authority)
                    }
                    Plugin::PermanentTransfer(permanent_transfer) => {
                        permanent_transfer.validate_create(authority_info, authority)
                    }
                    Plugin::PermanentBurn(permanent_burn) => {
                        permanent_burn.validate_create(authority_info, authority)
                    }
                }
            }
            /// Route the validation of the update action to the appropriate plugin.
            pub(crate) fn validate_update(
                plugin: &Plugin,
                authority_info: &AccountInfo,
                _: Option<&AccountInfo>,
                authority: &Authority,
                _: Option<&Plugin>,
                _: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                match plugin {
                    Plugin::Royalties(royalties) => {
                        royalties.validate_update(authority_info, authority)
                    }
                    Plugin::Freeze(freeze) => {
                        freeze.validate_update(authority_info, authority)
                    }
                    Plugin::Burn(burn) => burn.validate_update(authority_info, authority),
                    Plugin::Transfer(transfer) => {
                        transfer.validate_update(authority_info, authority)
                    }
                    Plugin::UpdateDelegate(update_delegate) => {
                        update_delegate.validate_update(authority_info, authority)
                    }
                    Plugin::PermanentFreeze(permanent_freeze) => {
                        permanent_freeze.validate_update(authority_info, authority)
                    }
                    Plugin::Attributes(attributes) => {
                        attributes.validate_update(authority_info, authority)
                    }
                    Plugin::PermanentTransfer(permanent_transfer) => {
                        permanent_transfer.validate_update(authority_info, authority)
                    }
                    Plugin::PermanentBurn(permanent_burn) => {
                        permanent_burn.validate_update(authority_info, authority)
                    }
                }
            }
            /// Route the validation of the update_plugin action to the appropriate plugin.
            /// There is no check for updating a plugin because the plugin itself MUST validate the change.
            pub(crate) fn validate_update_plugin<T: CoreAsset>(
                plugin: &Plugin,
                core_asset: &T,
                authority_info: &AccountInfo,
                _: Option<&AccountInfo>,
                authority: &Authority,
                _: Option<&Plugin>,
                _: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                match plugin {
                    Plugin::Royalties(royalties) => {
                        royalties
                            .validate_update_plugin(
                                core_asset,
                                authority_info,
                                authority,
                            )
                    }
                    Plugin::Freeze(freeze) => {
                        freeze
                            .validate_update_plugin(
                                core_asset,
                                authority_info,
                                authority,
                            )
                    }
                    Plugin::Burn(burn) => {
                        burn.validate_update_plugin(
                            core_asset,
                            authority_info,
                            authority,
                        )
                    }
                    Plugin::Transfer(transfer) => {
                        transfer
                            .validate_update_plugin(
                                core_asset,
                                authority_info,
                                authority,
                            )
                    }
                    Plugin::UpdateDelegate(update_delegate) => {
                        update_delegate
                            .validate_update_plugin(
                                core_asset,
                                authority_info,
                                authority,
                            )
                    }
                    Plugin::PermanentFreeze(permanent_freeze) => {
                        permanent_freeze
                            .validate_update_plugin(
                                core_asset,
                                authority_info,
                                authority,
                            )
                    }
                    Plugin::Attributes(attributes) => {
                        attributes
                            .validate_update_plugin(
                                core_asset,
                                authority_info,
                                authority,
                            )
                    }
                    Plugin::PermanentTransfer(permanent_transfer) => {
                        permanent_transfer
                            .validate_update_plugin(
                                core_asset,
                                authority_info,
                                authority,
                            )
                    }
                    Plugin::PermanentBurn(permanent_burn) => {
                        permanent_burn
                            .validate_update_plugin(
                                core_asset,
                                authority_info,
                                authority,
                            )
                    }
                }
            }
            /// Route the validation of the burn action to the appropriate plugin.
            pub(crate) fn validate_burn(
                plugin: &Plugin,
                authority_info: &AccountInfo,
                _: Option<&AccountInfo>,
                authority: &Authority,
                _: Option<&Plugin>,
                resolved_authority: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                match plugin {
                    Plugin::Royalties(royalties) => {
                        royalties
                            .validate_burn(authority_info, authority, resolved_authority)
                    }
                    Plugin::Freeze(freeze) => {
                        freeze
                            .validate_burn(authority_info, authority, resolved_authority)
                    }
                    Plugin::Burn(burn) => {
                        burn.validate_burn(authority_info, authority, resolved_authority)
                    }
                    Plugin::Transfer(transfer) => {
                        transfer
                            .validate_burn(authority_info, authority, resolved_authority)
                    }
                    Plugin::UpdateDelegate(update_delegate) => {
                        update_delegate
                            .validate_burn(authority_info, authority, resolved_authority)
                    }
                    Plugin::PermanentFreeze(permanent_freeze) => {
                        permanent_freeze
                            .validate_burn(authority_info, authority, resolved_authority)
                    }
                    Plugin::Attributes(attributes) => {
                        attributes
                            .validate_burn(authority_info, authority, resolved_authority)
                    }
                    Plugin::PermanentTransfer(permanent_transfer) => {
                        permanent_transfer
                            .validate_burn(authority_info, authority, resolved_authority)
                    }
                    Plugin::PermanentBurn(permanent_burn) => {
                        permanent_burn
                            .validate_burn(authority_info, authority, resolved_authority)
                    }
                }
            }
            /// Route the validation of the transfer action to the appropriate plugin.
            pub(crate) fn validate_transfer(
                plugin: &Plugin,
                authority_info: &AccountInfo,
                new_owner: Option<&AccountInfo>,
                authority: &Authority,
                _: Option<&Plugin>,
                resolved_authority: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                let new_owner = new_owner.ok_or(MplCoreError::MissingNewOwner)?;
                match plugin {
                    Plugin::Royalties(royalties) => {
                        royalties
                            .validate_transfer(
                                authority_info,
                                new_owner,
                                authority,
                                resolved_authority,
                            )
                    }
                    Plugin::Freeze(freeze) => {
                        freeze
                            .validate_transfer(
                                authority_info,
                                new_owner,
                                authority,
                                resolved_authority,
                            )
                    }
                    Plugin::Burn(burn) => {
                        burn.validate_transfer(
                            authority_info,
                            new_owner,
                            authority,
                            resolved_authority,
                        )
                    }
                    Plugin::Transfer(transfer) => {
                        transfer
                            .validate_transfer(
                                authority_info,
                                new_owner,
                                authority,
                                resolved_authority,
                            )
                    }
                    Plugin::UpdateDelegate(update_delegate) => {
                        update_delegate
                            .validate_transfer(
                                authority_info,
                                new_owner,
                                authority,
                                resolved_authority,
                            )
                    }
                    Plugin::PermanentFreeze(permanent_freeze) => {
                        permanent_freeze
                            .validate_transfer(
                                authority_info,
                                new_owner,
                                authority,
                                resolved_authority,
                            )
                    }
                    Plugin::PermanentTransfer(permanent_transfer) => {
                        permanent_transfer
                            .validate_transfer(
                                authority_info,
                                new_owner,
                                authority,
                                resolved_authority,
                            )
                    }
                    Plugin::Attributes(attributes_transfer) => {
                        attributes_transfer
                            .validate_transfer(
                                authority_info,
                                new_owner,
                                authority,
                                resolved_authority,
                            )
                    }
                    Plugin::PermanentBurn(burn_transfer) => {
                        burn_transfer
                            .validate_transfer(
                                authority_info,
                                new_owner,
                                authority,
                                resolved_authority,
                            )
                    }
                }
            }
            /// Route the validation of the compress action to the appropriate plugin.
            pub(crate) fn validate_compress(
                plugin: &Plugin,
                authority_info: &AccountInfo,
                _: Option<&AccountInfo>,
                authority: &Authority,
                _: Option<&Plugin>,
                _: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                match plugin {
                    Plugin::Royalties(royalties) => {
                        royalties.validate_compress(authority_info, authority)
                    }
                    Plugin::Freeze(freeze) => {
                        freeze.validate_compress(authority_info, authority)
                    }
                    Plugin::Burn(burn) => {
                        burn.validate_compress(authority_info, authority)
                    }
                    Plugin::Transfer(transfer) => {
                        transfer.validate_compress(authority_info, authority)
                    }
                    Plugin::UpdateDelegate(update_delegate) => {
                        update_delegate.validate_compress(authority_info, authority)
                    }
                    Plugin::PermanentFreeze(permanent_freeze) => {
                        permanent_freeze.validate_compress(authority_info, authority)
                    }
                    Plugin::Attributes(attributes) => {
                        attributes.validate_compress(authority_info, authority)
                    }
                    Plugin::PermanentTransfer(permanent_transfer) => {
                        permanent_transfer.validate_compress(authority_info, authority)
                    }
                    Plugin::PermanentBurn(burn_transfer) => {
                        burn_transfer.validate_compress(authority_info, authority)
                    }
                }
            }
            /// Route the validation of the decompress action to the appropriate plugin.
            pub(crate) fn validate_decompress(
                plugin: &Plugin,
                authority_info: &AccountInfo,
                _: Option<&AccountInfo>,
                authority: &Authority,
                _: Option<&Plugin>,
                _: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                match plugin {
                    Plugin::Royalties(royalties) => {
                        royalties.validate_decompress(authority_info, authority)
                    }
                    Plugin::Freeze(freeze) => {
                        freeze.validate_decompress(authority_info, authority)
                    }
                    Plugin::Burn(burn) => {
                        burn.validate_decompress(authority_info, authority)
                    }
                    Plugin::Transfer(transfer) => {
                        transfer.validate_decompress(authority_info, authority)
                    }
                    Plugin::UpdateDelegate(update_delegate) => {
                        update_delegate.validate_decompress(authority_info, authority)
                    }
                    Plugin::PermanentFreeze(permanent_freeze) => {
                        permanent_freeze.validate_decompress(authority_info, authority)
                    }
                    Plugin::Attributes(attributes) => {
                        attributes.validate_decompress(authority_info, authority)
                    }
                    Plugin::PermanentTransfer(permanent_transfer) => {
                        permanent_transfer.validate_decompress(authority_info, authority)
                    }
                    Plugin::PermanentBurn(permanent_burn) => {
                        permanent_burn.validate_decompress(authority_info, authority)
                    }
                }
            }
        }
        /// Lifecycle validations
        /// Plugins utilize this to indicate whether they approve or reject a lifecycle action.
        pub enum ValidationResult {
            /// The plugin approves the lifecycle action.
            Approved,
            /// The plugin rejects the lifecycle action.
            Rejected,
            /// The plugin abstains from approving or rejecting the lifecycle action.
            Pass,
            /// The plugin force approves the lifecycle action.
            ForceApproved,
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for ValidationResult {}
        #[automatically_derived]
        impl ::core::cmp::Eq for ValidationResult {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for ValidationResult {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for ValidationResult {
            #[inline]
            fn eq(&self, other: &ValidationResult) -> bool {
                let __self_tag = ::core::intrinsics::discriminant_value(self);
                let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                __self_tag == __arg1_tag
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ValidationResult {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        ValidationResult::Approved => "Approved",
                        ValidationResult::Rejected => "Rejected",
                        ValidationResult::Pass => "Pass",
                        ValidationResult::ForceApproved => "ForceApproved",
                    },
                )
            }
        }
        /// The required context for a plugin validation.
        #[allow(dead_code)]
        pub(crate) struct PluginValidationContext<'a, 'b> {
            /// The authority.
            pub self_authority: &'b Authority,
            /// The authority account.
            pub authority_info: &'a AccountInfo<'a>,
            /// The resolved authority.
            pub resolved_authority: Option<&'a Authority>,
            /// The new owner account.
            pub new_owner: Option<&'a AccountInfo<'a>>,
            /// The new plugin.
            pub target_plugin: Option<&'a Plugin>,
        }
        /// Plugin validation trait which is implemented by each plugin.
        pub(crate) trait PluginValidation {
            /// Validate the add plugin lifecycle action.
            fn validate_add_plugin(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
                _new_plugin: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the remove plugin lifecycle action.
            fn validate_remove_plugin(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
                _plugin_to_remove: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the approve plugin authority lifecycle action.
            fn validate_approve_plugin_authority(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
                _plugin_to_approve: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the revoke plugin authority lifecycle action.
            fn validate_revoke_plugin_authority(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
                _plugin_to_revoke: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the create lifecycle action.
            fn validate_create(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the update lifecycle action.
            fn validate_update(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the update_plugin lifecycle action.
            fn validate_update_plugin<T: CoreAsset>(
                &self,
                core_asset: &T,
                authority_info: &AccountInfo,
                authority: &Authority,
            ) -> Result<ValidationResult, ProgramError> {
                if (authority_info.key == core_asset.owner()
                    && authority == &Authority::Owner)
                    || (authority_info.key == &core_asset.update_authority().key()
                        && authority == &Authority::UpdateAuthority)
                    || authority
                        == (&Authority::Pubkey {
                            address: *authority_info.key,
                        })
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the burn lifecycle action.
            fn validate_burn(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
                _resolved_authority: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the transfer lifecycle action.
            fn validate_transfer(
                &self,
                _authority_info: &AccountInfo,
                _new_owner: &AccountInfo,
                _authority: &Authority,
                _resolved_authority: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the compress lifecycle action.
            fn validate_compress(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the decompress lifecycle action.
            fn validate_decompress(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the add_authority lifecycle action.
            fn validate_add_authority(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
            ) -> Result<super::ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the add_authority lifecycle action.
            fn validate_remove_authority(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
            ) -> Result<super::ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
        }
        /// This function iterates through all plugin checks passed in and performs the validation
        /// by deserializing and calling validate on the plugin.
        /// The STRONGEST result is returned.
        #[allow(clippy::too_many_arguments, clippy::type_complexity)]
        pub(crate) fn validate_plugin_checks<'a>(
            key: Key,
            checks: &BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
            authority: &AccountInfo<'a>,
            new_owner: Option<&AccountInfo>,
            new_plugin: Option<&Plugin>,
            asset: Option<&AccountInfo<'a>>,
            collection: Option<&AccountInfo<'a>>,
            resolved_authority: &Authority,
            validate_fp: fn(
                &Plugin,
                &AccountInfo<'a>,
                Option<&AccountInfo>,
                &Authority,
                Option<&Plugin>,
                Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError>,
        ) -> Result<ValidationResult, ProgramError> {
            let mut approved = false;
            let mut rejected = false;
            for (check_key, check_result, registry_record) in checks.values() {
                if *check_key == key
                    && match check_result {
                        CheckResult::CanApprove | CheckResult::CanReject => true,
                        _ => false,
                    }
                {
                    ::solana_program::log::sol_log("Validating plugin checks");
                    let account = match key {
                        Key::Collection => {
                            collection.ok_or(MplCoreError::InvalidCollection)?
                        }
                        Key::Asset => asset.ok_or(MplCoreError::InvalidAsset)?,
                        _ => {
                            ::core::panicking::panic(
                                "internal error: entered unreachable code",
                            )
                        }
                    };
                    let result = validate_fp(
                        &Plugin::load(account, registry_record.offset)?,
                        authority,
                        new_owner,
                        &registry_record.authority,
                        new_plugin,
                        Some(resolved_authority),
                    )?;
                    match result {
                        ValidationResult::Rejected => rejected = true,
                        ValidationResult::Approved => approved = true,
                        ValidationResult::Pass => continue,
                        ValidationResult::ForceApproved => {
                            return Ok(ValidationResult::ForceApproved);
                        }
                    }
                }
            }
            if rejected {
                Ok(ValidationResult::Rejected)
            } else if approved {
                Ok(ValidationResult::Approved)
            } else {
                Ok(ValidationResult::Pass)
            }
        }
    }
    mod permanent_burn {
        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::{account_info::AccountInfo, program_error::ProgramError};
        use crate::state::{Authority, DataBlob};
        use super::{Plugin, PluginValidation, ValidationResult};
        /// The permanent burn plugin allows any authority to burn the asset.
        /// The default authority for this plugin is the update authority.
        #[repr(C)]
        pub struct PermanentBurn {}
        #[automatically_derived]
        impl ::core::clone::Clone for PermanentBurn {
            #[inline]
            fn clone(&self) -> PermanentBurn {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for PermanentBurn {}
        impl borsh::ser::BorshSerialize for PermanentBurn {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for PermanentBurn {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {})
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for PermanentBurn {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "PermanentBurn")
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for PermanentBurn {
            #[inline]
            fn default() -> PermanentBurn {
                PermanentBurn {}
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for PermanentBurn {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for PermanentBurn {
            #[inline]
            fn eq(&self, other: &PermanentBurn) -> bool {
                true
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for PermanentBurn {}
        #[automatically_derived]
        impl ::core::cmp::Eq for PermanentBurn {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        impl DataBlob for PermanentBurn {
            fn get_initial_size() -> usize {
                0
            }
            fn get_size(&self) -> usize {
                0
            }
        }
        impl PluginValidation for PermanentBurn {
            fn validate_add_plugin(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
                _new_plugin: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Rejected)
            }
            fn validate_revoke_plugin_authority(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
                _plugin_to_revoke: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Approved)
            }
            fn validate_burn(
                &self,
                _authority_info: &AccountInfo,
                authority: &Authority,
                resolved_authority: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                if let Some(resolved_authority) = resolved_authority {
                    if resolved_authority == authority {
                        return Ok(ValidationResult::ForceApproved);
                    }
                }
                Ok(ValidationResult::Pass)
            }
        }
    }
    mod permanent_freeze {
        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::{account_info::AccountInfo, program_error::ProgramError};
        use crate::{plugins::PluginType, state::{Authority, CoreAsset, DataBlob}};
        use super::{Plugin, PluginValidation, ValidationResult};
        /// The permanent freeze plugin allows any authority to lock the asset so it's no longer transferable.
        /// The default authority for this plugin is the update authority.
        #[repr(C)]
        pub struct PermanentFreeze {
            /// The current state of the asset and whether or not it's transferable.
            pub frozen: bool,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for PermanentFreeze {
            #[inline]
            fn clone(&self) -> PermanentFreeze {
                let _: ::core::clone::AssertParamIsClone<bool>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for PermanentFreeze {}
        impl borsh::ser::BorshSerialize for PermanentFreeze
        where
            bool: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.frozen, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for PermanentFreeze
        where
            bool: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    frozen: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for PermanentFreeze {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "PermanentFreeze",
                    "frozen",
                    &&self.frozen,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for PermanentFreeze {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for PermanentFreeze {
            #[inline]
            fn eq(&self, other: &PermanentFreeze) -> bool {
                self.frozen == other.frozen
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for PermanentFreeze {}
        #[automatically_derived]
        impl ::core::cmp::Eq for PermanentFreeze {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<bool>;
            }
        }
        impl PermanentFreeze {
            /// Initialize the PermanentFreeze plugin, unfrozen by default.
            pub fn new() -> Self {
                Self { frozen: false }
            }
        }
        impl Default for PermanentFreeze {
            fn default() -> Self {
                Self::new()
            }
        }
        impl DataBlob for PermanentFreeze {
            fn get_initial_size() -> usize {
                1
            }
            fn get_size(&self) -> usize {
                1
            }
        }
        impl PluginValidation for PermanentFreeze {
            fn validate_update_plugin<T: CoreAsset>(
                &self,
                core_asset: &T,
                authority_info: &AccountInfo,
                authority: &Authority,
            ) -> Result<ValidationResult, ProgramError> {
                if (authority_info.key == &core_asset.update_authority().key()
                    && authority == (&Authority::UpdateAuthority))
                    || authority
                        == (&Authority::Pubkey {
                            address: *authority_info.key,
                        })
                    || (authority_info.key == core_asset.owner()
                        && authority == (&Authority::Owner))
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            fn validate_burn(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
                _resolved_authority: Option<&Authority>,
            ) -> Result<super::ValidationResult, ProgramError> {
                if self.frozen {
                    Ok(ValidationResult::Rejected)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            fn validate_transfer(
                &self,
                _authority_info: &AccountInfo,
                _new_owner: &AccountInfo,
                _authority: &Authority,
                _resolved_authority: Option<&Authority>,
            ) -> Result<super::ValidationResult, ProgramError> {
                if self.frozen {
                    Ok(ValidationResult::Rejected)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            fn validate_add_authority(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
            ) -> Result<super::ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            fn validate_add_plugin(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
                _new_plugin: Option<&super::Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Rejected)
            }
            /// Validate the revoke plugin authority lifecycle action.
            fn validate_revoke_plugin_authority(
                &self,
                authority_info: &AccountInfo,
                authority: &Authority,
                plugin_to_revoke: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if authority
                    == &(Authority::Pubkey {
                        address: *authority_info.key,
                    }) && plugin_to_revoke.is_some()
                    && PluginType::from(plugin_to_revoke.unwrap()) == PluginType::Freeze
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
        }
    }
    mod permanent_transfer {
        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::{account_info::AccountInfo, program_error::ProgramError};
        use crate::state::{Authority, DataBlob};
        use super::{Plugin, PluginValidation, ValidationResult};
        /// The permanent transfer plugin allows any authority to transfer the asset.
        /// The default authority for this plugin is the update authority.
        #[repr(C)]
        pub struct PermanentTransfer {}
        #[automatically_derived]
        impl ::core::clone::Clone for PermanentTransfer {
            #[inline]
            fn clone(&self) -> PermanentTransfer {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for PermanentTransfer {}
        impl borsh::ser::BorshSerialize for PermanentTransfer {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for PermanentTransfer {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {})
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for PermanentTransfer {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "PermanentTransfer")
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for PermanentTransfer {
            #[inline]
            fn default() -> PermanentTransfer {
                PermanentTransfer {}
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for PermanentTransfer {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for PermanentTransfer {
            #[inline]
            fn eq(&self, other: &PermanentTransfer) -> bool {
                true
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for PermanentTransfer {}
        #[automatically_derived]
        impl ::core::cmp::Eq for PermanentTransfer {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        impl DataBlob for PermanentTransfer {
            fn get_initial_size() -> usize {
                0
            }
            fn get_size(&self) -> usize {
                0
            }
        }
        impl PluginValidation for PermanentTransfer {
            fn validate_add_plugin(
                &self,
                _authority: &AccountInfo,
                _authorities: &Authority,
                _new_plugin: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Rejected)
            }
            fn validate_revoke_plugin_authority(
                &self,
                _authority: &AccountInfo,
                _authorities: &Authority,
                _plugin_to_revoke: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Approved)
            }
            fn validate_transfer(
                &self,
                _authority_info: &AccountInfo,
                _new_owner: &AccountInfo,
                authority: &Authority,
                resolved_authority: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                if let Some(resolved_authority) = resolved_authority {
                    if resolved_authority == authority {
                        return Ok(ValidationResult::ForceApproved);
                    }
                }
                Ok(ValidationResult::Pass)
            }
        }
    }
    mod plugin_header {
        use crate::state::{DataBlob, Key, SolanaAccount};
        use borsh::{BorshDeserialize, BorshSerialize};
        use shank::ShankAccount;
        /// The plugin header is the first part of the plugin metadata.
        /// This field stores the Key
        /// And a pointer to the Plugin Registry stored at the end of the account.
        #[repr(C)]
        pub struct PluginHeader {
            /// The Discriminator of the header which doubles as a Plugin metadata version.
            pub key: Key,
            /// The offset to the plugin registry stored at the end of the account.
            pub plugin_registry_offset: usize,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for PluginHeader {
            #[inline]
            fn clone(&self) -> PluginHeader {
                PluginHeader {
                    key: ::core::clone::Clone::clone(&self.key),
                    plugin_registry_offset: ::core::clone::Clone::clone(
                        &self.plugin_registry_offset,
                    ),
                }
            }
        }
        impl borsh::ser::BorshSerialize for PluginHeader
        where
            Key: borsh::ser::BorshSerialize,
            usize: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.key, writer)?;
                borsh::BorshSerialize::serialize(&self.plugin_registry_offset, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for PluginHeader
        where
            Key: borsh::BorshDeserialize,
            usize: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    key: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    plugin_registry_offset: borsh::BorshDeserialize::deserialize_reader(
                        reader,
                    )?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for PluginHeader {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "PluginHeader",
                    "key",
                    &self.key,
                    "plugin_registry_offset",
                    &&self.plugin_registry_offset,
                )
            }
        }
        impl DataBlob for PluginHeader {
            fn get_initial_size() -> usize {
                1 + 8
            }
            fn get_size(&self) -> usize {
                1 + 8
            }
        }
        impl SolanaAccount for PluginHeader {
            fn key() -> Key {
                Key::PluginHeader
            }
        }
    }
    mod plugin_registry {
        use borsh::{BorshDeserialize, BorshSerialize};
        use shank::ShankAccount;
        use std::{cmp::Ordering, collections::BTreeMap};
        use crate::state::{Authority, DataBlob, Key, SolanaAccount};
        use super::{CheckResult, PluginType};
        /// The Plugin Registry stores a record of all plugins, their location, and their authorities.
        #[repr(C)]
        pub struct PluginRegistry {
            /// The Discriminator of the header which doubles as a Plugin metadata version.
            pub key: Key,
            /// The registry of all plugins.
            pub registry: Vec<RegistryRecord>,
            /// The registry of all external, third party, plugins.
            pub external_plugins: Vec<ExternalPluginRecord>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for PluginRegistry {
            #[inline]
            fn clone(&self) -> PluginRegistry {
                PluginRegistry {
                    key: ::core::clone::Clone::clone(&self.key),
                    registry: ::core::clone::Clone::clone(&self.registry),
                    external_plugins: ::core::clone::Clone::clone(&self.external_plugins),
                }
            }
        }
        impl borsh::ser::BorshSerialize for PluginRegistry
        where
            Key: borsh::ser::BorshSerialize,
            Vec<RegistryRecord>: borsh::ser::BorshSerialize,
            Vec<ExternalPluginRecord>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.key, writer)?;
                borsh::BorshSerialize::serialize(&self.registry, writer)?;
                borsh::BorshSerialize::serialize(&self.external_plugins, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for PluginRegistry
        where
            Key: borsh::BorshDeserialize,
            Vec<RegistryRecord>: borsh::BorshDeserialize,
            Vec<ExternalPluginRecord>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    key: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    registry: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    external_plugins: borsh::BorshDeserialize::deserialize_reader(
                        reader,
                    )?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for PluginRegistry {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "PluginRegistry",
                    "key",
                    &self.key,
                    "registry",
                    &self.registry,
                    "external_plugins",
                    &&self.external_plugins,
                )
            }
        }
        impl PluginRegistry {
            /// Evaluate checks for all plugins in the registry.
            pub(crate) fn check_registry(
                &self,
                key: Key,
                check_fp: fn(&PluginType) -> CheckResult,
                result: &mut BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
            ) {
                for record in &self.registry {
                    result
                        .insert(
                            record.plugin_type,
                            (key, check_fp(&record.plugin_type), record.clone()),
                        );
                }
            }
        }
        impl DataBlob for PluginRegistry {
            fn get_initial_size() -> usize {
                9
            }
            fn get_size(&self) -> usize {
                9
            }
        }
        impl SolanaAccount for PluginRegistry {
            fn key() -> Key {
                Key::PluginRegistry
            }
        }
        /// A simple type to store the mapping of Plugin type to Plugin data.
        #[repr(C)]
        pub struct RegistryRecord {
            /// The type of plugin.
            pub plugin_type: PluginType,
            /// The authority who has permission to utilize a plugin.
            pub authority: Authority,
            /// The offset to the plugin in the account.
            pub offset: usize,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for RegistryRecord {
            #[inline]
            fn clone(&self) -> RegistryRecord {
                RegistryRecord {
                    plugin_type: ::core::clone::Clone::clone(&self.plugin_type),
                    authority: ::core::clone::Clone::clone(&self.authority),
                    offset: ::core::clone::Clone::clone(&self.offset),
                }
            }
        }
        impl borsh::ser::BorshSerialize for RegistryRecord
        where
            PluginType: borsh::ser::BorshSerialize,
            Authority: borsh::ser::BorshSerialize,
            usize: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.plugin_type, writer)?;
                borsh::BorshSerialize::serialize(&self.authority, writer)?;
                borsh::BorshSerialize::serialize(&self.offset, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for RegistryRecord
        where
            PluginType: borsh::BorshDeserialize,
            Authority: borsh::BorshDeserialize,
            usize: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    plugin_type: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    authority: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    offset: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for RegistryRecord {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "RegistryRecord",
                    "plugin_type",
                    &self.plugin_type,
                    "authority",
                    &self.authority,
                    "offset",
                    &&self.offset,
                )
            }
        }
        impl RegistryRecord {
            /// Associated function for sorting `RegistryRecords` by offset.
            pub fn compare_offsets(a: &RegistryRecord, b: &RegistryRecord) -> Ordering {
                a.offset.cmp(&b.offset)
            }
        }
        /// A simple type to store the mapping of external Plugin authority to Plugin data.
        #[repr(C)]
        pub struct ExternalPluginRecord {
            /// The authority of the external plugin.
            pub authority: Authority,
            /// The offset to the plugin in the account.
            pub offset: usize,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for ExternalPluginRecord {
            #[inline]
            fn clone(&self) -> ExternalPluginRecord {
                ExternalPluginRecord {
                    authority: ::core::clone::Clone::clone(&self.authority),
                    offset: ::core::clone::Clone::clone(&self.offset),
                }
            }
        }
        impl borsh::ser::BorshSerialize for ExternalPluginRecord
        where
            Authority: borsh::ser::BorshSerialize,
            usize: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.authority, writer)?;
                borsh::BorshSerialize::serialize(&self.offset, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for ExternalPluginRecord
        where
            Authority: borsh::BorshDeserialize,
            usize: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    authority: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    offset: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ExternalPluginRecord {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "ExternalPluginRecord",
                    "authority",
                    &self.authority,
                    "offset",
                    &&self.offset,
                )
            }
        }
    }
    mod royalties {
        use std::collections::HashSet;
        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::{
            account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey,
        };
        use crate::{plugins::PluginType, state::Authority};
        use super::{Plugin, PluginValidation, ValidationResult};
        /// The creator on an asset and whether or not they are verified.
        pub struct Creator {
            address: Pubkey,
            percentage: u8,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Creator {
            #[inline]
            fn clone(&self) -> Creator {
                Creator {
                    address: ::core::clone::Clone::clone(&self.address),
                    percentage: ::core::clone::Clone::clone(&self.percentage),
                }
            }
        }
        impl borsh::ser::BorshSerialize for Creator
        where
            Pubkey: borsh::ser::BorshSerialize,
            u8: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.address, writer)?;
                borsh::BorshSerialize::serialize(&self.percentage, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for Creator
        where
            Pubkey: borsh::BorshDeserialize,
            u8: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    address: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    percentage: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Creator {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "Creator",
                    "address",
                    &self.address,
                    "percentage",
                    &&self.percentage,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Creator {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Creator {
            #[inline]
            fn eq(&self, other: &Creator) -> bool {
                self.address == other.address && self.percentage == other.percentage
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for Creator {}
        #[automatically_derived]
        impl ::core::cmp::Eq for Creator {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Pubkey>;
                let _: ::core::cmp::AssertParamIsEq<u8>;
            }
        }
        /// The rule set for an asset indicating where it is allowed to be transferred.
        pub enum RuleSet {
            /// No rules are enforced.
            None,
            /// Allow list of programs that are allowed to transfer, receive, or send the asset.
            ProgramAllowList(Vec<Pubkey>),
            /// Deny list of programs that are not allowed to transfer, receive, or send the asset.
            ProgramDenyList(Vec<Pubkey>),
        }
        #[automatically_derived]
        impl ::core::clone::Clone for RuleSet {
            #[inline]
            fn clone(&self) -> RuleSet {
                match self {
                    RuleSet::None => RuleSet::None,
                    RuleSet::ProgramAllowList(__self_0) => {
                        RuleSet::ProgramAllowList(::core::clone::Clone::clone(__self_0))
                    }
                    RuleSet::ProgramDenyList(__self_0) => {
                        RuleSet::ProgramDenyList(::core::clone::Clone::clone(__self_0))
                    }
                }
            }
        }
        impl borsh::ser::BorshSerialize for RuleSet
        where
            Vec<Pubkey>: borsh::ser::BorshSerialize,
            Vec<Pubkey>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                let variant_idx: u8 = match self {
                    RuleSet::None => 0u8,
                    RuleSet::ProgramAllowList(..) => 1u8,
                    RuleSet::ProgramDenyList(..) => 2u8,
                };
                writer.write_all(&variant_idx.to_le_bytes())?;
                match self {
                    RuleSet::None => {}
                    RuleSet::ProgramAllowList(id0) => {
                        borsh::BorshSerialize::serialize(id0, writer)?;
                    }
                    RuleSet::ProgramDenyList(id0) => {
                        borsh::BorshSerialize::serialize(id0, writer)?;
                    }
                }
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for RuleSet
        where
            Vec<Pubkey>: borsh::BorshDeserialize,
            Vec<Pubkey>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                let tag = <u8 as borsh::de::BorshDeserialize>::deserialize_reader(
                    reader,
                )?;
                <Self as borsh::de::EnumExt>::deserialize_variant(reader, tag)
            }
        }
        impl borsh::de::EnumExt for RuleSet
        where
            Vec<Pubkey>: borsh::BorshDeserialize,
            Vec<Pubkey>: borsh::BorshDeserialize,
        {
            fn deserialize_variant<R: borsh::maybestd::io::Read>(
                reader: &mut R,
                variant_idx: u8,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                let mut return_value = match variant_idx {
                    0u8 => RuleSet::None,
                    1u8 => {
                        RuleSet::ProgramAllowList(
                            borsh::BorshDeserialize::deserialize_reader(reader)?,
                        )
                    }
                    2u8 => {
                        RuleSet::ProgramDenyList(
                            borsh::BorshDeserialize::deserialize_reader(reader)?,
                        )
                    }
                    _ => {
                        return Err(
                            borsh::maybestd::io::Error::new(
                                borsh::maybestd::io::ErrorKind::InvalidInput,
                                {
                                    let res = ::alloc::fmt::format(
                                        format_args!("Unexpected variant index: {0:?}", variant_idx),
                                    );
                                    res
                                },
                            ),
                        );
                    }
                };
                Ok(return_value)
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for RuleSet {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    RuleSet::None => ::core::fmt::Formatter::write_str(f, "None"),
                    RuleSet::ProgramAllowList(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ProgramAllowList",
                            &__self_0,
                        )
                    }
                    RuleSet::ProgramDenyList(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ProgramDenyList",
                            &__self_0,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for RuleSet {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for RuleSet {
            #[inline]
            fn eq(&self, other: &RuleSet) -> bool {
                let __self_tag = ::core::intrinsics::discriminant_value(self);
                let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                __self_tag == __arg1_tag
                    && match (self, other) {
                        (
                            RuleSet::ProgramAllowList(__self_0),
                            RuleSet::ProgramAllowList(__arg1_0),
                        ) => *__self_0 == *__arg1_0,
                        (
                            RuleSet::ProgramDenyList(__self_0),
                            RuleSet::ProgramDenyList(__arg1_0),
                        ) => *__self_0 == *__arg1_0,
                        _ => true,
                    }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for RuleSet {}
        #[automatically_derived]
        impl ::core::cmp::Eq for RuleSet {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Vec<Pubkey>>;
                let _: ::core::cmp::AssertParamIsEq<Vec<Pubkey>>;
            }
        }
        /// Traditional royalties structure for an asset.
        pub struct Royalties {
            /// The percentage of royalties to be paid to the creators.
            basis_points: u16,
            /// A list of creators to receive royalties.
            creators: Vec<Creator>,
            /// The rule set for the asset to enforce royalties.
            rule_set: RuleSet,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Royalties {
            #[inline]
            fn clone(&self) -> Royalties {
                Royalties {
                    basis_points: ::core::clone::Clone::clone(&self.basis_points),
                    creators: ::core::clone::Clone::clone(&self.creators),
                    rule_set: ::core::clone::Clone::clone(&self.rule_set),
                }
            }
        }
        impl borsh::ser::BorshSerialize for Royalties
        where
            u16: borsh::ser::BorshSerialize,
            Vec<Creator>: borsh::ser::BorshSerialize,
            RuleSet: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.basis_points, writer)?;
                borsh::BorshSerialize::serialize(&self.creators, writer)?;
                borsh::BorshSerialize::serialize(&self.rule_set, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for Royalties
        where
            u16: borsh::BorshDeserialize,
            Vec<Creator>: borsh::BorshDeserialize,
            RuleSet: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    basis_points: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    creators: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    rule_set: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Royalties {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "Royalties",
                    "basis_points",
                    &self.basis_points,
                    "creators",
                    &self.creators,
                    "rule_set",
                    &&self.rule_set,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for Royalties {}
        #[automatically_derived]
        impl ::core::cmp::Eq for Royalties {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<u16>;
                let _: ::core::cmp::AssertParamIsEq<Vec<Creator>>;
                let _: ::core::cmp::AssertParamIsEq<RuleSet>;
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Royalties {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Royalties {
            #[inline]
            fn eq(&self, other: &Royalties) -> bool {
                self.basis_points == other.basis_points
                    && self.creators == other.creators && self.rule_set == other.rule_set
            }
        }
        fn validate_royalties(
            royalties: &Royalties,
        ) -> Result<super::ValidationResult, ProgramError> {
            if royalties.basis_points > 10000 {
                return Err(ProgramError::InvalidArgument);
            }
            if royalties
                .creators
                .iter()
                .fold(0u8, |acc, creator| acc.saturating_add(creator.percentage)) != 100
            {
                return Err(ProgramError::InvalidArgument);
            }
            let mut seen_addresses = HashSet::new();
            if !royalties
                .creators
                .iter()
                .all(|creator| seen_addresses.insert(creator.address))
            {
                return Err(ProgramError::InvalidArgument);
            }
            Ok(ValidationResult::Pass)
        }
        impl PluginValidation for Royalties {
            fn validate_create(
                &self,
                _authority_info: &AccountInfo,
                _authority: &Authority,
            ) -> Result<super::ValidationResult, ProgramError> {
                validate_royalties(self)
            }
            fn validate_transfer(
                &self,
                authority_info: &AccountInfo,
                new_owner: &AccountInfo,
                _authority: &Authority,
                _resolved_authority: Option<&Authority>,
            ) -> Result<ValidationResult, ProgramError> {
                match &self.rule_set {
                    RuleSet::None => Ok(ValidationResult::Pass),
                    RuleSet::ProgramAllowList(allow_list) => {
                        ::solana_program::log::sol_log("Evaluating royalties");
                        if allow_list.contains(authority_info.owner)
                            || allow_list.contains(new_owner.owner)
                        {
                            Ok(ValidationResult::Pass)
                        } else {
                            Ok(ValidationResult::Rejected)
                        }
                    }
                    RuleSet::ProgramDenyList(deny_list) => {
                        if deny_list.contains(authority_info.owner)
                            || deny_list.contains(new_owner.owner)
                        {
                            Ok(ValidationResult::Rejected)
                        } else {
                            Ok(ValidationResult::Pass)
                        }
                    }
                }
            }
            fn validate_add_plugin(
                &self,
                _authority: &AccountInfo,
                _authorities: &Authority,
                _new_plugin: Option<&super::Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                validate_royalties(self)
            }
            fn validate_update_plugin<T: crate::state::CoreAsset>(
                &self,
                _core_asset: &T,
                _authority_info: &AccountInfo,
                _authority: &Authority,
            ) -> Result<ValidationResult, ProgramError> {
                validate_royalties(self)
            }
            /// Validate the revoke plugin authority lifecycle action.
            fn validate_revoke_plugin_authority(
                &self,
                authority_info: &AccountInfo,
                authority: &Authority,
                plugin_to_revoke: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                ::solana_program::log::sol_log(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("authority_info: {0:?}", authority_info.key),
                        );
                        res
                    },
                );
                ::solana_program::log::sol_log(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("authority: {0:?}", authority),
                        );
                        res
                    },
                );
                if authority
                    == &(Authority::Pubkey {
                        address: *authority_info.key,
                    }) && plugin_to_revoke.is_some()
                    && PluginType::from(plugin_to_revoke.unwrap())
                        == PluginType::Royalties
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
        }
    }
    mod transfer {
        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::{account_info::AccountInfo, program_error::ProgramError};
        use crate::{plugins::PluginType, state::{Authority, DataBlob}};
        use super::{Plugin, PluginValidation, ValidationResult};
        /// This plugin manages the ability to transfer an asset and any authorities
        /// approved are permitted to transfer the asset on behalf of the owner.
        #[repr(C)]
        pub struct Transfer {}
        #[automatically_derived]
        impl ::core::clone::Clone for Transfer {
            #[inline]
            fn clone(&self) -> Transfer {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Transfer {}
        impl borsh::ser::BorshSerialize for Transfer {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for Transfer {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {})
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Transfer {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "Transfer")
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Transfer {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Transfer {
            #[inline]
            fn eq(&self, other: &Transfer) -> bool {
                true
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for Transfer {}
        #[automatically_derived]
        impl ::core::cmp::Eq for Transfer {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        impl Transfer {
            /// Initialize the Transfer plugin.
            pub fn new() -> Self {
                Self {}
            }
        }
        impl Default for Transfer {
            fn default() -> Self {
                Self::new()
            }
        }
        impl DataBlob for Transfer {
            fn get_initial_size() -> usize {
                0
            }
            fn get_size(&self) -> usize {
                0
            }
        }
        impl PluginValidation for Transfer {
            fn validate_burn(
                &self,
                authority_info: &AccountInfo,
                authority: &Authority,
                _resolved_authority: Option<&Authority>,
            ) -> Result<super::ValidationResult, ProgramError> {
                if authority
                    == (&Authority::Pubkey {
                        address: *authority_info.key,
                    })
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            fn validate_transfer(
                &self,
                authority_info: &AccountInfo,
                _new_owner: &AccountInfo,
                authority: &Authority,
                _resolved_authority: Option<&Authority>,
            ) -> Result<super::ValidationResult, ProgramError> {
                if authority
                    == (&Authority::Pubkey {
                        address: *authority_info.key,
                    })
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the revoke plugin authority lifecycle action.
            fn validate_revoke_plugin_authority(
                &self,
                authority_info: &AccountInfo,
                authority: &Authority,
                plugin_to_revoke: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                ::solana_program::log::sol_log(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("authority_info: {0:?}", authority_info.key),
                        );
                        res
                    },
                );
                ::solana_program::log::sol_log(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("authority: {0:?}", authority),
                        );
                        res
                    },
                );
                if authority
                    == &(Authority::Pubkey {
                        address: *authority_info.key,
                    }) && plugin_to_revoke.is_some()
                    && PluginType::from(plugin_to_revoke.unwrap())
                        == PluginType::Transfer
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
        }
    }
    mod update_delegate {
        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::{account_info::AccountInfo, program_error::ProgramError};
        use crate::{plugins::PluginType, state::{Authority, DataBlob}};
        use super::{Plugin, PluginValidation, ValidationResult};
        /// This plugin manages additional permissions to burn.
        /// Any authorities approved are given permission to burn the asset on behalf of the owner.
        #[repr(C)]
        pub struct UpdateDelegate {}
        #[automatically_derived]
        impl ::core::clone::Clone for UpdateDelegate {
            #[inline]
            fn clone(&self) -> UpdateDelegate {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for UpdateDelegate {}
        impl borsh::ser::BorshSerialize for UpdateDelegate {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for UpdateDelegate {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {})
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for UpdateDelegate {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "UpdateDelegate")
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for UpdateDelegate {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for UpdateDelegate {
            #[inline]
            fn eq(&self, other: &UpdateDelegate) -> bool {
                true
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for UpdateDelegate {}
        #[automatically_derived]
        impl ::core::cmp::Eq for UpdateDelegate {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        impl UpdateDelegate {
            /// Initialize the UpdateDelegate plugin.
            pub fn new() -> Self {
                Self {}
            }
        }
        impl Default for UpdateDelegate {
            fn default() -> Self {
                Self::new()
            }
        }
        impl DataBlob for UpdateDelegate {
            fn get_initial_size() -> usize {
                0
            }
            fn get_size(&self) -> usize {
                0
            }
        }
        impl PluginValidation for UpdateDelegate {
            fn validate_add_plugin(
                &self,
                authority_info: &AccountInfo,
                authority: &Authority,
                _new_plugin: Option<&super::Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if authority
                    == (&Authority::Pubkey {
                        address: *authority_info.key,
                    })
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            fn validate_remove_plugin(
                &self,
                authority_info: &AccountInfo,
                authority: &Authority,
                _plugin_to_remove: Option<&super::Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if authority
                    == (&Authority::Pubkey {
                        address: *authority_info.key,
                    })
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the revoke plugin authority lifecycle action.
            fn validate_revoke_plugin_authority(
                &self,
                authority_info: &AccountInfo,
                authority: &Authority,
                plugin_to_revoke: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                ::solana_program::log::sol_log(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("authority_info: {0:?}", authority_info.key),
                        );
                        res
                    },
                );
                ::solana_program::log::sol_log(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("authority: {0:?}", authority),
                        );
                        res
                    },
                );
                if authority
                    == &(Authority::Pubkey {
                        address: *authority_info.key,
                    }) && plugin_to_revoke.is_some()
                    && PluginType::from(plugin_to_revoke.unwrap())
                        == PluginType::UpdateDelegate
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
        }
    }
    mod utils {
        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::{
            account_info::AccountInfo, entrypoint::ProgramResult,
            program_error::ProgramError, program_memory::sol_memcpy,
        };
        use crate::{
            error::MplCoreError,
            state::{Asset, Authority, CoreAsset, DataBlob, Key, SolanaAccount},
            utils::resize_or_reallocate_account,
        };
        use super::{Plugin, PluginHeader, PluginRegistry, PluginType, RegistryRecord};
        /// Create plugin header and registry if it doesn't exist
        pub fn create_meta_idempotent<'a, T: SolanaAccount + DataBlob>(
            account: &AccountInfo<'a>,
            payer: &AccountInfo<'a>,
            system_program: &AccountInfo<'a>,
        ) -> Result<(T, PluginHeader, PluginRegistry), ProgramError> {
            let core = T::load(account, 0)?;
            let header_offset = core.get_size();
            if header_offset == account.data_len() {
                let header = PluginHeader {
                    key: Key::PluginHeader,
                    plugin_registry_offset: header_offset
                        + PluginHeader::get_initial_size(),
                };
                let registry = PluginRegistry {
                    key: Key::PluginRegistry,
                    registry: ::alloc::vec::Vec::new(),
                    external_plugins: ::alloc::vec::Vec::new(),
                };
                resize_or_reallocate_account(
                    account,
                    payer,
                    system_program,
                    header.plugin_registry_offset + PluginRegistry::get_initial_size(),
                )?;
                header.save(account, header_offset)?;
                registry.save(account, header.plugin_registry_offset)?;
                Ok((core, header, registry))
            } else {
                let header = PluginHeader::load(account, header_offset)?;
                let registry = PluginRegistry::load(
                    account,
                    header.plugin_registry_offset,
                )?;
                Ok((core, header, registry))
            }
        }
        /// Create plugin header and registry if it doesn't exist
        pub fn create_plugin_meta<'a, T: SolanaAccount + DataBlob>(
            asset: T,
            account: &AccountInfo<'a>,
            payer: &AccountInfo<'a>,
            system_program: &AccountInfo<'a>,
        ) -> Result<(PluginHeader, PluginRegistry), ProgramError> {
            let header_offset = asset.get_size();
            let header = PluginHeader {
                key: Key::PluginHeader,
                plugin_registry_offset: header_offset + PluginHeader::get_initial_size(),
            };
            let registry = PluginRegistry {
                key: Key::PluginRegistry,
                registry: ::alloc::vec::Vec::new(),
                external_plugins: ::alloc::vec::Vec::new(),
            };
            resize_or_reallocate_account(
                account,
                payer,
                system_program,
                header.plugin_registry_offset + PluginRegistry::get_initial_size(),
            )?;
            header.save(account, header_offset)?;
            registry.save(account, header.plugin_registry_offset)?;
            Ok((header, registry))
        }
        /// Assert that the Plugin metadata has been initialized.
        pub fn assert_plugins_initialized(account: &AccountInfo) -> ProgramResult {
            let mut bytes: &[u8] = &(*account.data).borrow();
            let asset = Asset::deserialize(&mut bytes)?;
            if asset.get_size() == account.data_len() {
                return Err(MplCoreError::PluginsNotInitialized.into());
            }
            Ok(())
        }
        /// Fetch the plugin from the registry.
        pub fn fetch_plugin<T: DataBlob + SolanaAccount, U: BorshDeserialize>(
            account: &AccountInfo,
            plugin_type: PluginType,
        ) -> Result<(Authority, U, usize), ProgramError> {
            let asset = T::load(account, 0)?;
            let header = PluginHeader::load(account, asset.get_size())?;
            let PluginRegistry { registry, .. } = PluginRegistry::load(
                account,
                header.plugin_registry_offset,
            )?;
            let registry_record = registry
                .iter()
                .find(|record| record.plugin_type == plugin_type)
                .ok_or(MplCoreError::PluginNotFound)?;
            let plugin = Plugin::deserialize(
                &mut &(*account.data).borrow()[registry_record.offset..],
            )?;
            if PluginType::from(&plugin) != plugin_type {
                return Err(MplCoreError::PluginNotFound.into());
            }
            let inner = U::deserialize(
                &mut &(*account.data)
                    .borrow()[registry_record
                    .offset
                    .checked_add(1)
                    .ok_or(MplCoreError::NumericalOverflow)?..],
            )?;
            Ok((registry_record.authority, inner, registry_record.offset))
        }
        /// Fetch the plugin from the registry.
        pub fn fetch_wrapped_plugin<T: DataBlob + SolanaAccount>(
            account: &AccountInfo,
            plugin_type: PluginType,
        ) -> Result<(Authority, Plugin), ProgramError> {
            let asset = T::load(account, 0)?;
            let header = PluginHeader::load(account, asset.get_size())?;
            let PluginRegistry { registry, .. } = PluginRegistry::load(
                account,
                header.plugin_registry_offset,
            )?;
            let registry_record = registry
                .iter()
                .find(|record| record.plugin_type == plugin_type)
                .ok_or(MplCoreError::PluginNotFound)?;
            let plugin = Plugin::deserialize(
                &mut &(*account.data).borrow()[registry_record.offset..],
            )?;
            Ok((registry_record.authority, plugin))
        }
        /// Fetch the plugin registry.
        pub fn fetch_plugins(
            account: &AccountInfo,
        ) -> Result<Vec<RegistryRecord>, ProgramError> {
            let asset = Asset::load(account, 0)?;
            let header = PluginHeader::load(account, asset.get_size())?;
            let PluginRegistry { registry, .. } = PluginRegistry::load(
                account,
                header.plugin_registry_offset,
            )?;
            Ok(registry)
        }
        /// Create plugin header and registry if it doesn't exist
        pub fn list_plugins(
            account: &AccountInfo,
        ) -> Result<Vec<PluginType>, ProgramError> {
            let asset = Asset::load(account, 0)?;
            let header = PluginHeader::load(account, asset.get_size())?;
            let PluginRegistry { registry, .. } = PluginRegistry::load(
                account,
                header.plugin_registry_offset,
            )?;
            Ok(
                registry
                    .iter()
                    .map(|registry_record| registry_record.plugin_type)
                    .collect(),
            )
        }
        /// Add a plugin to the registry and initialize it.
        pub fn initialize_plugin<'a, T: DataBlob + SolanaAccount>(
            plugin: &Plugin,
            authority: &Authority,
            account: &AccountInfo<'a>,
            payer: &AccountInfo<'a>,
            system_program: &AccountInfo<'a>,
        ) -> ProgramResult {
            let core = T::load(account, 0)?;
            let header_offset = core.get_size();
            let mut header = PluginHeader::load(account, header_offset)?;
            let mut plugin_registry = PluginRegistry::load(
                account,
                header.plugin_registry_offset,
            )?;
            let plugin_type = plugin.into();
            let plugin_data = plugin.try_to_vec()?;
            let plugin_size = plugin_data.len();
            if plugin_registry
                .registry
                .iter_mut()
                .any(|record| record.plugin_type == plugin_type)
            {
                return Err(MplCoreError::PluginAlreadyExists.into());
            }
            let old_registry_offset = header.plugin_registry_offset;
            let new_registry_record = RegistryRecord {
                plugin_type,
                offset: old_registry_offset,
                authority: *authority,
            };
            let size_increase = plugin_size
                .checked_add(new_registry_record.try_to_vec()?.len())
                .ok_or(MplCoreError::NumericalOverflow)?;
            let new_registry_offset = header
                .plugin_registry_offset
                .checked_add(plugin_size)
                .ok_or(MplCoreError::NumericalOverflow)?;
            header.plugin_registry_offset = new_registry_offset;
            plugin_registry.registry.push(new_registry_record);
            let new_size = account
                .data_len()
                .checked_add(size_increase)
                .ok_or(MplCoreError::NumericalOverflow)?;
            resize_or_reallocate_account(account, payer, system_program, new_size)?;
            header.save(account, header_offset)?;
            plugin.save(account, old_registry_offset)?;
            plugin_registry.save(account, new_registry_offset)?;
            Ok(())
        }
        /// Remove a plugin from the registry and delete it.
        pub fn delete_plugin<'a, T: DataBlob>(
            plugin_type: &PluginType,
            asset: &T,
            account: &AccountInfo<'a>,
            payer: &AccountInfo<'a>,
            system_program: &AccountInfo<'a>,
        ) -> ProgramResult {
            let mut header = PluginHeader::load(account, asset.get_size())?;
            let mut plugin_registry = PluginRegistry::load(
                account,
                header.plugin_registry_offset,
            )?;
            if let Some(index) = plugin_registry
                .registry
                .iter_mut()
                .position(|record| record.plugin_type == *plugin_type)
            {
                let registry_record = plugin_registry.registry.remove(index);
                let serialized_registry_record = registry_record.try_to_vec()?;
                let plugin_offset = registry_record.offset;
                let plugin = Plugin::load(account, plugin_offset)?;
                let serialized_plugin = plugin.try_to_vec()?;
                let next_plugin_offset = plugin_offset
                    .checked_add(serialized_plugin.len())
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let new_size = account
                    .data_len()
                    .checked_sub(serialized_registry_record.len())
                    .ok_or(MplCoreError::NumericalOverflow)?
                    .checked_sub(serialized_plugin.len())
                    .ok_or(MplCoreError::NumericalOverflow)?;
                ::solana_program::log::sol_log(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("size: {0:?}", account.data_len()),
                        );
                        res
                    },
                );
                ::solana_program::log::sol_log(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!("new_size: {0:?}", new_size),
                        );
                        res
                    },
                );
                let new_registry_offset = header
                    .plugin_registry_offset
                    .checked_sub(serialized_plugin.len())
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let data_to_move = header
                    .plugin_registry_offset
                    .checked_sub(new_registry_offset)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let src = account.data.borrow()[next_plugin_offset..].to_vec();
                sol_memcpy(
                    &mut account.data.borrow_mut()[plugin_offset..],
                    &src,
                    data_to_move,
                );
                header.plugin_registry_offset = new_registry_offset;
                header.save(account, asset.get_size())?;
                plugin_registry.save(account, new_registry_offset)?;
                resize_or_reallocate_account(account, payer, system_program, new_size)?;
            } else {
                return Err(MplCoreError::PluginNotFound.into());
            }
            Ok(())
        }
        /// Add an authority to a plugin.
        #[allow(clippy::too_many_arguments)]
        pub fn approve_authority_on_plugin<'a, T: CoreAsset>(
            plugin_type: &PluginType,
            new_authority: &Authority,
            account: &AccountInfo<'a>,
            plugin_header: &PluginHeader,
            plugin_registry: &mut PluginRegistry,
            payer: &AccountInfo<'a>,
            system_program: &AccountInfo<'a>,
        ) -> ProgramResult {
            let registry_record = &mut plugin_registry
                .registry
                .iter_mut()
                .find(|record| record.plugin_type == *plugin_type)
                .ok_or(MplCoreError::PluginNotFound)?;
            registry_record.authority = *new_authority;
            let authority_bytes = new_authority.try_to_vec()?;
            let new_size = account
                .data_len()
                .checked_add(authority_bytes.len())
                .ok_or(MplCoreError::NumericalOverflow)?;
            resize_or_reallocate_account(account, payer, system_program, new_size)?;
            plugin_registry.save(account, plugin_header.plugin_registry_offset)?;
            Ok(())
        }
        /// Remove an authority from a plugin.
        #[allow(clippy::too_many_arguments)]
        pub fn revoke_authority_on_plugin<'a>(
            plugin_type: &PluginType,
            account: &AccountInfo<'a>,
            plugin_header: &PluginHeader,
            plugin_registry: &mut PluginRegistry,
            payer: &AccountInfo<'a>,
            system_program: &AccountInfo<'a>,
        ) -> ProgramResult {
            let registry_record = &mut plugin_registry
                .registry
                .iter_mut()
                .find(|record| record.plugin_type == *plugin_type)
                .ok_or(MplCoreError::PluginNotFound)?;
            let old_authority_bytes = registry_record.authority.try_to_vec()?;
            registry_record.authority = registry_record.plugin_type.manager();
            let new_authority_bytes = registry_record.authority.try_to_vec()?;
            let size_diff = (new_authority_bytes.len() as isize)
                .checked_sub(old_authority_bytes.len() as isize)
                .ok_or(MplCoreError::NumericalOverflow)?;
            let new_size = (account.data_len() as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?;
            resize_or_reallocate_account(
                account,
                payer,
                system_program,
                new_size as usize,
            )?;
            plugin_registry.save(account, plugin_header.plugin_registry_offset)?;
            Ok(())
        }
    }
    pub use attributes::*;
    pub use burn::*;
    use core_macros::plugin_validator;
    pub use freeze::*;
    pub use lifecycle::*;
    use num_derive::ToPrimitive;
    pub use permanent_burn::*;
    pub use permanent_freeze::*;
    pub use permanent_transfer::*;
    pub use plugin_header::*;
    pub use plugin_registry::*;
    pub use royalties::*;
    pub use transfer::*;
    pub use update_delegate::*;
    pub use utils::*;
    use solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, msg,
        program_error::ProgramError,
    };
    use borsh::{BorshDeserialize, BorshSerialize};
    use strum::EnumCount;
    use crate::{error::MplCoreError, state::{Authority, Compressible, DataBlob}};
    /// Definition of the plugin variants, each containing a link to the plugin struct.
    #[repr(u16)]
    pub enum Plugin {
        /// Royalties plugin.
        Royalties(Royalties),
        /// Freeze plugin.
        Freeze(Freeze),
        /// Burn plugin.
        Burn(Burn),
        /// Transfer plugin.
        Transfer(Transfer),
        /// Update Delegate plugin.
        UpdateDelegate(UpdateDelegate),
        /// Permanent Freeze authority which allows the creator to freeze
        PermanentFreeze(PermanentFreeze),
        /// Attributes plugin for arbitrary Key-Value pairs.
        Attributes(Attributes),
        /// Permanent Transfer authority which allows the creator of an asset to become the person who can transfer an Asset
        PermanentTransfer(PermanentTransfer),
        /// Permanent Burn authority allows the creator of an asset to become the person who can burn an Asset
        PermanentBurn(PermanentBurn),
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Plugin {
        #[inline]
        fn clone(&self) -> Plugin {
            match self {
                Plugin::Royalties(__self_0) => {
                    Plugin::Royalties(::core::clone::Clone::clone(__self_0))
                }
                Plugin::Freeze(__self_0) => {
                    Plugin::Freeze(::core::clone::Clone::clone(__self_0))
                }
                Plugin::Burn(__self_0) => {
                    Plugin::Burn(::core::clone::Clone::clone(__self_0))
                }
                Plugin::Transfer(__self_0) => {
                    Plugin::Transfer(::core::clone::Clone::clone(__self_0))
                }
                Plugin::UpdateDelegate(__self_0) => {
                    Plugin::UpdateDelegate(::core::clone::Clone::clone(__self_0))
                }
                Plugin::PermanentFreeze(__self_0) => {
                    Plugin::PermanentFreeze(::core::clone::Clone::clone(__self_0))
                }
                Plugin::Attributes(__self_0) => {
                    Plugin::Attributes(::core::clone::Clone::clone(__self_0))
                }
                Plugin::PermanentTransfer(__self_0) => {
                    Plugin::PermanentTransfer(::core::clone::Clone::clone(__self_0))
                }
                Plugin::PermanentBurn(__self_0) => {
                    Plugin::PermanentBurn(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Plugin {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Plugin::Royalties(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Royalties",
                        &__self_0,
                    )
                }
                Plugin::Freeze(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Freeze",
                        &__self_0,
                    )
                }
                Plugin::Burn(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Burn",
                        &__self_0,
                    )
                }
                Plugin::Transfer(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Transfer",
                        &__self_0,
                    )
                }
                Plugin::UpdateDelegate(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "UpdateDelegate",
                        &__self_0,
                    )
                }
                Plugin::PermanentFreeze(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "PermanentFreeze",
                        &__self_0,
                    )
                }
                Plugin::Attributes(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Attributes",
                        &__self_0,
                    )
                }
                Plugin::PermanentTransfer(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "PermanentTransfer",
                        &__self_0,
                    )
                }
                Plugin::PermanentBurn(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "PermanentBurn",
                        &__self_0,
                    )
                }
            }
        }
    }
    impl borsh::ser::BorshSerialize for Plugin
    where
        Royalties: borsh::ser::BorshSerialize,
        Freeze: borsh::ser::BorshSerialize,
        Burn: borsh::ser::BorshSerialize,
        Transfer: borsh::ser::BorshSerialize,
        UpdateDelegate: borsh::ser::BorshSerialize,
        PermanentFreeze: borsh::ser::BorshSerialize,
        Attributes: borsh::ser::BorshSerialize,
        PermanentTransfer: borsh::ser::BorshSerialize,
        PermanentBurn: borsh::ser::BorshSerialize,
    {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            let variant_idx: u8 = match self {
                Plugin::Royalties(..) => 0u8,
                Plugin::Freeze(..) => 1u8,
                Plugin::Burn(..) => 2u8,
                Plugin::Transfer(..) => 3u8,
                Plugin::UpdateDelegate(..) => 4u8,
                Plugin::PermanentFreeze(..) => 5u8,
                Plugin::Attributes(..) => 6u8,
                Plugin::PermanentTransfer(..) => 7u8,
                Plugin::PermanentBurn(..) => 8u8,
            };
            writer.write_all(&variant_idx.to_le_bytes())?;
            match self {
                Plugin::Royalties(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                Plugin::Freeze(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                Plugin::Burn(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                Plugin::Transfer(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                Plugin::UpdateDelegate(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                Plugin::PermanentFreeze(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                Plugin::Attributes(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                Plugin::PermanentTransfer(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
                Plugin::PermanentBurn(id0) => {
                    borsh::BorshSerialize::serialize(id0, writer)?;
                }
            }
            Ok(())
        }
    }
    impl borsh::de::BorshDeserialize for Plugin
    where
        Royalties: borsh::BorshDeserialize,
        Freeze: borsh::BorshDeserialize,
        Burn: borsh::BorshDeserialize,
        Transfer: borsh::BorshDeserialize,
        UpdateDelegate: borsh::BorshDeserialize,
        PermanentFreeze: borsh::BorshDeserialize,
        Attributes: borsh::BorshDeserialize,
        PermanentTransfer: borsh::BorshDeserialize,
        PermanentBurn: borsh::BorshDeserialize,
    {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let tag = <u8 as borsh::de::BorshDeserialize>::deserialize_reader(reader)?;
            <Self as borsh::de::EnumExt>::deserialize_variant(reader, tag)
        }
    }
    impl borsh::de::EnumExt for Plugin
    where
        Royalties: borsh::BorshDeserialize,
        Freeze: borsh::BorshDeserialize,
        Burn: borsh::BorshDeserialize,
        Transfer: borsh::BorshDeserialize,
        UpdateDelegate: borsh::BorshDeserialize,
        PermanentFreeze: borsh::BorshDeserialize,
        Attributes: borsh::BorshDeserialize,
        PermanentTransfer: borsh::BorshDeserialize,
        PermanentBurn: borsh::BorshDeserialize,
    {
        fn deserialize_variant<R: borsh::maybestd::io::Read>(
            reader: &mut R,
            variant_idx: u8,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let mut return_value = match variant_idx {
                0u8 => {
                    Plugin::Royalties(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                1u8 => {
                    Plugin::Freeze(borsh::BorshDeserialize::deserialize_reader(reader)?)
                }
                2u8 => Plugin::Burn(borsh::BorshDeserialize::deserialize_reader(reader)?),
                3u8 => {
                    Plugin::Transfer(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                4u8 => {
                    Plugin::UpdateDelegate(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                5u8 => {
                    Plugin::PermanentFreeze(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                6u8 => {
                    Plugin::Attributes(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                7u8 => {
                    Plugin::PermanentTransfer(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                8u8 => {
                    Plugin::PermanentBurn(
                        borsh::BorshDeserialize::deserialize_reader(reader)?,
                    )
                }
                _ => {
                    return Err(
                        borsh::maybestd::io::Error::new(
                            borsh::maybestd::io::ErrorKind::InvalidInput,
                            {
                                let res = ::alloc::fmt::format(
                                    format_args!("Unexpected variant index: {0:?}", variant_idx),
                                );
                                res
                            },
                        ),
                    );
                }
            };
            Ok(return_value)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Plugin {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Plugin {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Royalties>;
            let _: ::core::cmp::AssertParamIsEq<Freeze>;
            let _: ::core::cmp::AssertParamIsEq<Burn>;
            let _: ::core::cmp::AssertParamIsEq<Transfer>;
            let _: ::core::cmp::AssertParamIsEq<UpdateDelegate>;
            let _: ::core::cmp::AssertParamIsEq<PermanentFreeze>;
            let _: ::core::cmp::AssertParamIsEq<Attributes>;
            let _: ::core::cmp::AssertParamIsEq<PermanentTransfer>;
            let _: ::core::cmp::AssertParamIsEq<PermanentBurn>;
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Plugin {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Plugin {
        #[inline]
        fn eq(&self, other: &Plugin) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (Plugin::Royalties(__self_0), Plugin::Royalties(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (Plugin::Freeze(__self_0), Plugin::Freeze(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (Plugin::Burn(__self_0), Plugin::Burn(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (Plugin::Transfer(__self_0), Plugin::Transfer(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (
                        Plugin::UpdateDelegate(__self_0),
                        Plugin::UpdateDelegate(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    (
                        Plugin::PermanentFreeze(__self_0),
                        Plugin::PermanentFreeze(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    (Plugin::Attributes(__self_0), Plugin::Attributes(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (
                        Plugin::PermanentTransfer(__self_0),
                        Plugin::PermanentTransfer(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    (
                        Plugin::PermanentBurn(__self_0),
                        Plugin::PermanentBurn(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
        }
    }
    impl Plugin {
        /// Get the default authority for a plugin which defines who must allow the plugin to be created.
        pub fn manager(&self) -> Authority {
            PluginType::from(self).manager()
        }
        /// Load and deserialize a plugin from an offset in the account.
        pub fn load(account: &AccountInfo, offset: usize) -> Result<Self, ProgramError> {
            let mut bytes: &[u8] = &(*account.data).borrow()[offset..];
            Self::deserialize(&mut bytes)
                .map_err(|error| {
                    ::solana_program::log::sol_log(
                        &{
                            let res = ::alloc::fmt::format(
                                format_args!("Error: {0}", error),
                            );
                            res
                        },
                    );
                    MplCoreError::DeserializationError.into()
                })
        }
        /// Save and serialize a plugin to an offset in the account.
        pub fn save(&self, account: &AccountInfo, offset: usize) -> ProgramResult {
            borsh::to_writer(&mut account.data.borrow_mut()[offset..], self)
                .map_err(|error| {
                    ::solana_program::log::sol_log(
                        &{
                            let res = ::alloc::fmt::format(
                                format_args!("Error: {0}", error),
                            );
                            res
                        },
                    );
                    MplCoreError::SerializationError.into()
                })
        }
    }
    impl Compressible for Plugin {}
    /// List of First Party Plugin types.
    #[repr(u16)]
    pub enum PluginType {
        /// Royalties plugin.
        Royalties,
        /// Freeze plugin.
        Freeze,
        /// Burn plugin.
        Burn,
        /// Transfer plugin.
        Transfer,
        /// Update Delegate plugin.
        UpdateDelegate,
        /// The Permanent Freeze plugin.
        PermanentFreeze,
        /// The Attributes plugin.
        Attributes,
        /// The Permanent Transfer plugin.
        PermanentTransfer,
        /// The Permanent Burn plugin.
        PermanentBurn,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for PluginType {
        #[inline]
        fn clone(&self) -> PluginType {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for PluginType {}
    #[automatically_derived]
    impl ::core::fmt::Debug for PluginType {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    PluginType::Royalties => "Royalties",
                    PluginType::Freeze => "Freeze",
                    PluginType::Burn => "Burn",
                    PluginType::Transfer => "Transfer",
                    PluginType::UpdateDelegate => "UpdateDelegate",
                    PluginType::PermanentFreeze => "PermanentFreeze",
                    PluginType::Attributes => "Attributes",
                    PluginType::PermanentTransfer => "PermanentTransfer",
                    PluginType::PermanentBurn => "PermanentBurn",
                },
            )
        }
    }
    impl borsh::ser::BorshSerialize for PluginType {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            let variant_idx: u8 = match self {
                PluginType::Royalties => 0u8,
                PluginType::Freeze => 1u8,
                PluginType::Burn => 2u8,
                PluginType::Transfer => 3u8,
                PluginType::UpdateDelegate => 4u8,
                PluginType::PermanentFreeze => 5u8,
                PluginType::Attributes => 6u8,
                PluginType::PermanentTransfer => 7u8,
                PluginType::PermanentBurn => 8u8,
            };
            writer.write_all(&variant_idx.to_le_bytes())?;
            match self {
                PluginType::Royalties => {}
                PluginType::Freeze => {}
                PluginType::Burn => {}
                PluginType::Transfer => {}
                PluginType::UpdateDelegate => {}
                PluginType::PermanentFreeze => {}
                PluginType::Attributes => {}
                PluginType::PermanentTransfer => {}
                PluginType::PermanentBurn => {}
            }
            Ok(())
        }
    }
    impl borsh::de::BorshDeserialize for PluginType {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let tag = <u8 as borsh::de::BorshDeserialize>::deserialize_reader(reader)?;
            <Self as borsh::de::EnumExt>::deserialize_variant(reader, tag)
        }
    }
    impl borsh::de::EnumExt for PluginType {
        fn deserialize_variant<R: borsh::maybestd::io::Read>(
            reader: &mut R,
            variant_idx: u8,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let mut return_value = match variant_idx {
                0u8 => PluginType::Royalties,
                1u8 => PluginType::Freeze,
                2u8 => PluginType::Burn,
                3u8 => PluginType::Transfer,
                4u8 => PluginType::UpdateDelegate,
                5u8 => PluginType::PermanentFreeze,
                6u8 => PluginType::Attributes,
                7u8 => PluginType::PermanentTransfer,
                8u8 => PluginType::PermanentBurn,
                _ => {
                    return Err(
                        borsh::maybestd::io::Error::new(
                            borsh::maybestd::io::ErrorKind::InvalidInput,
                            {
                                let res = ::alloc::fmt::format(
                                    format_args!("Unexpected variant index: {0:?}", variant_idx),
                                );
                                res
                            },
                        ),
                    );
                }
            };
            Ok(return_value)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for PluginType {}
    #[automatically_derived]
    impl ::core::cmp::Eq for PluginType {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for PluginType {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for PluginType {
        #[inline]
        fn eq(&self, other: &PluginType) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[allow(non_upper_case_globals, unused_qualifications)]
    const _IMPL_NUM_ToPrimitive_FOR_PluginType: () = {
        #[allow(clippy::useless_attribute)]
        #[allow(rust_2018_idioms)]
        extern crate num_traits as _num_traits;
        impl _num_traits::ToPrimitive for PluginType {
            #[inline]
            #[allow(trivial_numeric_casts)]
            fn to_i64(&self) -> Option<i64> {
                Some(
                    match *self {
                        PluginType::Royalties => PluginType::Royalties as i64,
                        PluginType::Freeze => PluginType::Freeze as i64,
                        PluginType::Burn => PluginType::Burn as i64,
                        PluginType::Transfer => PluginType::Transfer as i64,
                        PluginType::UpdateDelegate => PluginType::UpdateDelegate as i64,
                        PluginType::PermanentFreeze => PluginType::PermanentFreeze as i64,
                        PluginType::Attributes => PluginType::Attributes as i64,
                        PluginType::PermanentTransfer => {
                            PluginType::PermanentTransfer as i64
                        }
                        PluginType::PermanentBurn => PluginType::PermanentBurn as i64,
                    },
                )
            }
            #[inline]
            fn to_u64(&self) -> Option<u64> {
                self.to_i64().map(|x| x as u64)
            }
        }
    };
    impl ::strum::EnumCount for PluginType {
        const COUNT: usize = 9usize;
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for PluginType {
        #[inline]
        fn partial_cmp(
            &self,
            other: &PluginType,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_tag, &__arg1_tag)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for PluginType {
        #[inline]
        fn cmp(&self, other: &PluginType) -> ::core::cmp::Ordering {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_tag, &__arg1_tag)
        }
    }
    impl DataBlob for PluginType {
        fn get_initial_size() -> usize {
            2
        }
        fn get_size(&self) -> usize {
            2
        }
    }
    impl From<&Plugin> for PluginType {
        fn from(plugin: &Plugin) -> Self {
            match plugin {
                Plugin::Royalties(_) => PluginType::Royalties,
                Plugin::Freeze(_) => PluginType::Freeze,
                Plugin::Burn(_) => PluginType::Burn,
                Plugin::Transfer(_) => PluginType::Transfer,
                Plugin::UpdateDelegate(_) => PluginType::UpdateDelegate,
                Plugin::PermanentFreeze(_) => PluginType::PermanentFreeze,
                Plugin::Attributes(_) => PluginType::Attributes,
                Plugin::PermanentTransfer(_) => PluginType::PermanentTransfer,
                Plugin::PermanentBurn(_) => PluginType::PermanentBurn,
            }
        }
    }
    impl PluginType {
        /// Get the default authority for a plugin which defines who must allow the plugin to be created.
        pub fn manager(&self) -> Authority {
            match self {
                PluginType::Royalties => Authority::UpdateAuthority,
                PluginType::Freeze => Authority::Owner,
                PluginType::Burn => Authority::Owner,
                PluginType::Transfer => Authority::Owner,
                PluginType::UpdateDelegate => Authority::UpdateAuthority,
                PluginType::PermanentFreeze => Authority::UpdateAuthority,
                PluginType::Attributes => Authority::UpdateAuthority,
                PluginType::PermanentTransfer => Authority::UpdateAuthority,
                PluginType::PermanentBurn => Authority::UpdateAuthority,
            }
        }
    }
    /// A pair of a plugin type and an optional authority.
    #[repr(C)]
    pub(crate) struct PluginAuthorityPair {
        pub(crate) plugin: Plugin,
        pub(crate) authority: Option<Authority>,
    }
    impl borsh::ser::BorshSerialize for PluginAuthorityPair
    where
        Plugin: borsh::ser::BorshSerialize,
        Option<Authority>: borsh::ser::BorshSerialize,
    {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            borsh::BorshSerialize::serialize(&self.plugin, writer)?;
            borsh::BorshSerialize::serialize(&self.authority, writer)?;
            Ok(())
        }
    }
    impl borsh::de::BorshDeserialize for PluginAuthorityPair
    where
        Plugin: borsh::BorshDeserialize,
        Option<Authority>: borsh::BorshDeserialize,
    {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            Ok(Self {
                plugin: borsh::BorshDeserialize::deserialize_reader(reader)?,
                authority: borsh::BorshDeserialize::deserialize_reader(reader)?,
            })
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for PluginAuthorityPair {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for PluginAuthorityPair {
        #[inline]
        fn eq(&self, other: &PluginAuthorityPair) -> bool {
            self.plugin == other.plugin && self.authority == other.authority
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for PluginAuthorityPair {}
    #[automatically_derived]
    impl ::core::cmp::Eq for PluginAuthorityPair {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Plugin>;
            let _: ::core::cmp::AssertParamIsEq<Option<Authority>>;
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for PluginAuthorityPair {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "PluginAuthorityPair",
                "plugin",
                &self.plugin,
                "authority",
                &&self.authority,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for PluginAuthorityPair {
        #[inline]
        fn clone(&self) -> PluginAuthorityPair {
            PluginAuthorityPair {
                plugin: ::core::clone::Clone::clone(&self.plugin),
                authority: ::core::clone::Clone::clone(&self.authority),
            }
        }
    }
}
/// Module for processing instructions and routing them
/// to the associated processor.
pub mod processor {
    use crate::instruction::MplAssetInstruction;
    use borsh::BorshDeserialize;
    use solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey,
    };
    mod create {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{
            account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke,
            program_memory::sol_memcpy, rent::Rent, system_instruction, system_program,
            sysvar::Sysvar,
        };
        use crate::{
            error::MplCoreError, instruction::accounts::CreateAccounts,
            plugins::{
                create_meta_idempotent, initialize_plugin, CheckResult, Plugin,
                PluginAuthorityPair, PluginType, ValidationResult,
            },
            state::{Asset, DataState, UpdateAuthority, COLLECT_AMOUNT},
            utils::fetch_core_data,
        };
        #[repr(C)]
        pub(crate) struct CreateArgs {
            pub(crate) data_state: DataState,
            pub(crate) name: String,
            pub(crate) uri: String,
            pub(crate) plugins: Option<Vec<PluginAuthorityPair>>,
        }
        impl borsh::ser::BorshSerialize for CreateArgs
        where
            DataState: borsh::ser::BorshSerialize,
            String: borsh::ser::BorshSerialize,
            String: borsh::ser::BorshSerialize,
            Option<Vec<PluginAuthorityPair>>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.data_state, writer)?;
                borsh::BorshSerialize::serialize(&self.name, writer)?;
                borsh::BorshSerialize::serialize(&self.uri, writer)?;
                borsh::BorshSerialize::serialize(&self.plugins, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for CreateArgs
        where
            DataState: borsh::BorshDeserialize,
            String: borsh::BorshDeserialize,
            String: borsh::BorshDeserialize,
            Option<Vec<PluginAuthorityPair>>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    data_state: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    name: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    uri: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    plugins: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for CreateArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for CreateArgs {
            #[inline]
            fn eq(&self, other: &CreateArgs) -> bool {
                self.data_state == other.data_state && self.name == other.name
                    && self.uri == other.uri && self.plugins == other.plugins
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for CreateArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for CreateArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<DataState>;
                let _: ::core::cmp::AssertParamIsEq<String>;
                let _: ::core::cmp::AssertParamIsEq<Option<Vec<PluginAuthorityPair>>>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for CreateArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field4_finish(
                    f,
                    "CreateArgs",
                    "data_state",
                    &self.data_state,
                    "name",
                    &self.name,
                    "uri",
                    &self.uri,
                    "plugins",
                    &&self.plugins,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for CreateArgs {
            #[inline]
            fn clone(&self) -> CreateArgs {
                CreateArgs {
                    data_state: ::core::clone::Clone::clone(&self.data_state),
                    name: ::core::clone::Clone::clone(&self.name),
                    uri: ::core::clone::Clone::clone(&self.uri),
                    plugins: ::core::clone::Clone::clone(&self.plugins),
                }
            }
        }
        pub(crate) fn create<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: CreateArgs,
        ) -> ProgramResult {
            let ctx = CreateAccounts::context(accounts)?;
            let rent = Rent::get()?;
            assert_signer(ctx.accounts.asset)?;
            assert_signer(ctx.accounts.payer)?;
            if *ctx.accounts.system_program.key != system_program::id() {
                return Err(MplCoreError::InvalidSystemProgram.into());
            }
            let update_authority = match ctx.accounts.collection {
                Some(collection) => UpdateAuthority::Collection(*collection.key),
                None => {
                    UpdateAuthority::Address(
                        *ctx.accounts.update_authority.unwrap_or(ctx.accounts.payer).key,
                    )
                }
            };
            if update_authority.validate_create(&ctx.accounts, &args)?
                == ValidationResult::Rejected
            {
                return Err(MplCoreError::InvalidAuthority.into());
            }
            let new_asset = Asset::new(
                *ctx
                    .accounts
                    .owner
                    .unwrap_or(
                        ctx.accounts.update_authority.unwrap_or(ctx.accounts.payer),
                    )
                    .key,
                update_authority,
                args.name.clone(),
                args.uri.clone(),
            );
            let serialized_data = new_asset.try_to_vec()?;
            let serialized_data = match args.data_state {
                DataState::AccountState => serialized_data,
                DataState::LedgerState => {
                    ::solana_program::log::sol_log(
                        "Error: Minting compressed is currently not available",
                    );
                    return Err(MplCoreError::NotAvailable.into());
                }
            };
            let lamports = rent.minimum_balance(serialized_data.len()) + COLLECT_AMOUNT;
            invoke(
                &system_instruction::create_account(
                    ctx.accounts.payer.key,
                    ctx.accounts.asset.key,
                    lamports,
                    serialized_data.len() as u64,
                    &crate::id(),
                ),
                &[
                    ctx.accounts.payer.clone(),
                    ctx.accounts.asset.clone(),
                    ctx.accounts.system_program.clone(),
                ],
            )?;
            sol_memcpy(
                &mut ctx.accounts.asset.try_borrow_mut_data()?,
                &serialized_data,
                serialized_data.len(),
            );
            if args.data_state == DataState::AccountState {
                create_meta_idempotent::<
                    Asset,
                >(ctx.accounts.asset, ctx.accounts.payer, ctx.accounts.system_program)?;
                for plugin in &args.plugins.unwrap_or_default() {
                    initialize_plugin::<
                        Asset,
                    >(
                        &plugin.plugin,
                        &plugin.authority.unwrap_or(plugin.plugin.manager()),
                        ctx.accounts.asset,
                        ctx.accounts.payer,
                        ctx.accounts.system_program,
                    )?;
                }
                let (_, _, plugin_registry) = fetch_core_data::<
                    Asset,
                >(ctx.accounts.asset)?;
                let mut approved = true;
                if let Some(plugin_registry) = plugin_registry {
                    for record in plugin_registry.registry {
                        if match PluginType::check_create(&record.plugin_type) {
                            CheckResult::CanApprove | CheckResult::CanReject => true,
                            _ => false,
                        } {
                            let result = Plugin::validate_create(
                                &Plugin::load(ctx.accounts.asset, record.offset)?,
                                ctx
                                    .accounts
                                    .owner
                                    .unwrap_or(
                                        ctx.accounts.update_authority.unwrap_or(ctx.accounts.payer),
                                    ),
                                None,
                                &record.authority,
                                None,
                                None,
                            )?;
                            if result == ValidationResult::Rejected {
                                return Err(MplCoreError::InvalidAuthority.into());
                            } else if result == ValidationResult::Approved {
                                approved = true;
                            }
                        }
                    }
                }
                if !approved {
                    return Err(MplCoreError::InvalidAuthority.into());
                }
            }
            Ok(())
        }
    }
    pub(crate) use create::*;
    mod create_collection {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{
            account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
            program_memory::sol_memcpy, rent::Rent, system_instruction, system_program,
            sysvar::Sysvar,
        };
        use crate::{
            error::MplCoreError, instruction::accounts::CreateCollectionAccounts,
            plugins::{create_meta_idempotent, initialize_plugin, PluginAuthorityPair},
            state::{Collection, Key},
        };
        #[repr(C)]
        pub(crate) struct CreateCollectionArgs {
            pub(crate) name: String,
            pub(crate) uri: String,
            pub(crate) plugins: Option<Vec<PluginAuthorityPair>>,
        }
        impl borsh::ser::BorshSerialize for CreateCollectionArgs
        where
            String: borsh::ser::BorshSerialize,
            String: borsh::ser::BorshSerialize,
            Option<Vec<PluginAuthorityPair>>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.name, writer)?;
                borsh::BorshSerialize::serialize(&self.uri, writer)?;
                borsh::BorshSerialize::serialize(&self.plugins, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for CreateCollectionArgs
        where
            String: borsh::BorshDeserialize,
            String: borsh::BorshDeserialize,
            Option<Vec<PluginAuthorityPair>>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    name: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    uri: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    plugins: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for CreateCollectionArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for CreateCollectionArgs {
            #[inline]
            fn eq(&self, other: &CreateCollectionArgs) -> bool {
                self.name == other.name && self.uri == other.uri
                    && self.plugins == other.plugins
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for CreateCollectionArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for CreateCollectionArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<String>;
                let _: ::core::cmp::AssertParamIsEq<Option<Vec<PluginAuthorityPair>>>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for CreateCollectionArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "CreateCollectionArgs",
                    "name",
                    &self.name,
                    "uri",
                    &self.uri,
                    "plugins",
                    &&self.plugins,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for CreateCollectionArgs {
            #[inline]
            fn clone(&self) -> CreateCollectionArgs {
                CreateCollectionArgs {
                    name: ::core::clone::Clone::clone(&self.name),
                    uri: ::core::clone::Clone::clone(&self.uri),
                    plugins: ::core::clone::Clone::clone(&self.plugins),
                }
            }
        }
        pub(crate) fn create_collection<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: CreateCollectionArgs,
        ) -> ProgramResult {
            let ctx = CreateCollectionAccounts::context(accounts)?;
            let rent = Rent::get()?;
            assert_signer(ctx.accounts.collection)?;
            assert_signer(ctx.accounts.payer)?;
            if *ctx.accounts.system_program.key != system_program::id() {
                return Err(MplCoreError::InvalidSystemProgram.into());
            }
            let new_collection = Collection {
                key: Key::Collection,
                update_authority: *ctx
                    .accounts
                    .update_authority
                    .unwrap_or(ctx.accounts.payer)
                    .key,
                name: args.name,
                uri: args.uri,
                num_minted: 0,
                current_size: 0,
            };
            let serialized_data = new_collection.try_to_vec()?;
            let lamports = rent.minimum_balance(serialized_data.len());
            invoke(
                &system_instruction::create_account(
                    ctx.accounts.payer.key,
                    ctx.accounts.collection.key,
                    lamports,
                    serialized_data.len() as u64,
                    &crate::id(),
                ),
                &[
                    ctx.accounts.payer.clone(),
                    ctx.accounts.collection.clone(),
                    ctx.accounts.system_program.clone(),
                ],
            )?;
            sol_memcpy(
                &mut ctx.accounts.collection.try_borrow_mut_data()?,
                &serialized_data,
                serialized_data.len(),
            );
            drop(serialized_data);
            create_meta_idempotent::<
                Collection,
            >(ctx.accounts.collection, ctx.accounts.payer, ctx.accounts.system_program)?;
            for plugin in args.plugins.unwrap_or_default() {
                initialize_plugin::<
                    Collection,
                >(
                    &plugin.plugin,
                    &plugin.authority.unwrap_or(plugin.plugin.manager()),
                    ctx.accounts.collection,
                    ctx.accounts.payer,
                    ctx.accounts.system_program,
                )?;
            }
            Ok(())
        }
    }
    pub(crate) use create_collection::*;
    mod add_plugin {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};
        use crate::{
            error::MplCoreError,
            instruction::accounts::{AddCollectionPluginAccounts, AddPluginAccounts},
            plugins::{
                create_meta_idempotent, initialize_plugin, Plugin, PluginType,
                ValidationResult,
            },
            state::{Asset, Authority, Collection, DataBlob, Key, SolanaAccount},
            utils::{
                load_key, resolve_authority, validate_asset_permissions,
                validate_collection_permissions,
            },
        };
        #[repr(C)]
        pub(crate) struct AddPluginArgs {
            plugin: Plugin,
            init_authority: Option<Authority>,
        }
        impl borsh::ser::BorshSerialize for AddPluginArgs
        where
            Plugin: borsh::ser::BorshSerialize,
            Option<Authority>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.plugin, writer)?;
                borsh::BorshSerialize::serialize(&self.init_authority, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for AddPluginArgs
        where
            Plugin: borsh::BorshDeserialize,
            Option<Authority>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    plugin: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    init_authority: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for AddPluginArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for AddPluginArgs {
            #[inline]
            fn eq(&self, other: &AddPluginArgs) -> bool {
                self.plugin == other.plugin
                    && self.init_authority == other.init_authority
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for AddPluginArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for AddPluginArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Plugin>;
                let _: ::core::cmp::AssertParamIsEq<Option<Authority>>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for AddPluginArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "AddPluginArgs",
                    "plugin",
                    &self.plugin,
                    "init_authority",
                    &&self.init_authority,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for AddPluginArgs {
            #[inline]
            fn clone(&self) -> AddPluginArgs {
                AddPluginArgs {
                    plugin: ::core::clone::Clone::clone(&self.plugin),
                    init_authority: ::core::clone::Clone::clone(&self.init_authority),
                }
            }
        }
        pub(crate) fn add_plugin<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: AddPluginArgs,
        ) -> ProgramResult {
            let ctx = AddPluginAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            if let Key::HashedAsset = load_key(ctx.accounts.asset, 0)? {
                ::solana_program::log::sol_log(
                    "Error: Adding plugin to compressed is not available",
                );
                return Err(MplCoreError::NotAvailable.into());
            }
            if Plugin::validate_add_plugin(
                &args.plugin,
                authority,
                None,
                &args.init_authority.unwrap_or(args.plugin.manager()),
                None,
                None,
            )? == ValidationResult::Rejected
            {
                return Err(MplCoreError::InvalidAuthority.into());
            }
            let (mut asset, _, _) = validate_asset_permissions(
                authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                None,
                Some(&args.plugin),
                Asset::check_add_plugin,
                Collection::check_add_plugin,
                PluginType::check_add_plugin,
                Asset::validate_add_plugin,
                Collection::validate_add_plugin,
                Plugin::validate_add_plugin,
            )?;
            asset.increment_seq_and_save(ctx.accounts.asset)?;
            process_add_plugin::<
                Asset,
            >(
                ctx.accounts.asset,
                ctx.accounts.payer,
                ctx.accounts.system_program,
                &args.plugin,
                &args.init_authority.unwrap_or(args.plugin.manager()),
            )
        }
        #[repr(C)]
        pub(crate) struct AddCollectionPluginArgs {
            plugin: Plugin,
            init_authority: Option<Authority>,
        }
        impl borsh::ser::BorshSerialize for AddCollectionPluginArgs
        where
            Plugin: borsh::ser::BorshSerialize,
            Option<Authority>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.plugin, writer)?;
                borsh::BorshSerialize::serialize(&self.init_authority, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for AddCollectionPluginArgs
        where
            Plugin: borsh::BorshDeserialize,
            Option<Authority>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    plugin: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    init_authority: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for AddCollectionPluginArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for AddCollectionPluginArgs {
            #[inline]
            fn eq(&self, other: &AddCollectionPluginArgs) -> bool {
                self.plugin == other.plugin
                    && self.init_authority == other.init_authority
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for AddCollectionPluginArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for AddCollectionPluginArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Plugin>;
                let _: ::core::cmp::AssertParamIsEq<Option<Authority>>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for AddCollectionPluginArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "AddCollectionPluginArgs",
                    "plugin",
                    &self.plugin,
                    "init_authority",
                    &&self.init_authority,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for AddCollectionPluginArgs {
            #[inline]
            fn clone(&self) -> AddCollectionPluginArgs {
                AddCollectionPluginArgs {
                    plugin: ::core::clone::Clone::clone(&self.plugin),
                    init_authority: ::core::clone::Clone::clone(&self.init_authority),
                }
            }
        }
        pub(crate) fn add_collection_plugin<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: AddCollectionPluginArgs,
        ) -> ProgramResult {
            let ctx = AddCollectionPluginAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            let _ = validate_collection_permissions(
                authority,
                ctx.accounts.collection,
                Some(&args.plugin),
                Collection::check_add_plugin,
                PluginType::check_add_plugin,
                Collection::validate_add_plugin,
                Plugin::validate_add_plugin,
            )?;
            process_add_plugin::<
                Collection,
            >(
                ctx.accounts.collection,
                ctx.accounts.payer,
                ctx.accounts.system_program,
                &args.plugin,
                &args.init_authority.unwrap_or(args.plugin.manager()),
            )
        }
        fn process_add_plugin<'a, T: DataBlob + SolanaAccount>(
            account: &AccountInfo<'a>,
            payer: &AccountInfo<'a>,
            system_program: &AccountInfo<'a>,
            plugin: &Plugin,
            authority: &Authority,
        ) -> ProgramResult {
            ::solana_program::log::sol_log("Creating meta if it doesn't exist");
            create_meta_idempotent::<T>(account, payer, system_program)?;
            ::solana_program::log::sol_log("Initializing plugin");
            initialize_plugin::<T>(plugin, authority, account, payer, system_program)?;
            ::solana_program::log::sol_log("Plugin added successfully");
            Ok(())
        }
    }
    pub(crate) use add_plugin::*;
    mod remove_plugin {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};
        use crate::{
            error::MplCoreError,
            instruction::accounts::{
                RemoveCollectionPluginAccounts, RemovePluginAccounts,
            },
            plugins::{delete_plugin, fetch_wrapped_plugin, Plugin, PluginType},
            state::{Asset, Collection, DataBlob, Key},
            utils::{
                fetch_core_data, load_key, resolve_authority, validate_asset_permissions,
                validate_collection_permissions,
            },
        };
        #[repr(C)]
        pub(crate) struct RemovePluginArgs {
            plugin_type: PluginType,
        }
        impl borsh::ser::BorshSerialize for RemovePluginArgs
        where
            PluginType: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.plugin_type, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for RemovePluginArgs
        where
            PluginType: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    plugin_type: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for RemovePluginArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for RemovePluginArgs {
            #[inline]
            fn eq(&self, other: &RemovePluginArgs) -> bool {
                self.plugin_type == other.plugin_type
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for RemovePluginArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for RemovePluginArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<PluginType>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for RemovePluginArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "RemovePluginArgs",
                    "plugin_type",
                    &&self.plugin_type,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for RemovePluginArgs {
            #[inline]
            fn clone(&self) -> RemovePluginArgs {
                RemovePluginArgs {
                    plugin_type: ::core::clone::Clone::clone(&self.plugin_type),
                }
            }
        }
        pub(crate) fn remove_plugin<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: RemovePluginArgs,
        ) -> ProgramResult {
            let ctx = RemovePluginAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            if let Key::HashedAsset = load_key(ctx.accounts.asset, 0)? {
                ::solana_program::log::sol_log(
                    "Error: Remove plugin for compressed is not available",
                );
                return Err(MplCoreError::NotAvailable.into());
            }
            let (mut asset, plugin_header, plugin_registry) = fetch_core_data::<
                Asset,
            >(ctx.accounts.asset)?;
            if plugin_header.is_none() || plugin_registry.is_none() {
                return Err(MplCoreError::PluginNotFound.into());
            }
            let (_, plugin_to_remove) = fetch_wrapped_plugin::<
                Asset,
            >(ctx.accounts.asset, args.plugin_type)?;
            let _ = validate_asset_permissions(
                authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                None,
                Some(&plugin_to_remove),
                Asset::check_add_plugin,
                Collection::check_add_plugin,
                PluginType::check_add_plugin,
                Asset::validate_add_plugin,
                Collection::validate_add_plugin,
                Plugin::validate_add_plugin,
            )?;
            asset.increment_seq_and_save(ctx.accounts.asset)?;
            process_remove_plugin(
                &args.plugin_type,
                &asset,
                ctx.accounts.asset,
                ctx.accounts.payer,
                ctx.accounts.system_program,
            )
        }
        #[repr(C)]
        pub(crate) struct RemoveCollectionPluginArgs {
            plugin_type: PluginType,
        }
        impl borsh::ser::BorshSerialize for RemoveCollectionPluginArgs
        where
            PluginType: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.plugin_type, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for RemoveCollectionPluginArgs
        where
            PluginType: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    plugin_type: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for RemoveCollectionPluginArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for RemoveCollectionPluginArgs {
            #[inline]
            fn eq(&self, other: &RemoveCollectionPluginArgs) -> bool {
                self.plugin_type == other.plugin_type
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for RemoveCollectionPluginArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for RemoveCollectionPluginArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<PluginType>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for RemoveCollectionPluginArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "RemoveCollectionPluginArgs",
                    "plugin_type",
                    &&self.plugin_type,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for RemoveCollectionPluginArgs {
            #[inline]
            fn clone(&self) -> RemoveCollectionPluginArgs {
                RemoveCollectionPluginArgs {
                    plugin_type: ::core::clone::Clone::clone(&self.plugin_type),
                }
            }
        }
        pub(crate) fn remove_collection_plugin<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: RemoveCollectionPluginArgs,
        ) -> ProgramResult {
            let ctx = RemoveCollectionPluginAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            let (collection, plugin_header, plugin_registry) = fetch_core_data::<
                Collection,
            >(ctx.accounts.collection)?;
            if plugin_header.is_none() || plugin_registry.is_none() {
                return Err(MplCoreError::PluginNotFound.into());
            }
            let (_, plugin_to_remove) = fetch_wrapped_plugin::<
                Collection,
            >(ctx.accounts.collection, args.plugin_type)?;
            let _ = validate_collection_permissions(
                authority,
                ctx.accounts.collection,
                Some(&plugin_to_remove),
                Collection::check_add_plugin,
                PluginType::check_add_plugin,
                Collection::validate_add_plugin,
                Plugin::validate_add_plugin,
            )?;
            process_remove_plugin(
                &args.plugin_type,
                &collection,
                ctx.accounts.collection,
                ctx.accounts.payer,
                ctx.accounts.system_program,
            )
        }
        fn process_remove_plugin<'a, T: DataBlob>(
            plugin_type: &PluginType,
            core: &T,
            account: &AccountInfo<'a>,
            payer: &AccountInfo<'a>,
            system_program: &AccountInfo<'a>,
        ) -> ProgramResult {
            delete_plugin(plugin_type, core, account, payer, system_program)
        }
    }
    pub(crate) use remove_plugin::*;
    mod approve_plugin_authority {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};
        use crate::{
            error::MplCoreError,
            instruction::accounts::{
                ApproveCollectionPluginAuthorityAccounts, ApprovePluginAuthorityAccounts,
            },
            plugins::{
                approve_authority_on_plugin, fetch_wrapped_plugin, Plugin, PluginType,
            },
            state::{
                Asset, Authority, Collection, CoreAsset, DataBlob, Key, SolanaAccount,
            },
            utils::{
                fetch_core_data, load_key, resolve_authority, validate_asset_permissions,
                validate_collection_permissions,
            },
        };
        #[repr(C)]
        pub(crate) struct ApprovePluginAuthorityArgs {
            pub plugin_type: PluginType,
            pub new_authority: Authority,
        }
        impl borsh::ser::BorshSerialize for ApprovePluginAuthorityArgs
        where
            PluginType: borsh::ser::BorshSerialize,
            Authority: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.plugin_type, writer)?;
                borsh::BorshSerialize::serialize(&self.new_authority, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for ApprovePluginAuthorityArgs
        where
            PluginType: borsh::BorshDeserialize,
            Authority: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    plugin_type: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    new_authority: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for ApprovePluginAuthorityArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for ApprovePluginAuthorityArgs {
            #[inline]
            fn eq(&self, other: &ApprovePluginAuthorityArgs) -> bool {
                self.plugin_type == other.plugin_type
                    && self.new_authority == other.new_authority
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for ApprovePluginAuthorityArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for ApprovePluginAuthorityArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<PluginType>;
                let _: ::core::cmp::AssertParamIsEq<Authority>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ApprovePluginAuthorityArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "ApprovePluginAuthorityArgs",
                    "plugin_type",
                    &self.plugin_type,
                    "new_authority",
                    &&self.new_authority,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for ApprovePluginAuthorityArgs {
            #[inline]
            fn clone(&self) -> ApprovePluginAuthorityArgs {
                ApprovePluginAuthorityArgs {
                    plugin_type: ::core::clone::Clone::clone(&self.plugin_type),
                    new_authority: ::core::clone::Clone::clone(&self.new_authority),
                }
            }
        }
        pub(crate) fn approve_plugin_authority<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: ApprovePluginAuthorityArgs,
        ) -> ProgramResult {
            let ctx = ApprovePluginAuthorityAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            if let Key::HashedAsset = load_key(ctx.accounts.asset, 0)? {
                ::solana_program::log::sol_log(
                    "Error: Approve plugin authority for compressed is not available",
                );
                return Err(MplCoreError::NotAvailable.into());
            }
            let (_, plugin) = fetch_wrapped_plugin::<
                Asset,
            >(ctx.accounts.asset, args.plugin_type)?;
            let (mut asset, _, _) = validate_asset_permissions(
                authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                None,
                Some(&plugin),
                Asset::check_approve_plugin_authority,
                Collection::check_approve_plugin_authority,
                PluginType::check_approve_plugin_authority,
                Asset::validate_approve_plugin_authority,
                Collection::validate_approve_plugin_authority,
                Plugin::validate_approve_plugin_authority,
            )?;
            asset.increment_seq_and_save(ctx.accounts.asset)?;
            process_approve_plugin_authority::<
                Asset,
            >(
                ctx.accounts.asset,
                ctx.accounts.payer,
                ctx.accounts.system_program,
                &args.plugin_type,
                &args.new_authority,
            )
        }
        #[repr(C)]
        pub(crate) struct ApproveCollectionPluginAuthorityArgs {
            pub plugin_type: PluginType,
            pub new_authority: Authority,
        }
        impl borsh::ser::BorshSerialize for ApproveCollectionPluginAuthorityArgs
        where
            PluginType: borsh::ser::BorshSerialize,
            Authority: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.plugin_type, writer)?;
                borsh::BorshSerialize::serialize(&self.new_authority, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for ApproveCollectionPluginAuthorityArgs
        where
            PluginType: borsh::BorshDeserialize,
            Authority: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    plugin_type: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    new_authority: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq
        for ApproveCollectionPluginAuthorityArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for ApproveCollectionPluginAuthorityArgs {
            #[inline]
            fn eq(&self, other: &ApproveCollectionPluginAuthorityArgs) -> bool {
                self.plugin_type == other.plugin_type
                    && self.new_authority == other.new_authority
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for ApproveCollectionPluginAuthorityArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for ApproveCollectionPluginAuthorityArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<PluginType>;
                let _: ::core::cmp::AssertParamIsEq<Authority>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ApproveCollectionPluginAuthorityArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "ApproveCollectionPluginAuthorityArgs",
                    "plugin_type",
                    &self.plugin_type,
                    "new_authority",
                    &&self.new_authority,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for ApproveCollectionPluginAuthorityArgs {
            #[inline]
            fn clone(&self) -> ApproveCollectionPluginAuthorityArgs {
                ApproveCollectionPluginAuthorityArgs {
                    plugin_type: ::core::clone::Clone::clone(&self.plugin_type),
                    new_authority: ::core::clone::Clone::clone(&self.new_authority),
                }
            }
        }
        pub(crate) fn approve_collection_plugin_authority<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: ApproveCollectionPluginAuthorityArgs,
        ) -> ProgramResult {
            let ctx = ApproveCollectionPluginAuthorityAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            let (_, plugin) = fetch_wrapped_plugin::<
                Collection,
            >(ctx.accounts.collection, args.plugin_type)?;
            let _ = validate_collection_permissions(
                authority,
                ctx.accounts.collection,
                Some(&plugin),
                Collection::check_approve_plugin_authority,
                PluginType::check_approve_plugin_authority,
                Collection::validate_approve_plugin_authority,
                Plugin::validate_approve_plugin_authority,
            )?;
            process_approve_plugin_authority::<
                Collection,
            >(
                ctx.accounts.collection,
                ctx.accounts.payer,
                ctx.accounts.system_program,
                &args.plugin_type,
                &args.new_authority,
            )
        }
        fn process_approve_plugin_authority<'a, T: CoreAsset + DataBlob + SolanaAccount>(
            core_info: &AccountInfo<'a>,
            payer: &AccountInfo<'a>,
            system_program: &AccountInfo<'a>,
            plugin_type: &PluginType,
            new_authority: &Authority,
        ) -> ProgramResult {
            let (_, plugin_header, plugin_registry) = fetch_core_data::<T>(core_info)?;
            let plugin_header = match plugin_header {
                Some(header) => header,
                None => return Err(MplCoreError::PluginsNotInitialized.into()),
            };
            let mut plugin_registry = match plugin_registry {
                Some(registry) => registry,
                None => return Err(MplCoreError::PluginsNotInitialized.into()),
            };
            approve_authority_on_plugin::<
                T,
            >(
                plugin_type,
                new_authority,
                core_info,
                &plugin_header,
                &mut plugin_registry,
                payer,
                system_program,
            )
        }
    }
    pub(crate) use approve_plugin_authority::*;
    mod revoke_plugin_authority {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};
        use crate::{
            error::MplCoreError,
            instruction::accounts::{
                RevokeCollectionPluginAuthorityAccounts, RevokePluginAuthorityAccounts,
            },
            plugins::{
                fetch_wrapped_plugin, revoke_authority_on_plugin, Plugin, PluginHeader,
                PluginRegistry, PluginType,
            },
            state::{Asset, Collection, Key},
            utils::{
                fetch_core_data, load_key, resolve_authority, validate_asset_permissions,
                validate_collection_permissions,
            },
        };
        #[repr(C)]
        pub(crate) struct RevokePluginAuthorityArgs {
            pub plugin_type: PluginType,
        }
        impl borsh::ser::BorshSerialize for RevokePluginAuthorityArgs
        where
            PluginType: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.plugin_type, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for RevokePluginAuthorityArgs
        where
            PluginType: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    plugin_type: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for RevokePluginAuthorityArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for RevokePluginAuthorityArgs {
            #[inline]
            fn eq(&self, other: &RevokePluginAuthorityArgs) -> bool {
                self.plugin_type == other.plugin_type
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for RevokePluginAuthorityArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for RevokePluginAuthorityArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<PluginType>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for RevokePluginAuthorityArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "RevokePluginAuthorityArgs",
                    "plugin_type",
                    &&self.plugin_type,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for RevokePluginAuthorityArgs {
            #[inline]
            fn clone(&self) -> RevokePluginAuthorityArgs {
                RevokePluginAuthorityArgs {
                    plugin_type: ::core::clone::Clone::clone(&self.plugin_type),
                }
            }
        }
        pub(crate) fn revoke_plugin_authority<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: RevokePluginAuthorityArgs,
        ) -> ProgramResult {
            let ctx = RevokePluginAuthorityAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            if let Key::HashedAsset = load_key(ctx.accounts.asset, 0)? {
                ::solana_program::log::sol_log(
                    "Error: Revoke plugin authority for compressed is not available",
                );
                return Err(MplCoreError::NotAvailable.into());
            }
            let (mut asset, plugin_header, mut plugin_registry) = fetch_core_data::<
                Asset,
            >(ctx.accounts.asset)?;
            let (_, plugin) = fetch_wrapped_plugin::<
                Asset,
            >(ctx.accounts.asset, args.plugin_type)?;
            let _ = validate_asset_permissions(
                authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                None,
                Some(&plugin),
                Asset::check_revoke_plugin_authority,
                Collection::check_revoke_plugin_authority,
                PluginType::check_revoke_plugin_authority,
                Asset::validate_revoke_plugin_authority,
                Collection::validate_revoke_plugin_authority,
                Plugin::validate_revoke_plugin_authority,
            )?;
            asset.increment_seq_and_save(ctx.accounts.asset)?;
            process_revoke_plugin_authority(
                ctx.accounts.asset,
                ctx.accounts.payer,
                ctx.accounts.system_program,
                &args.plugin_type,
                plugin_header.as_ref(),
                plugin_registry.as_mut(),
            )
        }
        #[repr(C)]
        pub(crate) struct RevokeCollectionPluginAuthorityArgs {
            pub plugin_type: PluginType,
        }
        impl borsh::ser::BorshSerialize for RevokeCollectionPluginAuthorityArgs
        where
            PluginType: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.plugin_type, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for RevokeCollectionPluginAuthorityArgs
        where
            PluginType: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    plugin_type: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq
        for RevokeCollectionPluginAuthorityArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for RevokeCollectionPluginAuthorityArgs {
            #[inline]
            fn eq(&self, other: &RevokeCollectionPluginAuthorityArgs) -> bool {
                self.plugin_type == other.plugin_type
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for RevokeCollectionPluginAuthorityArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for RevokeCollectionPluginAuthorityArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<PluginType>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for RevokeCollectionPluginAuthorityArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "RevokeCollectionPluginAuthorityArgs",
                    "plugin_type",
                    &&self.plugin_type,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for RevokeCollectionPluginAuthorityArgs {
            #[inline]
            fn clone(&self) -> RevokeCollectionPluginAuthorityArgs {
                RevokeCollectionPluginAuthorityArgs {
                    plugin_type: ::core::clone::Clone::clone(&self.plugin_type),
                }
            }
        }
        pub(crate) fn revoke_collection_plugin_authority<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: RevokeCollectionPluginAuthorityArgs,
        ) -> ProgramResult {
            let ctx = RevokeCollectionPluginAuthorityAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            let (_, plugin_header, mut plugin_registry) = fetch_core_data::<
                Collection,
            >(ctx.accounts.collection)?;
            let (_, plugin) = fetch_wrapped_plugin::<
                Collection,
            >(ctx.accounts.collection, args.plugin_type)?;
            let _ = validate_collection_permissions(
                authority,
                ctx.accounts.collection,
                Some(&plugin),
                Collection::check_revoke_plugin_authority,
                PluginType::check_revoke_plugin_authority,
                Collection::validate_revoke_plugin_authority,
                Plugin::validate_revoke_plugin_authority,
            )?;
            process_revoke_plugin_authority(
                ctx.accounts.collection,
                ctx.accounts.payer,
                ctx.accounts.system_program,
                &args.plugin_type,
                plugin_header.as_ref(),
                plugin_registry.as_mut(),
            )
        }
        #[allow(clippy::too_many_arguments)]
        fn process_revoke_plugin_authority<'a>(
            core_info: &AccountInfo<'a>,
            payer: &AccountInfo<'a>,
            system_program: &AccountInfo<'a>,
            plugin_type: &PluginType,
            plugin_header: Option<&PluginHeader>,
            plugin_registry: Option<&mut PluginRegistry>,
        ) -> ProgramResult {
            let plugin_header = match plugin_header {
                Some(header) => header,
                None => return Err(MplCoreError::PluginsNotInitialized.into()),
            };
            let plugin_registry = match plugin_registry {
                Some(registry) => registry,
                None => return Err(MplCoreError::PluginsNotInitialized.into()),
            };
            revoke_authority_on_plugin(
                plugin_type,
                core_info,
                plugin_header,
                plugin_registry,
                payer,
                system_program,
            )
        }
    }
    pub(crate) use revoke_plugin_authority::*;
    mod burn {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};
        use crate::{
            error::MplCoreError,
            instruction::accounts::{BurnAccounts, BurnCollectionAccounts},
            plugins::{Plugin, PluginType},
            state::{Asset, Collection, CompressionProof, Key, Wrappable},
            utils::{
                close_program_account, load_key, rebuild_account_state_from_proof_data,
                resolve_authority, validate_asset_permissions,
                validate_collection_permissions, verify_proof,
            },
        };
        pub(crate) struct BurnArgs {
            compression_proof: Option<CompressionProof>,
        }
        impl borsh::ser::BorshSerialize for BurnArgs
        where
            Option<CompressionProof>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.compression_proof, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for BurnArgs
        where
            Option<CompressionProof>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    compression_proof: borsh::BorshDeserialize::deserialize_reader(
                        reader,
                    )?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for BurnArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for BurnArgs {
            #[inline]
            fn eq(&self, other: &BurnArgs) -> bool {
                self.compression_proof == other.compression_proof
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for BurnArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for BurnArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Option<CompressionProof>>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for BurnArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "BurnArgs",
                    "compression_proof",
                    &&self.compression_proof,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for BurnArgs {
            #[inline]
            fn clone(&self) -> BurnArgs {
                BurnArgs {
                    compression_proof: ::core::clone::Clone::clone(
                        &self.compression_proof,
                    ),
                }
            }
        }
        pub(crate) fn burn<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: BurnArgs,
        ) -> ProgramResult {
            let ctx = BurnAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            match load_key(ctx.accounts.asset, 0)? {
                Key::HashedAsset => {
                    let mut compression_proof = args
                        .compression_proof
                        .ok_or(MplCoreError::MissingCompressionProof)?;
                    let system_program = ctx
                        .accounts
                        .system_program
                        .ok_or(MplCoreError::MissingSystemProgram)?;
                    let (asset, plugins) = verify_proof(
                        ctx.accounts.asset,
                        &compression_proof,
                    )?;
                    rebuild_account_state_from_proof_data(
                        asset,
                        plugins,
                        ctx.accounts.asset,
                        ctx.accounts.payer,
                        system_program,
                    )?;
                    compression_proof.seq = compression_proof.seq.saturating_add(1);
                    compression_proof.wrap()?;
                    ::solana_program::log::sol_log(
                        "Error: Burning compressed is currently not available",
                    );
                    return Err(MplCoreError::NotAvailable.into());
                }
                Key::Asset => {}
                _ => return Err(MplCoreError::IncorrectAccount.into()),
            }
            let _ = validate_asset_permissions(
                authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                None,
                None,
                Asset::check_burn,
                Collection::check_burn,
                PluginType::check_burn,
                Asset::validate_burn,
                Collection::validate_burn,
                Plugin::validate_burn,
            )?;
            process_burn(ctx.accounts.asset, authority)
        }
        pub(crate) struct BurnCollectionArgs {
            compression_proof: Option<CompressionProof>,
        }
        impl borsh::ser::BorshSerialize for BurnCollectionArgs
        where
            Option<CompressionProof>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.compression_proof, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for BurnCollectionArgs
        where
            Option<CompressionProof>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    compression_proof: borsh::BorshDeserialize::deserialize_reader(
                        reader,
                    )?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for BurnCollectionArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for BurnCollectionArgs {
            #[inline]
            fn eq(&self, other: &BurnCollectionArgs) -> bool {
                self.compression_proof == other.compression_proof
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for BurnCollectionArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for BurnCollectionArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Option<CompressionProof>>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for BurnCollectionArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "BurnCollectionArgs",
                    "compression_proof",
                    &&self.compression_proof,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for BurnCollectionArgs {
            #[inline]
            fn clone(&self) -> BurnCollectionArgs {
                BurnCollectionArgs {
                    compression_proof: ::core::clone::Clone::clone(
                        &self.compression_proof,
                    ),
                }
            }
        }
        pub(crate) fn burn_collection<'a>(
            accounts: &'a [AccountInfo<'a>],
            _args: BurnCollectionArgs,
        ) -> ProgramResult {
            let ctx = BurnCollectionAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            let _ = validate_collection_permissions(
                authority,
                ctx.accounts.collection,
                None,
                Collection::check_burn,
                PluginType::check_burn,
                Collection::validate_burn,
                Plugin::validate_burn,
            )?;
            process_burn(ctx.accounts.collection, authority)
        }
        fn process_burn<'a>(
            core_info: &AccountInfo<'a>,
            authority: &AccountInfo<'a>,
        ) -> ProgramResult {
            close_program_account(core_info, authority)
        }
    }
    pub(crate) use burn::*;
    mod transfer {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};
        use crate::{
            error::MplCoreError, instruction::accounts::TransferAccounts,
            plugins::{Plugin, PluginType},
            state::{Asset, Collection, CompressionProof, Key, SolanaAccount, Wrappable},
            utils::{
                compress_into_account_space, load_key,
                rebuild_account_state_from_proof_data, resolve_authority,
                validate_asset_permissions, verify_proof,
            },
        };
        #[repr(C)]
        pub struct TransferArgs {
            compression_proof: Option<CompressionProof>,
        }
        impl borsh::ser::BorshSerialize for TransferArgs
        where
            Option<CompressionProof>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.compression_proof, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for TransferArgs
        where
            Option<CompressionProof>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    compression_proof: borsh::BorshDeserialize::deserialize_reader(
                        reader,
                    )?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for TransferArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for TransferArgs {
            #[inline]
            fn eq(&self, other: &TransferArgs) -> bool {
                self.compression_proof == other.compression_proof
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for TransferArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for TransferArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Option<CompressionProof>>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for TransferArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "TransferArgs",
                    "compression_proof",
                    &&self.compression_proof,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for TransferArgs {
            #[inline]
            fn clone(&self) -> TransferArgs {
                TransferArgs {
                    compression_proof: ::core::clone::Clone::clone(
                        &self.compression_proof,
                    ),
                }
            }
        }
        pub(crate) fn transfer<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: TransferArgs,
        ) -> ProgramResult {
            let ctx = TransferAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            let key = load_key(ctx.accounts.asset, 0)?;
            match key {
                Key::HashedAsset => {
                    let compression_proof = args
                        .compression_proof
                        .ok_or(MplCoreError::MissingCompressionProof)?;
                    let system_program = ctx
                        .accounts
                        .system_program
                        .ok_or(MplCoreError::MissingSystemProgram)?;
                    let (mut asset, plugins) = verify_proof(
                        ctx.accounts.asset,
                        &compression_proof,
                    )?;
                    asset.owner = *ctx.accounts.new_owner.key;
                    rebuild_account_state_from_proof_data(
                        asset,
                        plugins,
                        ctx.accounts.asset,
                        ctx.accounts.payer,
                        system_program,
                    )?;
                    ::solana_program::log::sol_log(
                        "Error: Transferring compressed is currently not available",
                    );
                    return Err(MplCoreError::NotAvailable.into());
                }
                Key::Asset => {}
                _ => return Err(MplCoreError::IncorrectAccount.into()),
            }
            let (mut asset, _, plugin_registry) = validate_asset_permissions(
                authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                Some(ctx.accounts.new_owner),
                None,
                Asset::check_transfer,
                Collection::check_transfer,
                PluginType::check_transfer,
                Asset::validate_transfer,
                Collection::validate_transfer,
                Plugin::validate_transfer,
            )?;
            asset.owner = *ctx.accounts.new_owner.key;
            match key {
                Key::HashedAsset => {
                    let system_program = ctx
                        .accounts
                        .system_program
                        .ok_or(MplCoreError::MissingSystemProgram)?;
                    let compression_proof = compress_into_account_space(
                        asset,
                        plugin_registry,
                        ctx.accounts.asset,
                        ctx.accounts.payer,
                        system_program,
                    )?;
                    compression_proof.wrap()
                }
                Key::Asset => {
                    asset.seq = asset.seq.map(|seq| seq.saturating_add(1));
                    asset.save(ctx.accounts.asset, 0)
                }
                _ => ::core::panicking::panic("internal error: entered unreachable code"),
            }
        }
    }
    pub(crate) use transfer::*;
    mod update {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{
            account_info::AccountInfo, entrypoint::ProgramResult, msg,
            program_memory::sol_memcpy,
        };
        use crate::{
            error::MplCoreError,
            instruction::accounts::{UpdateAccounts, UpdateCollectionAccounts},
            plugins::{Plugin, PluginHeader, PluginRegistry, PluginType, RegistryRecord},
            state::{Asset, Collection, DataBlob, Key, SolanaAccount, UpdateAuthority},
            utils::{
                load_key, resize_or_reallocate_account, resolve_authority,
                validate_asset_permissions, validate_collection_permissions,
            },
        };
        #[repr(C)]
        pub(crate) struct UpdateArgs {
            pub new_name: Option<String>,
            pub new_uri: Option<String>,
        }
        impl borsh::ser::BorshSerialize for UpdateArgs
        where
            Option<String>: borsh::ser::BorshSerialize,
            Option<String>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.new_name, writer)?;
                borsh::BorshSerialize::serialize(&self.new_uri, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for UpdateArgs
        where
            Option<String>: borsh::BorshDeserialize,
            Option<String>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    new_name: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    new_uri: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for UpdateArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for UpdateArgs {
            #[inline]
            fn eq(&self, other: &UpdateArgs) -> bool {
                self.new_name == other.new_name && self.new_uri == other.new_uri
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for UpdateArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for UpdateArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Option<String>>;
                let _: ::core::cmp::AssertParamIsEq<Option<String>>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for UpdateArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "UpdateArgs",
                    "new_name",
                    &self.new_name,
                    "new_uri",
                    &&self.new_uri,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for UpdateArgs {
            #[inline]
            fn clone(&self) -> UpdateArgs {
                UpdateArgs {
                    new_name: ::core::clone::Clone::clone(&self.new_name),
                    new_uri: ::core::clone::Clone::clone(&self.new_uri),
                }
            }
        }
        pub(crate) fn update<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: UpdateArgs,
        ) -> ProgramResult {
            let ctx = UpdateAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            if let Key::HashedAsset = load_key(ctx.accounts.asset, 0)? {
                ::solana_program::log::sol_log(
                    "Error: Update for compressed is not available",
                );
                return Err(MplCoreError::NotAvailable.into());
            }
            let (mut asset, plugin_header, plugin_registry) = validate_asset_permissions(
                authority,
                ctx.accounts.asset,
                ctx.accounts.collection,
                None,
                None,
                Asset::check_update,
                Collection::check_update,
                PluginType::check_update,
                Asset::validate_update,
                Collection::validate_update,
                Plugin::validate_update,
            )?;
            asset.increment_seq_and_save(ctx.accounts.asset)?;
            let asset_size = asset.get_size() as isize;
            let mut dirty = false;
            if let Some(new_update_authority) = ctx.accounts.new_update_authority {
                asset
                    .update_authority = UpdateAuthority::Address(
                    *new_update_authority.key,
                );
                dirty = true;
            }
            if let Some(new_name) = &args.new_name {
                asset.name = new_name.clone();
                dirty = true;
            }
            if let Some(new_uri) = &args.new_uri {
                asset.uri = new_uri.clone();
                dirty = true;
            }
            if dirty {
                process_update(
                    asset,
                    &plugin_header,
                    &plugin_registry,
                    asset_size,
                    ctx.accounts.asset,
                    ctx.accounts.payer,
                    ctx.accounts.system_program,
                )?;
            }
            Ok(())
        }
        #[repr(C)]
        pub(crate) struct UpdateCollectionArgs {
            pub new_name: Option<String>,
            pub new_uri: Option<String>,
        }
        impl borsh::ser::BorshSerialize for UpdateCollectionArgs
        where
            Option<String>: borsh::ser::BorshSerialize,
            Option<String>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.new_name, writer)?;
                borsh::BorshSerialize::serialize(&self.new_uri, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for UpdateCollectionArgs
        where
            Option<String>: borsh::BorshDeserialize,
            Option<String>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    new_name: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    new_uri: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for UpdateCollectionArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for UpdateCollectionArgs {
            #[inline]
            fn eq(&self, other: &UpdateCollectionArgs) -> bool {
                self.new_name == other.new_name && self.new_uri == other.new_uri
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for UpdateCollectionArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for UpdateCollectionArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Option<String>>;
                let _: ::core::cmp::AssertParamIsEq<Option<String>>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for UpdateCollectionArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "UpdateCollectionArgs",
                    "new_name",
                    &self.new_name,
                    "new_uri",
                    &&self.new_uri,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for UpdateCollectionArgs {
            #[inline]
            fn clone(&self) -> UpdateCollectionArgs {
                UpdateCollectionArgs {
                    new_name: ::core::clone::Clone::clone(&self.new_name),
                    new_uri: ::core::clone::Clone::clone(&self.new_uri),
                }
            }
        }
        pub(crate) fn update_collection<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: UpdateCollectionArgs,
        ) -> ProgramResult {
            let ctx = UpdateCollectionAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            let (mut collection, plugin_header, plugin_registry) = validate_collection_permissions(
                authority,
                ctx.accounts.collection,
                None,
                Collection::check_update,
                PluginType::check_update,
                Collection::validate_update,
                Plugin::validate_update,
            )?;
            let collection_size = collection.get_size() as isize;
            let mut dirty = false;
            if let Some(new_update_authority) = ctx.accounts.new_update_authority {
                collection.update_authority = *new_update_authority.key;
                dirty = true;
            }
            if let Some(new_name) = &args.new_name {
                collection.name = new_name.clone();
                dirty = true;
            }
            if let Some(new_uri) = &args.new_uri {
                collection.uri = new_uri.clone();
                dirty = true;
            }
            if dirty {
                process_update(
                    collection,
                    &plugin_header,
                    &plugin_registry,
                    collection_size,
                    ctx.accounts.collection,
                    ctx.accounts.payer,
                    ctx.accounts.system_program,
                )?;
            }
            Ok(())
        }
        fn process_update<'a, T: DataBlob + SolanaAccount>(
            core: T,
            plugin_header: &Option<PluginHeader>,
            plugin_registry: &Option<PluginRegistry>,
            core_size: isize,
            account: &AccountInfo<'a>,
            payer: &AccountInfo<'a>,
            system_program: &AccountInfo<'a>,
        ) -> ProgramResult {
            if let (Some(mut plugin_header), Some(mut plugin_registry)) = (
                plugin_header.clone(),
                plugin_registry.clone(),
            ) {
                let new_core_size = core.get_size() as isize;
                let size_diff = new_core_size
                    .checked_sub(core_size)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let new_size = (account.data_len() as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let registry_offset = plugin_header.plugin_registry_offset;
                let new_registry_offset = (registry_offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                plugin_header.plugin_registry_offset = new_registry_offset as usize;
                let plugin_offset = core_size
                    .checked_add(plugin_header.get_size() as isize)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let new_plugin_offset = plugin_offset
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let src = account
                    .data
                    .borrow()[(plugin_offset as usize)..registry_offset]
                    .to_vec();
                resize_or_reallocate_account(
                    account,
                    payer,
                    system_program,
                    new_size as usize,
                )?;
                sol_memcpy(
                    &mut account.data.borrow_mut()[(new_plugin_offset as usize)..],
                    &src,
                    src.len(),
                );
                plugin_header.save(account, new_core_size as usize)?;
                plugin_registry
                    .registry = plugin_registry
                    .registry
                    .iter_mut()
                    .map(|record| {
                        let new_offset = (record.offset as isize)
                            .checked_add(size_diff)
                            .ok_or(MplCoreError::NumericalOverflow)?;
                        Ok(RegistryRecord {
                            plugin_type: record.plugin_type,
                            offset: new_offset as usize,
                            authority: record.authority,
                        })
                    })
                    .collect::<Result<Vec<_>, MplCoreError>>()?;
                plugin_registry.save(account, new_registry_offset as usize)?;
            } else {
                resize_or_reallocate_account(
                    account,
                    payer,
                    system_program,
                    core.get_size(),
                )?;
            }
            core.save(account, 0)?;
            Ok(())
        }
    }
    pub(crate) use update::*;
    mod compress {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};
        use crate::{
            error::MplCoreError, instruction::accounts::CompressAccounts,
            plugins::{Plugin, PluginType},
            state::{Asset, Collection, Key, Wrappable},
            utils::{
                compress_into_account_space, fetch_core_data, load_key,
                resolve_authority, validate_asset_permissions,
            },
        };
        #[repr(C)]
        pub struct CompressArgs {}
        impl borsh::ser::BorshSerialize for CompressArgs {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for CompressArgs {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {})
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for CompressArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for CompressArgs {
            #[inline]
            fn eq(&self, other: &CompressArgs) -> bool {
                true
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for CompressArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for CompressArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for CompressArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "CompressArgs")
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for CompressArgs {
            #[inline]
            fn clone(&self) -> CompressArgs {
                CompressArgs {}
            }
        }
        pub(crate) fn compress<'a>(
            accounts: &'a [AccountInfo<'a>],
            _args: CompressArgs,
        ) -> ProgramResult {
            let ctx = CompressAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            match load_key(ctx.accounts.asset, 0)? {
                Key::Asset => {
                    let (asset, _, plugin_registry) = fetch_core_data::<
                        Asset,
                    >(ctx.accounts.asset)?;
                    let _ = validate_asset_permissions(
                        authority,
                        ctx.accounts.asset,
                        ctx.accounts.collection,
                        None,
                        None,
                        Asset::check_compress,
                        Collection::check_compress,
                        PluginType::check_compress,
                        Asset::validate_compress,
                        Collection::validate_compress,
                        Plugin::validate_compress,
                    )?;
                    let compression_proof = compress_into_account_space(
                        asset,
                        plugin_registry,
                        ctx.accounts.asset,
                        ctx.accounts.payer,
                        ctx.accounts.system_program,
                    )?;
                    compression_proof.wrap()?;
                    ::solana_program::log::sol_log(
                        "Error: Compression currently not available",
                    );
                    Err(MplCoreError::NotAvailable.into())
                }
                Key::HashedAsset => Err(MplCoreError::AlreadyCompressed.into()),
                _ => Err(MplCoreError::IncorrectAccount.into()),
            }
        }
    }
    pub(crate) use compress::*;
    mod decompress {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{
            account_info::AccountInfo, entrypoint::ProgramResult, msg, system_program,
        };
        use crate::{
            error::MplCoreError, instruction::accounts::DecompressAccounts,
            plugins::{Plugin, PluginType},
            state::{Asset, Collection, CompressionProof, Key},
            utils::{
                load_key, rebuild_account_state_from_proof_data, resolve_authority,
                validate_asset_permissions, verify_proof,
            },
        };
        #[repr(C)]
        pub struct DecompressArgs {
            compression_proof: CompressionProof,
        }
        impl borsh::ser::BorshSerialize for DecompressArgs
        where
            CompressionProof: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.compression_proof, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for DecompressArgs
        where
            CompressionProof: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    compression_proof: borsh::BorshDeserialize::deserialize_reader(
                        reader,
                    )?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for DecompressArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for DecompressArgs {
            #[inline]
            fn eq(&self, other: &DecompressArgs) -> bool {
                self.compression_proof == other.compression_proof
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for DecompressArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for DecompressArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<CompressionProof>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for DecompressArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "DecompressArgs",
                    "compression_proof",
                    &&self.compression_proof,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for DecompressArgs {
            #[inline]
            fn clone(&self) -> DecompressArgs {
                DecompressArgs {
                    compression_proof: ::core::clone::Clone::clone(
                        &self.compression_proof,
                    ),
                }
            }
        }
        pub(crate) fn decompress<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: DecompressArgs,
        ) -> ProgramResult {
            let ctx = DecompressAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            if *ctx.accounts.system_program.key != system_program::id() {
                return Err(MplCoreError::InvalidSystemProgram.into());
            }
            match load_key(ctx.accounts.asset, 0)? {
                Key::HashedAsset => {
                    let (mut asset, plugins) = verify_proof(
                        ctx.accounts.asset,
                        &args.compression_proof,
                    )?;
                    asset.seq = asset.seq.map(|seq| seq.saturating_add(1));
                    rebuild_account_state_from_proof_data(
                        asset,
                        plugins,
                        ctx.accounts.asset,
                        ctx.accounts.payer,
                        ctx.accounts.system_program,
                    )?;
                    let _ = validate_asset_permissions(
                        authority,
                        ctx.accounts.asset,
                        ctx.accounts.collection,
                        None,
                        None,
                        Asset::check_decompress,
                        Collection::check_decompress,
                        PluginType::check_decompress,
                        Asset::validate_decompress,
                        Collection::validate_decompress,
                        Plugin::validate_decompress,
                    )?;
                    ::solana_program::log::sol_log(
                        "Error: Decompression currently not available",
                    );
                    Err(MplCoreError::NotAvailable.into())
                }
                Key::Asset => Err(MplCoreError::AlreadyDecompressed.into()),
                _ => Err(MplCoreError::IncorrectAccount.into()),
            }
        }
    }
    pub(crate) use decompress::*;
    mod update_plugin {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{
            account_info::AccountInfo, entrypoint::ProgramResult, msg,
            program_memory::sol_memcpy,
        };
        use crate::{
            error::MplCoreError,
            instruction::accounts::{
                UpdateCollectionPluginAccounts, UpdatePluginAccounts,
            },
            plugins::{Plugin, PluginType, RegistryRecord, ValidationResult},
            state::{Asset, Collection, DataBlob, Key, SolanaAccount},
            utils::{
                fetch_core_data, load_key, resize_or_reallocate_account,
                resolve_authority,
            },
        };
        #[repr(C)]
        pub(crate) struct UpdatePluginArgs {
            pub plugin: Plugin,
        }
        impl borsh::ser::BorshSerialize for UpdatePluginArgs
        where
            Plugin: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.plugin, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for UpdatePluginArgs
        where
            Plugin: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    plugin: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for UpdatePluginArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for UpdatePluginArgs {
            #[inline]
            fn eq(&self, other: &UpdatePluginArgs) -> bool {
                self.plugin == other.plugin
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for UpdatePluginArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for UpdatePluginArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Plugin>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for UpdatePluginArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "UpdatePluginArgs",
                    "plugin",
                    &&self.plugin,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for UpdatePluginArgs {
            #[inline]
            fn clone(&self) -> UpdatePluginArgs {
                UpdatePluginArgs {
                    plugin: ::core::clone::Clone::clone(&self.plugin),
                }
            }
        }
        pub(crate) fn update_plugin<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: UpdatePluginArgs,
        ) -> ProgramResult {
            let ctx = UpdatePluginAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            if let Key::HashedAsset = load_key(ctx.accounts.asset, 0)? {
                ::solana_program::log::sol_log(
                    "Error: Update plugin for compressed is not available",
                );
                return Err(MplCoreError::NotAvailable.into());
            }
            let (mut asset, plugin_header, plugin_registry) = fetch_core_data::<
                Asset,
            >(ctx.accounts.asset)?;
            let mut plugin_registry = plugin_registry
                .ok_or(MplCoreError::PluginsNotInitialized)?;
            let mut plugin_header = plugin_header
                .ok_or(MplCoreError::PluginsNotInitialized)?;
            let plugin_registry_clone = plugin_registry.clone();
            let plugin_type: PluginType = (&args.plugin).into();
            let registry_record = plugin_registry_clone
                .registry
                .iter()
                .find(|record| record.plugin_type == plugin_type)
                .ok_or(MplCoreError::PluginNotFound)?;
            let plugin = Plugin::load(ctx.accounts.asset, registry_record.offset)?;
            let result = Plugin::validate_update_plugin(
                &plugin,
                &asset,
                authority,
                None,
                &registry_record.authority,
                None,
                None,
            )?;
            if result == ValidationResult::Rejected {
                return Err(MplCoreError::InvalidAuthority.into());
            } else if result == ValidationResult::Approved {
                let plugin_data = plugin.try_to_vec()?;
                let new_plugin_data = args.plugin.try_to_vec()?;
                let plugin_size = plugin_data.len() as isize;
                let size_diff = (new_plugin_data.len() as isize)
                    .checked_sub(plugin_size)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let new_size = (ctx.accounts.asset.data_len() as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let registry_offset = plugin_header.plugin_registry_offset;
                let new_registry_offset = (registry_offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                plugin_header.plugin_registry_offset = new_registry_offset as usize;
                let next_plugin_offset = (registry_record.offset as isize)
                    .checked_add(plugin_size)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let new_next_plugin_offset = next_plugin_offset
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let src = ctx
                    .accounts
                    .asset
                    .data
                    .borrow()[(next_plugin_offset as usize)..registry_offset]
                    .to_vec();
                resize_or_reallocate_account(
                    ctx.accounts.asset,
                    ctx.accounts.payer,
                    ctx.accounts.system_program,
                    new_size as usize,
                )?;
                sol_memcpy(
                    &mut ctx
                        .accounts
                        .asset
                        .data
                        .borrow_mut()[(new_next_plugin_offset as usize)..],
                    &src,
                    src.len(),
                );
                plugin_header.save(ctx.accounts.asset, asset.get_size())?;
                plugin_registry
                    .registry = plugin_registry
                    .registry
                    .clone()
                    .iter_mut()
                    .map(|record| {
                        let new_offset = if record.offset > registry_record.offset {
                            (record.offset as isize)
                                .checked_add(size_diff)
                                .ok_or(MplCoreError::NumericalOverflow)?
                        } else {
                            record.offset as isize
                        };
                        Ok(RegistryRecord {
                            plugin_type: record.plugin_type,
                            offset: new_offset as usize,
                            authority: record.authority,
                        })
                    })
                    .collect::<Result<Vec<_>, MplCoreError>>()?;
                plugin_registry.save(ctx.accounts.asset, new_registry_offset as usize)?;
                args.plugin.save(ctx.accounts.asset, registry_record.offset)?;
            } else {
                return Err(MplCoreError::InvalidAuthority.into());
            }
            asset.increment_seq_and_save(ctx.accounts.asset)?;
            process_update_plugin()
        }
        #[repr(C)]
        pub(crate) struct UpdateCollectionPluginArgs {
            pub plugin: Plugin,
        }
        impl borsh::ser::BorshSerialize for UpdateCollectionPluginArgs
        where
            Plugin: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.plugin, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for UpdateCollectionPluginArgs
        where
            Plugin: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    plugin: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for UpdateCollectionPluginArgs {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for UpdateCollectionPluginArgs {
            #[inline]
            fn eq(&self, other: &UpdateCollectionPluginArgs) -> bool {
                self.plugin == other.plugin
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for UpdateCollectionPluginArgs {}
        #[automatically_derived]
        impl ::core::cmp::Eq for UpdateCollectionPluginArgs {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Plugin>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for UpdateCollectionPluginArgs {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "UpdateCollectionPluginArgs",
                    "plugin",
                    &&self.plugin,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for UpdateCollectionPluginArgs {
            #[inline]
            fn clone(&self) -> UpdateCollectionPluginArgs {
                UpdateCollectionPluginArgs {
                    plugin: ::core::clone::Clone::clone(&self.plugin),
                }
            }
        }
        pub(crate) fn update_collection_plugin<'a>(
            accounts: &'a [AccountInfo<'a>],
            args: UpdateCollectionPluginArgs,
        ) -> ProgramResult {
            let ctx = UpdateCollectionPluginAccounts::context(accounts)?;
            assert_signer(ctx.accounts.payer)?;
            let authority = resolve_authority(
                ctx.accounts.payer,
                ctx.accounts.authority,
            )?;
            let (collection, plugin_header, plugin_registry) = fetch_core_data::<
                Collection,
            >(ctx.accounts.collection)?;
            let mut plugin_registry = plugin_registry
                .ok_or(MplCoreError::PluginsNotInitialized)?;
            let mut plugin_header = plugin_header
                .ok_or(MplCoreError::PluginsNotInitialized)?;
            let plugin_registry_clone = plugin_registry.clone();
            let plugin_type: PluginType = (&args.plugin).into();
            let registry_record = plugin_registry_clone
                .registry
                .iter()
                .find(|record| record.plugin_type == plugin_type)
                .ok_or(MplCoreError::PluginNotFound)?;
            let plugin = Plugin::load(ctx.accounts.collection, registry_record.offset)?;
            let result = Plugin::validate_update_plugin(
                &plugin,
                &collection,
                authority,
                None,
                &registry_record.authority,
                None,
                None,
            )?;
            if result == ValidationResult::Rejected {
                return Err(MplCoreError::InvalidAuthority.into());
            }
            if result == ValidationResult::Approved {
                let plugin_data = plugin.try_to_vec()?;
                let new_plugin_data = args.plugin.try_to_vec()?;
                let plugin_size = plugin_data.len() as isize;
                let size_diff = (new_plugin_data.len() as isize)
                    .checked_sub(plugin_size)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let new_size = (ctx.accounts.collection.data_len() as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let registry_offset = plugin_header.plugin_registry_offset;
                let new_registry_offset = (registry_offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                plugin_header.plugin_registry_offset = new_registry_offset as usize;
                let next_plugin_offset = (registry_record.offset as isize)
                    .checked_add(plugin_size)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let new_next_plugin_offset = next_plugin_offset
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                let src = ctx
                    .accounts
                    .collection
                    .data
                    .borrow()[(next_plugin_offset as usize)..registry_offset]
                    .to_vec();
                resize_or_reallocate_account(
                    ctx.accounts.collection,
                    ctx.accounts.payer,
                    ctx.accounts.system_program,
                    new_size as usize,
                )?;
                sol_memcpy(
                    &mut ctx
                        .accounts
                        .collection
                        .data
                        .borrow_mut()[(new_next_plugin_offset as usize)..],
                    &src,
                    src.len(),
                );
                plugin_header.save(ctx.accounts.collection, collection.get_size())?;
                plugin_registry
                    .registry = plugin_registry
                    .registry
                    .clone()
                    .iter_mut()
                    .map(|record| {
                        let new_offset = if record.offset > registry_record.offset {
                            (record.offset as isize)
                                .checked_add(size_diff)
                                .ok_or(MplCoreError::NumericalOverflow)?
                        } else {
                            record.offset as isize
                        };
                        Ok(RegistryRecord {
                            plugin_type: record.plugin_type,
                            offset: new_offset as usize,
                            authority: record.authority,
                        })
                    })
                    .collect::<Result<Vec<_>, MplCoreError>>()?;
                plugin_registry
                    .save(ctx.accounts.collection, new_registry_offset as usize)?;
                args.plugin.save(ctx.accounts.collection, registry_record.offset)?;
            } else {
                return Err(MplCoreError::InvalidAuthority.into());
            }
            process_update_plugin()
        }
        fn process_update_plugin() -> ProgramResult {
            Ok(())
        }
    }
    pub(crate) use update_plugin::*;
    mod collect {
        use solana_program::{rent::Rent, system_program, sysvar::Sysvar};
        use super::*;
        use crate::state::{DataBlob, COLLECT_RECIPIENT};
        use crate::{
            error::MplCoreError, instruction::accounts::CollectAccounts,
            state::{Asset, HashedAsset, Key},
            utils::{fetch_core_data, load_key},
            ID,
        };
        pub(crate) fn collect<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
            let ctx = CollectAccounts::context(accounts)?;
            if *ctx.accounts.recipient.key != COLLECT_RECIPIENT {
                return Err(MplCoreError::IncorrectAccount.into());
            }
            let recipient_info = ctx.accounts.recipient;
            for account_info in ctx.remaining_accounts {
                if account_info.owner != &ID {
                    return Err(MplCoreError::IncorrectAccount.into());
                }
                collect_from_account(account_info, recipient_info)?;
            }
            Ok(())
        }
        fn collect_from_account(
            account_info: &AccountInfo,
            dest_info: &AccountInfo,
        ) -> ProgramResult {
            let rent = Rent::get()?;
            let (fee_amount, rent_amount) = match load_key(account_info, 0)? {
                Key::Uninitialized => {
                    account_info.assign(&system_program::ID);
                    (account_info.lamports(), 0)
                }
                Key::Asset => {
                    let (asset, header, registry) = fetch_core_data::<
                        Asset,
                    >(account_info)?;
                    let header_size = match header {
                        Some(header) => header.get_size(),
                        None => 0,
                    };
                    let registry_size = match registry {
                        Some(registry) => registry.get_size(),
                        None => 0,
                    };
                    let asset_rent = rent
                        .minimum_balance(asset.get_size() + header_size + registry_size);
                    let fee_amount = account_info
                        .lamports()
                        .checked_sub(asset_rent)
                        .ok_or(MplCoreError::NumericalOverflowError)?;
                    (fee_amount, asset_rent)
                }
                Key::HashedAsset => {
                    let hashed_rent = rent.minimum_balance(HashedAsset::LENGTH);
                    let fee_amount = account_info
                        .lamports()
                        .checked_sub(hashed_rent)
                        .ok_or(MplCoreError::NumericalOverflowError)?;
                    (fee_amount, hashed_rent)
                }
                _ => return Err(MplCoreError::IncorrectAccount.into()),
            };
            let dest_starting_lamports = dest_info.lamports();
            **dest_info
                .lamports
                .borrow_mut() = dest_starting_lamports
                .checked_add(fee_amount)
                .ok_or(MplCoreError::NumericalOverflowError)?;
            **account_info.lamports.borrow_mut() = rent_amount;
            Ok(())
        }
    }
    pub(crate) use collect::*;
    /// Standard processor that deserializes and instruction and routes it to the appropriate handler.
    pub fn process_instruction<'a>(
        _program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction: MplAssetInstruction = MplAssetInstruction::try_from_slice(
            instruction_data,
        )?;
        match instruction {
            MplAssetInstruction::Create(args) => {
                ::solana_program::log::sol_log("Instruction: Create");
                create(accounts, args)
            }
            MplAssetInstruction::CreateCollection(args) => {
                ::solana_program::log::sol_log("Instruction: CreateCollection");
                create_collection(accounts, args)
            }
            MplAssetInstruction::AddPlugin(args) => {
                ::solana_program::log::sol_log("Instruction: AddPlugin");
                add_plugin(accounts, args)
            }
            MplAssetInstruction::AddCollectionPlugin(args) => {
                ::solana_program::log::sol_log("Instruction: AddCollectionPlugin");
                add_collection_plugin(accounts, args)
            }
            MplAssetInstruction::RemovePlugin(args) => {
                ::solana_program::log::sol_log("Instruction: RemovePlugin");
                remove_plugin(accounts, args)
            }
            MplAssetInstruction::RemoveCollectionPlugin(args) => {
                ::solana_program::log::sol_log("Instruction: RemoveCollectionPlugin");
                remove_collection_plugin(accounts, args)
            }
            MplAssetInstruction::UpdatePlugin(args) => {
                ::solana_program::log::sol_log("Instruction: UpdatePlugin");
                update_plugin(accounts, args)
            }
            MplAssetInstruction::UpdateCollectionPlugin(args) => {
                ::solana_program::log::sol_log("Instruction: UpdateCollectionPlugin");
                update_collection_plugin(accounts, args)
            }
            MplAssetInstruction::ApprovePluginAuthority(args) => {
                ::solana_program::log::sol_log("Instruction: ApprovePluginAuthority");
                approve_plugin_authority(accounts, args)
            }
            MplAssetInstruction::ApproveCollectionPluginAuthority(args) => {
                ::solana_program::log::sol_log(
                    "Instruction: ApproveCollectionPluginAuthority",
                );
                approve_collection_plugin_authority(accounts, args)
            }
            MplAssetInstruction::RevokePluginAuthority(args) => {
                ::solana_program::log::sol_log("Instruction: RevokePluginAuthority");
                revoke_plugin_authority(accounts, args)
            }
            MplAssetInstruction::RevokeCollectionPluginAuthority(args) => {
                ::solana_program::log::sol_log(
                    "Instruction: RevokeCollectionPluginAuthority",
                );
                revoke_collection_plugin_authority(accounts, args)
            }
            MplAssetInstruction::Burn(args) => {
                ::solana_program::log::sol_log("Instruction: Burn");
                burn(accounts, args)
            }
            MplAssetInstruction::BurnCollection(args) => {
                ::solana_program::log::sol_log("Instruction: BurnCollection");
                burn_collection(accounts, args)
            }
            MplAssetInstruction::Transfer(args) => {
                ::solana_program::log::sol_log("Instruction: Transfer");
                transfer(accounts, args)
            }
            MplAssetInstruction::Update(args) => {
                ::solana_program::log::sol_log("Instruction: Update");
                update(accounts, args)
            }
            MplAssetInstruction::UpdateCollection(args) => {
                ::solana_program::log::sol_log("Instruction: UpdateCollection");
                update_collection(accounts, args)
            }
            MplAssetInstruction::Compress(args) => {
                ::solana_program::log::sol_log("Instruction: Compress");
                compress(accounts, args)
            }
            MplAssetInstruction::Decompress(args) => {
                ::solana_program::log::sol_log("Instruction: Decompress");
                decompress(accounts, args)
            }
            MplAssetInstruction::Collect => collect(accounts),
        }
    }
}
/// State and Type definitions for MPL Core.
pub mod state {
    mod asset {
        use borsh::{BorshDeserialize, BorshSerialize};
        use shank::ShankAccount;
        use solana_program::{
            account_info::AccountInfo, entrypoint::ProgramResult,
            program_error::ProgramError, pubkey::Pubkey,
        };
        use std::mem::size_of;
        use crate::{
            error::MplCoreError, plugins::{CheckResult, Plugin, ValidationResult},
            state::{Compressible, CompressionProof, DataBlob, Key, SolanaAccount},
        };
        use super::{Authority, CoreAsset, UpdateAuthority};
        /// The Core Asset structure that exists at the beginning of every asset account.
        pub struct Asset {
            /// The account discriminator.
            pub key: Key,
            /// The owner of the asset.
            pub owner: Pubkey,
            /// The update authority of the asset.
            pub update_authority: UpdateAuthority,
            /// The name of the asset.
            pub name: String,
            /// The URI of the asset that points to the off-chain data.
            pub uri: String,
            /// The sequence number used for indexing with compression.
            pub seq: Option<u64>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Asset {
            #[inline]
            fn clone(&self) -> Asset {
                Asset {
                    key: ::core::clone::Clone::clone(&self.key),
                    owner: ::core::clone::Clone::clone(&self.owner),
                    update_authority: ::core::clone::Clone::clone(
                        &self.update_authority,
                    ),
                    name: ::core::clone::Clone::clone(&self.name),
                    uri: ::core::clone::Clone::clone(&self.uri),
                    seq: ::core::clone::Clone::clone(&self.seq),
                }
            }
        }
        impl borsh::ser::BorshSerialize for Asset
        where
            Key: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            UpdateAuthority: borsh::ser::BorshSerialize,
            String: borsh::ser::BorshSerialize,
            String: borsh::ser::BorshSerialize,
            Option<u64>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.key, writer)?;
                borsh::BorshSerialize::serialize(&self.owner, writer)?;
                borsh::BorshSerialize::serialize(&self.update_authority, writer)?;
                borsh::BorshSerialize::serialize(&self.name, writer)?;
                borsh::BorshSerialize::serialize(&self.uri, writer)?;
                borsh::BorshSerialize::serialize(&self.seq, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for Asset
        where
            Key: borsh::BorshDeserialize,
            Pubkey: borsh::BorshDeserialize,
            UpdateAuthority: borsh::BorshDeserialize,
            String: borsh::BorshDeserialize,
            String: borsh::BorshDeserialize,
            Option<u64>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    key: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    owner: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    update_authority: borsh::BorshDeserialize::deserialize_reader(
                        reader,
                    )?,
                    name: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    uri: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    seq: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Asset {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "key",
                    "owner",
                    "update_authority",
                    "name",
                    "uri",
                    "seq",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.key,
                    &self.owner,
                    &self.update_authority,
                    &self.name,
                    &self.uri,
                    &&self.seq,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "Asset",
                    names,
                    values,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for Asset {}
        #[automatically_derived]
        impl ::core::cmp::Eq for Asset {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Key>;
                let _: ::core::cmp::AssertParamIsEq<Pubkey>;
                let _: ::core::cmp::AssertParamIsEq<UpdateAuthority>;
                let _: ::core::cmp::AssertParamIsEq<String>;
                let _: ::core::cmp::AssertParamIsEq<Option<u64>>;
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Asset {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Asset {
            #[inline]
            fn eq(&self, other: &Asset) -> bool {
                self.key == other.key && self.owner == other.owner
                    && self.update_authority == other.update_authority
                    && self.name == other.name && self.uri == other.uri
                    && self.seq == other.seq
            }
        }
        impl Asset {
            /// Create a new `Asset` with correct `Key` and `seq` of None.
            pub fn new(
                owner: Pubkey,
                update_authority: UpdateAuthority,
                name: String,
                uri: String,
            ) -> Self {
                Self {
                    key: Key::Asset,
                    owner,
                    update_authority,
                    name,
                    uri,
                    seq: None,
                }
            }
            /// If `asset.seq` is `Some(_)` then increment and save asset to account space.
            pub fn increment_seq_and_save(
                &mut self,
                account: &AccountInfo,
            ) -> ProgramResult {
                if let Some(seq) = &mut self.seq {
                    *seq = seq.saturating_add(1);
                    self.save(account, 0)?;
                }
                Ok(())
            }
            /// The base length of the asset account with an empty name and uri and no seq.
            pub const BASE_LENGTH: usize = 1 + 32 + 33 + 4 + 4 + 1;
            /// Check permissions for the add plugin lifecycle event.
            pub fn check_add_plugin() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the remove plugin lifecycle event.
            pub fn check_remove_plugin() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the approve plugin authority lifecycle event.
            pub fn check_approve_plugin_authority() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the revoke plugin authority lifecycle event.
            pub fn check_revoke_plugin_authority() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the transfer lifecycle event.
            pub fn check_transfer() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the burn lifecycle event.
            pub fn check_burn() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the update lifecycle event.
            pub fn check_update() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the compress lifecycle event.
            pub fn check_compress() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the decompress lifecycle event.
            pub fn check_decompress() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Validate the add plugin lifecycle event.
            pub fn validate_add_plugin(
                &self,
                authority_info: &AccountInfo,
                new_plugin: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                let new_plugin = match new_plugin {
                    Some(plugin) => plugin,
                    None => return Err(MplCoreError::InvalidPlugin.into()),
                };
                if (authority_info.key == &self.owner
                    && new_plugin.manager() == Authority::Owner)
                    || (UpdateAuthority::Address(*authority_info.key)
                        == self.update_authority
                        && new_plugin.manager() == Authority::UpdateAuthority)
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the remove plugin lifecycle event.
            pub fn validate_remove_plugin(
                &self,
                authority_info: &AccountInfo,
                _authority: &Authority,
                _plugin_to_remove: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if authority_info.key == &self.owner {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the approve plugin authority lifecycle event.
            pub fn validate_approve_plugin_authority(
                &self,
                authority_info: &AccountInfo,
                plugin: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if let Some(plugin) = plugin {
                    if (plugin.manager() == Authority::UpdateAuthority
                        && self.update_authority
                            == UpdateAuthority::Address(*authority_info.key))
                        || (plugin.manager() == Authority::Owner
                            && authority_info.key == &self.owner)
                    {
                        Ok(ValidationResult::Approved)
                    } else {
                        Ok(ValidationResult::Pass)
                    }
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the revoke plugin authority lifecycle event.
            pub fn validate_revoke_plugin_authority(
                &self,
                authority_info: &AccountInfo,
                plugin: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if let Some(plugin) = plugin {
                    if (plugin.manager() == Authority::UpdateAuthority
                        && self.update_authority
                            == UpdateAuthority::Address(*authority_info.key))
                        || (plugin.manager() == Authority::Owner
                            && authority_info.key == &self.owner)
                    {
                        Ok(ValidationResult::Approved)
                    } else {
                        Ok(ValidationResult::Pass)
                    }
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the update lifecycle event.
            pub fn validate_update(
                &self,
                authority_info: &AccountInfo,
                _: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if authority_info.key == &self.update_authority.key() {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the burn lifecycle event.
            pub fn validate_burn(
                &self,
                authority_info: &AccountInfo,
                _: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if authority_info.key == &self.owner {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the transfer lifecycle event.
            pub fn validate_transfer(
                &self,
                authority_info: &AccountInfo,
                _: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if authority_info.key == &self.owner {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the compress lifecycle event.
            pub fn validate_compress(
                &self,
                authority_info: &AccountInfo,
                _: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if authority_info.key == &self.owner {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the decompress lifecycle event.
            pub fn validate_decompress(
                &self,
                authority_info: &AccountInfo,
                _: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if authority_info.key == &self.owner {
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
                let mut size = Asset::BASE_LENGTH + self.name.len() + self.uri.len();
                if self.seq.is_some() {
                    size += size_of::<u64>();
                }
                size
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
                    seq: Some(compression_proof.seq),
                }
            }
        }
        impl CoreAsset for Asset {
            fn update_authority(&self) -> UpdateAuthority {
                self.update_authority.clone()
            }
            fn owner(&self) -> &Pubkey {
                &self.owner
            }
        }
    }
    pub use asset::*;
    mod collect {
        use solana_program::pubkey::Pubkey;
        pub(crate) const COLLECT_RECIPIENT: Pubkey = ::solana_program::pubkey::Pubkey::new_from_array([
            106u8,
            109u8,
            140u8,
            10u8,
            186u8,
            181u8,
            112u8,
            182u8,
            99u8,
            16u8,
            138u8,
            104u8,
            13u8,
            200u8,
            150u8,
            166u8,
            78u8,
            207u8,
            166u8,
            130u8,
            219u8,
            39u8,
            100u8,
            53u8,
            125u8,
            119u8,
            63u8,
            42u8,
            227u8,
            213u8,
            130u8,
            53u8,
        ]);
        pub(crate) const COLLECT_AMOUNT: u64 = 1_500_000;
    }
    pub(crate) use collect::*;
    mod collection {
        use borsh::{BorshDeserialize, BorshSerialize};
        use shank::ShankAccount;
        use solana_program::{
            account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey,
        };
        use crate::{
            error::MplCoreError, plugins::{CheckResult, Plugin, ValidationResult},
        };
        use super::{Authority, CoreAsset, DataBlob, Key, SolanaAccount, UpdateAuthority};
        /// The representation of a collection of assets.
        pub struct Collection {
            /// The account discriminator.
            pub key: Key,
            /// The update authority of the collection.
            pub update_authority: Pubkey,
            /// The name of the collection.
            pub name: String,
            /// The URI that links to what data to show for the collection.
            pub uri: String,
            /// The number of assets minted in the collection.
            pub num_minted: u32,
            /// The number of assets currently in the collection.
            pub current_size: u32,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Collection {
            #[inline]
            fn clone(&self) -> Collection {
                Collection {
                    key: ::core::clone::Clone::clone(&self.key),
                    update_authority: ::core::clone::Clone::clone(
                        &self.update_authority,
                    ),
                    name: ::core::clone::Clone::clone(&self.name),
                    uri: ::core::clone::Clone::clone(&self.uri),
                    num_minted: ::core::clone::Clone::clone(&self.num_minted),
                    current_size: ::core::clone::Clone::clone(&self.current_size),
                }
            }
        }
        impl borsh::ser::BorshSerialize for Collection
        where
            Key: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            String: borsh::ser::BorshSerialize,
            String: borsh::ser::BorshSerialize,
            u32: borsh::ser::BorshSerialize,
            u32: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.key, writer)?;
                borsh::BorshSerialize::serialize(&self.update_authority, writer)?;
                borsh::BorshSerialize::serialize(&self.name, writer)?;
                borsh::BorshSerialize::serialize(&self.uri, writer)?;
                borsh::BorshSerialize::serialize(&self.num_minted, writer)?;
                borsh::BorshSerialize::serialize(&self.current_size, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for Collection
        where
            Key: borsh::BorshDeserialize,
            Pubkey: borsh::BorshDeserialize,
            String: borsh::BorshDeserialize,
            String: borsh::BorshDeserialize,
            u32: borsh::BorshDeserialize,
            u32: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    key: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    update_authority: borsh::BorshDeserialize::deserialize_reader(
                        reader,
                    )?,
                    name: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    uri: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    num_minted: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    current_size: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Collection {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "key",
                    "update_authority",
                    "name",
                    "uri",
                    "num_minted",
                    "current_size",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.key,
                    &self.update_authority,
                    &self.name,
                    &self.uri,
                    &self.num_minted,
                    &&self.current_size,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "Collection",
                    names,
                    values,
                )
            }
        }
        impl Collection {
            /// The base length of the collection account with an empty name and uri.
            pub const BASE_LENGTH: usize = 1 + 32 + 4 + 4 + 4 + 4;
            /// Create a new collection.
            pub fn new(
                update_authority: Pubkey,
                name: String,
                uri: String,
                num_minted: u32,
                current_size: u32,
            ) -> Self {
                Self {
                    key: Key::Collection,
                    update_authority,
                    name,
                    uri,
                    num_minted,
                    current_size,
                }
            }
            /// Check permissions for the add plugin lifecycle event.
            pub fn check_add_plugin() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the remove plugin lifecycle event.
            pub fn check_remove_plugin() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the approve plugin authority lifecycle event.
            pub fn check_approve_plugin_authority() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the revoke plugin authority lifecycle event.
            pub fn check_revoke_plugin_authority() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the transfer lifecycle event.
            pub fn check_transfer() -> CheckResult {
                CheckResult::None
            }
            /// Check permissions for the burn lifecycle event.
            pub fn check_burn() -> CheckResult {
                CheckResult::None
            }
            /// Check permissions for the update lifecycle event.
            pub fn check_update() -> CheckResult {
                CheckResult::CanApprove
            }
            /// Check permissions for the compress lifecycle event.
            pub fn check_compress() -> CheckResult {
                CheckResult::None
            }
            /// Check permissions for the decompress lifecycle event.
            pub fn check_decompress() -> CheckResult {
                CheckResult::None
            }
            /// Validate the add plugin lifecycle event.
            pub fn validate_add_plugin(
                &self,
                authority_info: &AccountInfo,
                new_plugin: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                let new_plugin = match new_plugin {
                    Some(plugin) => plugin,
                    None => return Err(MplCoreError::InvalidPlugin.into()),
                };
                if *authority_info.key == self.update_authority
                    && new_plugin.manager() == Authority::UpdateAuthority
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the remove plugin lifecycle event.
            pub fn validate_remove_plugin(
                &self,
                authority_info: &AccountInfo,
                plugin_to_remove: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                let plugin_to_remove = match plugin_to_remove {
                    Some(plugin) => plugin,
                    None => return Err(MplCoreError::InvalidPlugin.into()),
                };
                if *authority_info.key == self.update_authority
                    && plugin_to_remove.manager() == Authority::UpdateAuthority
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the approve plugin authority lifecycle event.
            pub fn validate_approve_plugin_authority(
                &self,
                authority_info: &AccountInfo,
                plugin: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                let plugin = match plugin {
                    Some(plugin) => plugin,
                    None => return Err(MplCoreError::InvalidPlugin.into()),
                };
                if *authority_info.key == self.update_authority
                    && plugin.manager() == Authority::UpdateAuthority
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the revoke plugin authority lifecycle event.
            pub fn validate_revoke_plugin_authority(
                &self,
                authority_info: &AccountInfo,
                plugin: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                let plugin = match plugin {
                    Some(plugin) => plugin,
                    None => return Err(MplCoreError::InvalidPlugin.into()),
                };
                if *authority_info.key == self.update_authority
                    && plugin.manager() == Authority::UpdateAuthority
                {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the transfer lifecycle event.
            pub fn validate_transfer(
                &self,
                _authority_info: &AccountInfo,
                _: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the burn lifecycle event.
            pub fn validate_burn(
                &self,
                _authority_info: &AccountInfo,
                _: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the update lifecycle event.
            pub fn validate_update(
                &self,
                authority_info: &AccountInfo,
                _: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                if authority_info.key == &self.update_authority {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the compress lifecycle event.
            pub fn validate_compress(
                &self,
                _authority_info: &AccountInfo,
                _: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
            /// Validate the decompress lifecycle event.
            pub fn validate_decompress(
                &self,
                _authority_info: &AccountInfo,
                _: Option<&Plugin>,
            ) -> Result<ValidationResult, ProgramError> {
                Ok(ValidationResult::Pass)
            }
        }
        impl DataBlob for Collection {
            fn get_initial_size() -> usize {
                Self::BASE_LENGTH
            }
            fn get_size(&self) -> usize {
                Self::BASE_LENGTH + self.name.len() + self.uri.len()
            }
        }
        impl SolanaAccount for Collection {
            fn key() -> Key {
                Key::Collection
            }
        }
        impl CoreAsset for Collection {
            fn update_authority(&self) -> UpdateAuthority {
                UpdateAuthority::Collection(self.update_authority)
            }
            fn owner(&self) -> &Pubkey {
                &self.update_authority
            }
        }
    }
    pub use collection::*;
    mod compression_proof {
        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::pubkey::Pubkey;
        use crate::state::{Asset, HashablePluginSchema, UpdateAuthority, Wrappable};
        /// A simple struct to store the compression proof of an asset.
        #[repr(C)]
        pub struct CompressionProof {
            /// The owner of the asset.
            pub owner: Pubkey,
            /// The update authority of the asset.
            pub update_authority: UpdateAuthority,
            /// The name of the asset.
            pub name: String,
            /// The URI of the asset that points to the off-chain data.
            pub uri: String,
            /// The sequence number used for indexing with compression.
            pub seq: u64,
            /// The plugins for the asset.
            pub plugins: Vec<HashablePluginSchema>,
        }
        impl borsh::ser::BorshSerialize for CompressionProof
        where
            Pubkey: borsh::ser::BorshSerialize,
            UpdateAuthority: borsh::ser::BorshSerialize,
            String: borsh::ser::BorshSerialize,
            String: borsh::ser::BorshSerialize,
            u64: borsh::ser::BorshSerialize,
            Vec<HashablePluginSchema>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.owner, writer)?;
                borsh::BorshSerialize::serialize(&self.update_authority, writer)?;
                borsh::BorshSerialize::serialize(&self.name, writer)?;
                borsh::BorshSerialize::serialize(&self.uri, writer)?;
                borsh::BorshSerialize::serialize(&self.seq, writer)?;
                borsh::BorshSerialize::serialize(&self.plugins, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for CompressionProof
        where
            Pubkey: borsh::BorshDeserialize,
            UpdateAuthority: borsh::BorshDeserialize,
            String: borsh::BorshDeserialize,
            String: borsh::BorshDeserialize,
            u64: borsh::BorshDeserialize,
            Vec<HashablePluginSchema>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    owner: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    update_authority: borsh::BorshDeserialize::deserialize_reader(
                        reader,
                    )?,
                    name: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    uri: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    seq: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    plugins: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for CompressionProof {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for CompressionProof {
            #[inline]
            fn eq(&self, other: &CompressionProof) -> bool {
                self.owner == other.owner
                    && self.update_authority == other.update_authority
                    && self.name == other.name && self.uri == other.uri
                    && self.seq == other.seq && self.plugins == other.plugins
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for CompressionProof {}
        #[automatically_derived]
        impl ::core::cmp::Eq for CompressionProof {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Pubkey>;
                let _: ::core::cmp::AssertParamIsEq<UpdateAuthority>;
                let _: ::core::cmp::AssertParamIsEq<String>;
                let _: ::core::cmp::AssertParamIsEq<u64>;
                let _: ::core::cmp::AssertParamIsEq<Vec<HashablePluginSchema>>;
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for CompressionProof {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "owner",
                    "update_authority",
                    "name",
                    "uri",
                    "seq",
                    "plugins",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.owner,
                    &self.update_authority,
                    &self.name,
                    &self.uri,
                    &self.seq,
                    &&self.plugins,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "CompressionProof",
                    names,
                    values,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for CompressionProof {
            #[inline]
            fn clone(&self) -> CompressionProof {
                CompressionProof {
                    owner: ::core::clone::Clone::clone(&self.owner),
                    update_authority: ::core::clone::Clone::clone(
                        &self.update_authority,
                    ),
                    name: ::core::clone::Clone::clone(&self.name),
                    uri: ::core::clone::Clone::clone(&self.uri),
                    seq: ::core::clone::Clone::clone(&self.seq),
                    plugins: ::core::clone::Clone::clone(&self.plugins),
                }
            }
        }
        impl CompressionProof {
            /// Create a new `CompressionProof`.  Note this uses a passed-in `seq` rather than
            /// the one contained in `asset` to avoid errors.
            pub fn new(
                asset: Asset,
                seq: u64,
                plugins: Vec<HashablePluginSchema>,
            ) -> Self {
                Self {
                    owner: asset.owner,
                    update_authority: asset.update_authority,
                    name: asset.name,
                    uri: asset.uri,
                    seq,
                    plugins,
                }
            }
        }
        impl Wrappable for CompressionProof {}
    }
    pub use compression_proof::*;
    mod hashable_plugin_schema {
        use borsh::{BorshDeserialize, BorshSerialize};
        use std::cmp::Ordering;
        use crate::{plugins::Plugin, state::{Authority, Compressible}};
        /// A type that stores a plugin's authority and deserialized data into a
        /// schema that will be later hashed into a hashed asset.  Also used in
        /// `CompressionProof`.
        pub struct HashablePluginSchema {
            /// This is the order the plugins are stored in the account, allowing us
            /// to keep track of their order in the hashing.
            pub index: usize,
            /// The authority who has permission to utilize a plugin.
            pub authority: Authority,
            /// The deserialized plugin.
            pub plugin: Plugin,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for HashablePluginSchema {
            #[inline]
            fn clone(&self) -> HashablePluginSchema {
                HashablePluginSchema {
                    index: ::core::clone::Clone::clone(&self.index),
                    authority: ::core::clone::Clone::clone(&self.authority),
                    plugin: ::core::clone::Clone::clone(&self.plugin),
                }
            }
        }
        impl borsh::ser::BorshSerialize for HashablePluginSchema
        where
            usize: borsh::ser::BorshSerialize,
            Authority: borsh::ser::BorshSerialize,
            Plugin: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.index, writer)?;
                borsh::BorshSerialize::serialize(&self.authority, writer)?;
                borsh::BorshSerialize::serialize(&self.plugin, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for HashablePluginSchema
        where
            usize: borsh::BorshDeserialize,
            Authority: borsh::BorshDeserialize,
            Plugin: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    index: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    authority: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    plugin: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for HashablePluginSchema {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "HashablePluginSchema",
                    "index",
                    &self.index,
                    "authority",
                    &self.authority,
                    "plugin",
                    &&self.plugin,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for HashablePluginSchema {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for HashablePluginSchema {
            #[inline]
            fn eq(&self, other: &HashablePluginSchema) -> bool {
                self.index == other.index && self.authority == other.authority
                    && self.plugin == other.plugin
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for HashablePluginSchema {}
        #[automatically_derived]
        impl ::core::cmp::Eq for HashablePluginSchema {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<Authority>;
                let _: ::core::cmp::AssertParamIsEq<Plugin>;
            }
        }
        impl HashablePluginSchema {
            /// Associated function for sorting `RegistryRecords` by offset.
            pub fn compare_indeces(
                a: &HashablePluginSchema,
                b: &HashablePluginSchema,
            ) -> Ordering {
                a.index.cmp(&b.index)
            }
        }
        impl Compressible for HashablePluginSchema {}
    }
    pub use hashable_plugin_schema::*;
    mod hashed_asset_schema {
        use borsh::{BorshDeserialize, BorshSerialize};
        use crate::state::Compressible;
        /// The hashed asset schema is a schema that contains a hash of the asset and a vec of plugin hashes.
        pub struct HashedAssetSchema {
            /// The hash of the asset.
            pub asset_hash: [u8; 32],
            /// A vec of plugin hashes.
            pub plugin_hashes: Vec<[u8; 32]>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for HashedAssetSchema {
            #[inline]
            fn clone(&self) -> HashedAssetSchema {
                HashedAssetSchema {
                    asset_hash: ::core::clone::Clone::clone(&self.asset_hash),
                    plugin_hashes: ::core::clone::Clone::clone(&self.plugin_hashes),
                }
            }
        }
        impl borsh::ser::BorshSerialize for HashedAssetSchema
        where
            [u8; 32]: borsh::ser::BorshSerialize,
            Vec<[u8; 32]>: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.asset_hash, writer)?;
                borsh::BorshSerialize::serialize(&self.plugin_hashes, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for HashedAssetSchema
        where
            [u8; 32]: borsh::BorshDeserialize,
            Vec<[u8; 32]>: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    asset_hash: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    plugin_hashes: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for HashedAssetSchema {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "HashedAssetSchema",
                    "asset_hash",
                    &self.asset_hash,
                    "plugin_hashes",
                    &&self.plugin_hashes,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for HashedAssetSchema {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for HashedAssetSchema {
            #[inline]
            fn eq(&self, other: &HashedAssetSchema) -> bool {
                self.asset_hash == other.asset_hash
                    && self.plugin_hashes == other.plugin_hashes
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for HashedAssetSchema {}
        #[automatically_derived]
        impl ::core::cmp::Eq for HashedAssetSchema {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<[u8; 32]>;
                let _: ::core::cmp::AssertParamIsEq<Vec<[u8; 32]>>;
            }
        }
        impl Compressible for HashedAssetSchema {}
    }
    pub use hashed_asset_schema::*;
    mod hashed_asset {
        use borsh::{BorshDeserialize, BorshSerialize};
        use shank::ShankAccount;
        use crate::state::{DataBlob, Key, SolanaAccount};
        /// The structure representing the hash of the asset.
        pub struct HashedAsset {
            /// The account discriminator.
            pub key: Key,
            /// The hash of the asset content.
            pub hash: [u8; 32],
        }
        #[automatically_derived]
        impl ::core::clone::Clone for HashedAsset {
            #[inline]
            fn clone(&self) -> HashedAsset {
                HashedAsset {
                    key: ::core::clone::Clone::clone(&self.key),
                    hash: ::core::clone::Clone::clone(&self.hash),
                }
            }
        }
        impl borsh::ser::BorshSerialize for HashedAsset
        where
            Key: borsh::ser::BorshSerialize,
            [u8; 32]: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.key, writer)?;
                borsh::BorshSerialize::serialize(&self.hash, writer)?;
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for HashedAsset
        where
            Key: borsh::BorshDeserialize,
            [u8; 32]: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                Ok(Self {
                    key: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    hash: borsh::BorshDeserialize::deserialize_reader(reader)?,
                })
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for HashedAsset {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "HashedAsset",
                    "key",
                    &self.key,
                    "hash",
                    &&self.hash,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for HashedAsset {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for HashedAsset {
            #[inline]
            fn eq(&self, other: &HashedAsset) -> bool {
                self.key == other.key && self.hash == other.hash
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for HashedAsset {}
        #[automatically_derived]
        impl ::core::cmp::Eq for HashedAsset {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Key>;
                let _: ::core::cmp::AssertParamIsEq<[u8; 32]>;
            }
        }
        impl HashedAsset {
            /// The length of the hashed asset account.
            pub const LENGTH: usize = 1 + 32;
            /// Create a new hashed asset.
            pub fn new(hash: [u8; 32]) -> Self {
                Self {
                    key: Key::HashedAsset,
                    hash,
                }
            }
        }
        impl DataBlob for HashedAsset {
            fn get_initial_size() -> usize {
                HashedAsset::LENGTH
            }
            fn get_size(&self) -> usize {
                HashedAsset::LENGTH
            }
        }
        impl SolanaAccount for HashedAsset {
            fn key() -> Key {
                Key::HashedAsset
            }
        }
    }
    pub use hashed_asset::*;
    mod traits {
        use crate::{error::MplCoreError, state::Key, utils::load_key};
        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::{
            account_info::AccountInfo, entrypoint::ProgramResult, keccak, msg,
            program::invoke, program_error::ProgramError, pubkey::Pubkey,
        };
        use super::UpdateAuthority;
        /// A trait for generic blobs of data that have size.
        pub trait DataBlob: BorshSerialize + BorshDeserialize {
            /// Get the size of an empty instance of the data blob.
            fn get_initial_size() -> usize;
            /// Get the current size of the data blob.
            fn get_size(&self) -> usize;
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
                let mut bytes: &[u8] = &(*account.data).borrow()[offset..];
                Self::deserialize(&mut bytes)
                    .map_err(|error| {
                        ::solana_program::log::sol_log(
                            &{
                                let res = ::alloc::fmt::format(
                                    format_args!("Error: {0}", error),
                                );
                                res
                            },
                        );
                        MplCoreError::DeserializationError.into()
                    })
            }
            /// Save the account to the given account info starting at the offset.
            fn save(&self, account: &AccountInfo, offset: usize) -> ProgramResult {
                borsh::to_writer(&mut account.data.borrow_mut()[offset..], self)
                    .map_err(|error| {
                        ::solana_program::log::sol_log(
                            &{
                                let res = ::alloc::fmt::format(
                                    format_args!("Error: {0}", error),
                                );
                                res
                            },
                        );
                        MplCoreError::SerializationError.into()
                    })
            }
        }
        /// A trait for data that can be compressed.
        pub trait Compressible: BorshSerialize + BorshDeserialize {
            /// Get the hash of the compressed data.
            fn hash(&self) -> Result<[u8; 32], ProgramError> {
                let serialized_data = self.try_to_vec()?;
                Ok(keccak::hash(serialized_data.as_slice()).to_bytes())
            }
        }
        /// A trait for data that can be wrapped by the spl-noop program.
        pub trait Wrappable: BorshSerialize + BorshDeserialize {
            /// Write the data to ledger state by wrapping it in a noop instruction.
            fn wrap(&self) -> ProgramResult {
                let serialized_data = self.try_to_vec()?;
                invoke(&spl_noop::instruction(serialized_data), &[])
            }
        }
        /// A trait for core assets.
        pub trait CoreAsset {
            /// Get the update authority of the asset.
            fn update_authority(&self) -> UpdateAuthority;
            /// Get the owner of the asset.
            fn owner(&self) -> &Pubkey;
        }
    }
    pub use traits::*;
    mod update_authority {
        use borsh::{BorshDeserialize, BorshSerialize};
        use mpl_utils::assert_signer;
        use solana_program::{program_error::ProgramError, pubkey::Pubkey};
        use crate::{
            error::MplCoreError,
            instruction::accounts::{
                BurnAccounts, CompressAccounts, CreateAccounts, DecompressAccounts,
                TransferAccounts, UpdateAccounts,
            },
            plugins::{
                fetch_plugin, CheckResult, PluginType, UpdateDelegate, ValidationResult,
            },
            processor::CreateArgs, state::{Authority, Collection, SolanaAccount},
            utils::assert_collection_authority,
        };
        /// An enum representing the types of accounts that can update data on an asset.
        pub enum UpdateAuthority {
            /// No update authority, used for immutability.
            None,
            /// A standard address or PDA.
            Address(Pubkey),
            /// Authority delegated to a collection.
            Collection(Pubkey),
        }
        #[automatically_derived]
        impl ::core::clone::Clone for UpdateAuthority {
            #[inline]
            fn clone(&self) -> UpdateAuthority {
                match self {
                    UpdateAuthority::None => UpdateAuthority::None,
                    UpdateAuthority::Address(__self_0) => {
                        UpdateAuthority::Address(::core::clone::Clone::clone(__self_0))
                    }
                    UpdateAuthority::Collection(__self_0) => {
                        UpdateAuthority::Collection(
                            ::core::clone::Clone::clone(__self_0),
                        )
                    }
                }
            }
        }
        impl borsh::ser::BorshSerialize for UpdateAuthority
        where
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                let variant_idx: u8 = match self {
                    UpdateAuthority::None => 0u8,
                    UpdateAuthority::Address(..) => 1u8,
                    UpdateAuthority::Collection(..) => 2u8,
                };
                writer.write_all(&variant_idx.to_le_bytes())?;
                match self {
                    UpdateAuthority::None => {}
                    UpdateAuthority::Address(id0) => {
                        borsh::BorshSerialize::serialize(id0, writer)?;
                    }
                    UpdateAuthority::Collection(id0) => {
                        borsh::BorshSerialize::serialize(id0, writer)?;
                    }
                }
                Ok(())
            }
        }
        impl borsh::de::BorshDeserialize for UpdateAuthority
        where
            Pubkey: borsh::BorshDeserialize,
            Pubkey: borsh::BorshDeserialize,
        {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                let tag = <u8 as borsh::de::BorshDeserialize>::deserialize_reader(
                    reader,
                )?;
                <Self as borsh::de::EnumExt>::deserialize_variant(reader, tag)
            }
        }
        impl borsh::de::EnumExt for UpdateAuthority
        where
            Pubkey: borsh::BorshDeserialize,
            Pubkey: borsh::BorshDeserialize,
        {
            fn deserialize_variant<R: borsh::maybestd::io::Read>(
                reader: &mut R,
                variant_idx: u8,
            ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                let mut return_value = match variant_idx {
                    0u8 => UpdateAuthority::None,
                    1u8 => {
                        UpdateAuthority::Address(
                            borsh::BorshDeserialize::deserialize_reader(reader)?,
                        )
                    }
                    2u8 => {
                        UpdateAuthority::Collection(
                            borsh::BorshDeserialize::deserialize_reader(reader)?,
                        )
                    }
                    _ => {
                        return Err(
                            borsh::maybestd::io::Error::new(
                                borsh::maybestd::io::ErrorKind::InvalidInput,
                                {
                                    let res = ::alloc::fmt::format(
                                        format_args!("Unexpected variant index: {0:?}", variant_idx),
                                    );
                                    res
                                },
                            ),
                        );
                    }
                };
                Ok(return_value)
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for UpdateAuthority {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    UpdateAuthority::None => ::core::fmt::Formatter::write_str(f, "None"),
                    UpdateAuthority::Address(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Address",
                            &__self_0,
                        )
                    }
                    UpdateAuthority::Collection(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Collection",
                            &__self_0,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for UpdateAuthority {}
        #[automatically_derived]
        impl ::core::cmp::Eq for UpdateAuthority {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Pubkey>;
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for UpdateAuthority {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for UpdateAuthority {
            #[inline]
            fn eq(&self, other: &UpdateAuthority) -> bool {
                let __self_tag = ::core::intrinsics::discriminant_value(self);
                let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                __self_tag == __arg1_tag
                    && match (self, other) {
                        (
                            UpdateAuthority::Address(__self_0),
                            UpdateAuthority::Address(__arg1_0),
                        ) => *__self_0 == *__arg1_0,
                        (
                            UpdateAuthority::Collection(__self_0),
                            UpdateAuthority::Collection(__arg1_0),
                        ) => *__self_0 == *__arg1_0,
                        _ => true,
                    }
            }
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
                ctx: &CreateAccounts,
                _args: &CreateArgs,
            ) -> Result<ValidationResult, ProgramError> {
                match (ctx.collection, self) {
                    (
                        Some(collection_info),
                        UpdateAuthority::Collection(collection_address),
                    ) => {
                        if collection_info.key != collection_address {
                            return Err(MplCoreError::InvalidCollection.into());
                        }
                        let collection = Collection::load(collection_info, 0)?;
                        ::solana_program::log::sol_log(
                            &{
                                let res = ::alloc::fmt::format(
                                    format_args!("Collection: {0:?}", collection),
                                );
                                res
                            },
                        );
                        let authority_info = match ctx.authority {
                            Some(authority) => {
                                assert_signer(authority)?;
                                authority
                            }
                            None => ctx.payer,
                        };
                        let maybe_update_delegate = fetch_plugin::<
                            Collection,
                            UpdateDelegate,
                        >(collection_info, PluginType::UpdateDelegate);
                        if let Ok((authority, _, _)) = maybe_update_delegate {
                            if assert_collection_authority(
                                    &collection,
                                    authority_info,
                                    &authority,
                                )
                                .is_err()
                                && assert_collection_authority(
                                        &collection,
                                        authority_info,
                                        &Authority::UpdateAuthority,
                                    )
                                    .is_err()
                            {
                                return Ok(ValidationResult::Rejected);
                            }
                        } else if authority_info.key != &collection.update_authority {
                            return Ok(ValidationResult::Rejected);
                        }
                        Ok(ValidationResult::Pass)
                    }
                    (_, UpdateAuthority::Address(_)) => Ok(ValidationResult::Pass),
                    _ => Ok(ValidationResult::Rejected),
                }
            }
            /// Validate the update lifecycle event.
            pub fn validate_update(
                &self,
                ctx: &UpdateAccounts,
            ) -> Result<ValidationResult, ProgramError> {
                let authority = match self {
                    Self::None => return Ok(ValidationResult::Rejected),
                    Self::Address(address) => address,
                    Self::Collection(address) => address,
                };
                if ctx.authority.unwrap_or(ctx.payer).key == authority {
                    Ok(ValidationResult::Approved)
                } else {
                    Ok(ValidationResult::Pass)
                }
            }
            /// Validate the burn lifecycle event.
            pub fn validate_burn(
                &self,
                _ctx: &BurnAccounts,
            ) -> Result<ValidationResult, ProgramError> {
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
    }
    pub use update_authority::*;
    use borsh::{BorshDeserialize, BorshSerialize};
    use num_derive::{FromPrimitive, ToPrimitive};
    use solana_program::pubkey::Pubkey;
    /// An enum representing the two types of data, compressed (stored in ledger) and uncompressed (stored in account state).
    #[repr(C)]
    pub enum DataState {
        /// The data is stored in account state.
        AccountState,
        /// The data is stored in the ledger history (compressed).
        LedgerState,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for DataState {
        #[inline]
        fn clone(&self) -> DataState {
            match self {
                DataState::AccountState => DataState::AccountState,
                DataState::LedgerState => DataState::LedgerState,
            }
        }
    }
    impl borsh::ser::BorshSerialize for DataState {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            let variant_idx: u8 = match self {
                DataState::AccountState => 0u8,
                DataState::LedgerState => 1u8,
            };
            writer.write_all(&variant_idx.to_le_bytes())?;
            match self {
                DataState::AccountState => {}
                DataState::LedgerState => {}
            }
            Ok(())
        }
    }
    impl borsh::de::BorshDeserialize for DataState {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let tag = <u8 as borsh::de::BorshDeserialize>::deserialize_reader(reader)?;
            <Self as borsh::de::EnumExt>::deserialize_variant(reader, tag)
        }
    }
    impl borsh::de::EnumExt for DataState {
        fn deserialize_variant<R: borsh::maybestd::io::Read>(
            reader: &mut R,
            variant_idx: u8,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let mut return_value = match variant_idx {
                0u8 => DataState::AccountState,
                1u8 => DataState::LedgerState,
                _ => {
                    return Err(
                        borsh::maybestd::io::Error::new(
                            borsh::maybestd::io::ErrorKind::InvalidInput,
                            {
                                let res = ::alloc::fmt::format(
                                    format_args!("Unexpected variant index: {0:?}", variant_idx),
                                );
                                res
                            },
                        ),
                    );
                }
            };
            Ok(return_value)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for DataState {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    DataState::AccountState => "AccountState",
                    DataState::LedgerState => "LedgerState",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for DataState {}
    #[automatically_derived]
    impl ::core::cmp::Eq for DataState {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for DataState {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for DataState {
        #[inline]
        fn eq(&self, other: &DataState) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    /// Variants representing the different types of authority that can have permissions over plugins.
    #[repr(u8)]
    pub enum Authority {
        /// No authority, used for immutability.
        None,
        /// The owner of the core asset.
        Owner,
        /// The update authority of the core asset.
        UpdateAuthority,
        /// A pubkey that is the authority over a plugin.
        Pubkey {
            /// The address of the authority.
            address: Pubkey,
        },
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Authority {}
    #[automatically_derived]
    impl ::core::clone::Clone for Authority {
        #[inline]
        fn clone(&self) -> Authority {
            let _: ::core::clone::AssertParamIsClone<Pubkey>;
            *self
        }
    }
    impl borsh::ser::BorshSerialize for Authority
    where
        Pubkey: borsh::ser::BorshSerialize,
    {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            let variant_idx: u8 = match self {
                Authority::None => 0u8,
                Authority::Owner => 1u8,
                Authority::UpdateAuthority => 2u8,
                Authority::Pubkey { .. } => 3u8,
            };
            writer.write_all(&variant_idx.to_le_bytes())?;
            match self {
                Authority::None => {}
                Authority::Owner => {}
                Authority::UpdateAuthority => {}
                Authority::Pubkey { address } => {
                    borsh::BorshSerialize::serialize(address, writer)?;
                }
            }
            Ok(())
        }
    }
    impl borsh::de::BorshDeserialize for Authority
    where
        Pubkey: borsh::BorshDeserialize,
    {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let tag = <u8 as borsh::de::BorshDeserialize>::deserialize_reader(reader)?;
            <Self as borsh::de::EnumExt>::deserialize_variant(reader, tag)
        }
    }
    impl borsh::de::EnumExt for Authority
    where
        Pubkey: borsh::BorshDeserialize,
    {
        fn deserialize_variant<R: borsh::maybestd::io::Read>(
            reader: &mut R,
            variant_idx: u8,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let mut return_value = match variant_idx {
                0u8 => Authority::None,
                1u8 => Authority::Owner,
                2u8 => Authority::UpdateAuthority,
                3u8 => {
                    Authority::Pubkey {
                        address: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    }
                }
                _ => {
                    return Err(
                        borsh::maybestd::io::Error::new(
                            borsh::maybestd::io::ErrorKind::InvalidInput,
                            {
                                let res = ::alloc::fmt::format(
                                    format_args!("Unexpected variant index: {0:?}", variant_idx),
                                );
                                res
                            },
                        ),
                    );
                }
            };
            Ok(return_value)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Authority {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Authority::None => ::core::fmt::Formatter::write_str(f, "None"),
                Authority::Owner => ::core::fmt::Formatter::write_str(f, "Owner"),
                Authority::UpdateAuthority => {
                    ::core::fmt::Formatter::write_str(f, "UpdateAuthority")
                }
                Authority::Pubkey { address: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "Pubkey",
                        "address",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Authority {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Authority {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Pubkey>;
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Authority {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Authority {
        #[inline]
        fn eq(&self, other: &Authority) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (
                        Authority::Pubkey { address: __self_0 },
                        Authority::Pubkey { address: __arg1_0 },
                    ) => *__self_0 == *__arg1_0,
                    _ => true,
                }
        }
    }
    /// Different types of extra accounts that can be passed in for lifecycle hooks.
    #[repr(C)]
    pub enum ExtraAccounts {
        /// No extra accounts.
        None,
        /// Compatible with spl-token-2022 transfer hooks.
        SplHook {
            /// An account meta accounts derived from the account pubkey.
            extra_account_metas: Pubkey,
        },
        /// A simpler method of passing in extra accounts using deterministic PDAs.
        MplHook {
            /// The PDA derived from the account pubkey.
            mint_pda: Option<Pubkey>,
            /// The PDA derived from the collection pubkey.
            collection_pda: Option<Pubkey>,
            /// The PDA derived from the asset owner pubkey.
            owner_pda: Option<Pubkey>,
        },
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ExtraAccounts {
        #[inline]
        fn clone(&self) -> ExtraAccounts {
            match self {
                ExtraAccounts::None => ExtraAccounts::None,
                ExtraAccounts::SplHook { extra_account_metas: __self_0 } => {
                    ExtraAccounts::SplHook {
                        extra_account_metas: ::core::clone::Clone::clone(__self_0),
                    }
                }
                ExtraAccounts::MplHook {
                    mint_pda: __self_0,
                    collection_pda: __self_1,
                    owner_pda: __self_2,
                } => {
                    ExtraAccounts::MplHook {
                        mint_pda: ::core::clone::Clone::clone(__self_0),
                        collection_pda: ::core::clone::Clone::clone(__self_1),
                        owner_pda: ::core::clone::Clone::clone(__self_2),
                    }
                }
            }
        }
    }
    impl borsh::ser::BorshSerialize for ExtraAccounts
    where
        Pubkey: borsh::ser::BorshSerialize,
        Option<Pubkey>: borsh::ser::BorshSerialize,
        Option<Pubkey>: borsh::ser::BorshSerialize,
        Option<Pubkey>: borsh::ser::BorshSerialize,
    {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            let variant_idx: u8 = match self {
                ExtraAccounts::None => 0u8,
                ExtraAccounts::SplHook { .. } => 1u8,
                ExtraAccounts::MplHook { .. } => 2u8,
            };
            writer.write_all(&variant_idx.to_le_bytes())?;
            match self {
                ExtraAccounts::None => {}
                ExtraAccounts::SplHook { extra_account_metas } => {
                    borsh::BorshSerialize::serialize(extra_account_metas, writer)?;
                }
                ExtraAccounts::MplHook { mint_pda, collection_pda, owner_pda } => {
                    borsh::BorshSerialize::serialize(mint_pda, writer)?;
                    borsh::BorshSerialize::serialize(collection_pda, writer)?;
                    borsh::BorshSerialize::serialize(owner_pda, writer)?;
                }
            }
            Ok(())
        }
    }
    impl borsh::de::BorshDeserialize for ExtraAccounts
    where
        Pubkey: borsh::BorshDeserialize,
        Option<Pubkey>: borsh::BorshDeserialize,
        Option<Pubkey>: borsh::BorshDeserialize,
        Option<Pubkey>: borsh::BorshDeserialize,
    {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let tag = <u8 as borsh::de::BorshDeserialize>::deserialize_reader(reader)?;
            <Self as borsh::de::EnumExt>::deserialize_variant(reader, tag)
        }
    }
    impl borsh::de::EnumExt for ExtraAccounts
    where
        Pubkey: borsh::BorshDeserialize,
        Option<Pubkey>: borsh::BorshDeserialize,
        Option<Pubkey>: borsh::BorshDeserialize,
        Option<Pubkey>: borsh::BorshDeserialize,
    {
        fn deserialize_variant<R: borsh::maybestd::io::Read>(
            reader: &mut R,
            variant_idx: u8,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let mut return_value = match variant_idx {
                0u8 => ExtraAccounts::None,
                1u8 => {
                    ExtraAccounts::SplHook {
                        extra_account_metas: borsh::BorshDeserialize::deserialize_reader(
                            reader,
                        )?,
                    }
                }
                2u8 => {
                    ExtraAccounts::MplHook {
                        mint_pda: borsh::BorshDeserialize::deserialize_reader(reader)?,
                        collection_pda: borsh::BorshDeserialize::deserialize_reader(
                            reader,
                        )?,
                        owner_pda: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    }
                }
                _ => {
                    return Err(
                        borsh::maybestd::io::Error::new(
                            borsh::maybestd::io::ErrorKind::InvalidInput,
                            {
                                let res = ::alloc::fmt::format(
                                    format_args!("Unexpected variant index: {0:?}", variant_idx),
                                );
                                res
                            },
                        ),
                    );
                }
            };
            Ok(return_value)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ExtraAccounts {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                ExtraAccounts::None => ::core::fmt::Formatter::write_str(f, "None"),
                ExtraAccounts::SplHook { extra_account_metas: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "SplHook",
                        "extra_account_metas",
                        &__self_0,
                    )
                }
                ExtraAccounts::MplHook {
                    mint_pda: __self_0,
                    collection_pda: __self_1,
                    owner_pda: __self_2,
                } => {
                    ::core::fmt::Formatter::debug_struct_field3_finish(
                        f,
                        "MplHook",
                        "mint_pda",
                        __self_0,
                        "collection_pda",
                        __self_1,
                        "owner_pda",
                        &__self_2,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for ExtraAccounts {}
    #[automatically_derived]
    impl ::core::cmp::Eq for ExtraAccounts {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Pubkey>;
            let _: ::core::cmp::AssertParamIsEq<Option<Pubkey>>;
            let _: ::core::cmp::AssertParamIsEq<Option<Pubkey>>;
            let _: ::core::cmp::AssertParamIsEq<Option<Pubkey>>;
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ExtraAccounts {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ExtraAccounts {
        #[inline]
        fn eq(&self, other: &ExtraAccounts) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (
                        ExtraAccounts::SplHook { extra_account_metas: __self_0 },
                        ExtraAccounts::SplHook { extra_account_metas: __arg1_0 },
                    ) => *__self_0 == *__arg1_0,
                    (
                        ExtraAccounts::MplHook {
                            mint_pda: __self_0,
                            collection_pda: __self_1,
                            owner_pda: __self_2,
                        },
                        ExtraAccounts::MplHook {
                            mint_pda: __arg1_0,
                            collection_pda: __arg1_1,
                            owner_pda: __arg1_2,
                        },
                    ) => {
                        *__self_0 == *__arg1_0 && *__self_1 == *__arg1_1
                            && *__self_2 == *__arg1_2
                    }
                    _ => true,
                }
        }
    }
    /// An enum representing account discriminators.
    pub enum Key {
        /// Uninitialized or invalid account.
        Uninitialized,
        /// An account holding an uncompressed asset.
        Asset,
        /// An account holding a compressed asset.
        HashedAsset,
        /// A discriminator indicating the plugin header.
        PluginHeader,
        /// A discriminator indicating the plugin registry.
        PluginRegistry,
        /// A discriminator indicating the collection.
        Collection,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Key {
        #[inline]
        fn clone(&self) -> Key {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Key {}
    impl borsh::ser::BorshSerialize for Key {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            let variant_idx: u8 = match self {
                Key::Uninitialized => 0u8,
                Key::Asset => 1u8,
                Key::HashedAsset => 2u8,
                Key::PluginHeader => 3u8,
                Key::PluginRegistry => 4u8,
                Key::Collection => 5u8,
            };
            writer.write_all(&variant_idx.to_le_bytes())?;
            match self {
                Key::Uninitialized => {}
                Key::Asset => {}
                Key::HashedAsset => {}
                Key::PluginHeader => {}
                Key::PluginRegistry => {}
                Key::Collection => {}
            }
            Ok(())
        }
    }
    impl borsh::de::BorshDeserialize for Key {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let tag = <u8 as borsh::de::BorshDeserialize>::deserialize_reader(reader)?;
            <Self as borsh::de::EnumExt>::deserialize_variant(reader, tag)
        }
    }
    impl borsh::de::EnumExt for Key {
        fn deserialize_variant<R: borsh::maybestd::io::Read>(
            reader: &mut R,
            variant_idx: u8,
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            let mut return_value = match variant_idx {
                0u8 => Key::Uninitialized,
                1u8 => Key::Asset,
                2u8 => Key::HashedAsset,
                3u8 => Key::PluginHeader,
                4u8 => Key::PluginRegistry,
                5u8 => Key::Collection,
                _ => {
                    return Err(
                        borsh::maybestd::io::Error::new(
                            borsh::maybestd::io::ErrorKind::InvalidInput,
                            {
                                let res = ::alloc::fmt::format(
                                    format_args!("Unexpected variant index: {0:?}", variant_idx),
                                );
                                res
                            },
                        ),
                    );
                }
            };
            Ok(return_value)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Key {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    Key::Uninitialized => "Uninitialized",
                    Key::Asset => "Asset",
                    Key::HashedAsset => "HashedAsset",
                    Key::PluginHeader => "PluginHeader",
                    Key::PluginRegistry => "PluginRegistry",
                    Key::Collection => "Collection",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Key {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Key {
        #[inline]
        fn eq(&self, other: &Key) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Key {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Key {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[allow(non_upper_case_globals, unused_qualifications)]
    const _IMPL_NUM_ToPrimitive_FOR_Key: () = {
        #[allow(clippy::useless_attribute)]
        #[allow(rust_2018_idioms)]
        extern crate num_traits as _num_traits;
        impl _num_traits::ToPrimitive for Key {
            #[inline]
            #[allow(trivial_numeric_casts)]
            fn to_i64(&self) -> Option<i64> {
                Some(
                    match *self {
                        Key::Uninitialized => Key::Uninitialized as i64,
                        Key::Asset => Key::Asset as i64,
                        Key::HashedAsset => Key::HashedAsset as i64,
                        Key::PluginHeader => Key::PluginHeader as i64,
                        Key::PluginRegistry => Key::PluginRegistry as i64,
                        Key::Collection => Key::Collection as i64,
                    },
                )
            }
            #[inline]
            fn to_u64(&self) -> Option<u64> {
                self.to_i64().map(|x| x as u64)
            }
        }
    };
    #[allow(non_upper_case_globals, unused_qualifications)]
    const _IMPL_NUM_FromPrimitive_FOR_Key: () = {
        #[allow(clippy::useless_attribute)]
        #[allow(rust_2018_idioms)]
        extern crate num_traits as _num_traits;
        impl _num_traits::FromPrimitive for Key {
            #[allow(trivial_numeric_casts)]
            #[inline]
            fn from_i64(n: i64) -> Option<Self> {
                if n == Key::Uninitialized as i64 {
                    Some(Key::Uninitialized)
                } else if n == Key::Asset as i64 {
                    Some(Key::Asset)
                } else if n == Key::HashedAsset as i64 {
                    Some(Key::HashedAsset)
                } else if n == Key::PluginHeader as i64 {
                    Some(Key::PluginHeader)
                } else if n == Key::PluginRegistry as i64 {
                    Some(Key::PluginRegistry)
                } else if n == Key::Collection as i64 {
                    Some(Key::Collection)
                } else {
                    None
                }
            }
            #[inline]
            fn from_u64(n: u64) -> Option<Self> {
                Self::from_i64(n as i64)
            }
        }
    };
    impl Key {
        /// Get the size of the Key.
        pub fn get_initial_size() -> usize {
            1
        }
    }
}
/// Program-wide utility functions.
pub mod utils {
    use std::collections::BTreeMap;
    use borsh::{BorshDeserialize, BorshSerialize};
    use mpl_utils::assert_signer;
    use num_traits::{FromPrimitive, ToPrimitive};
    use solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
        program_error::ProgramError, program_memory::sol_memcpy, rent::Rent,
        system_instruction, sysvar::Sysvar,
    };
    use crate::{
        error::MplCoreError,
        plugins::{
            create_meta_idempotent, initialize_plugin, validate_plugin_checks,
            CheckResult, Plugin, PluginHeader, PluginRegistry, PluginType,
            RegistryRecord, ValidationResult,
        },
        state::{
            Asset, Authority, Collection, Compressible, CompressionProof, CoreAsset,
            DataBlob, HashablePluginSchema, HashedAsset, HashedAssetSchema, Key,
            SolanaAccount, UpdateAuthority,
        },
    };
    /// Load the one byte key from the account data at the given offset.
    pub fn load_key(account: &AccountInfo, offset: usize) -> Result<Key, ProgramError> {
        let key = Key::from_u8((*account.data).borrow()[offset])
            .ok_or(MplCoreError::DeserializationError)?;
        Ok(key)
    }
    /// Assert that the account info address is in the same as the authority.
    pub fn assert_authority<T: CoreAsset>(
        asset: &T,
        authority_info: &AccountInfo,
        authority: &Authority,
    ) -> ProgramResult {
        ::solana_program::log::sol_log(
            &{
                let res = ::alloc::fmt::format(
                    format_args!("Update authority: {0:?}", asset.update_authority()),
                );
                res
            },
        );
        ::solana_program::log::sol_log(
            &{
                let res = ::alloc::fmt::format(
                    format_args!(
                        "Check if {0:?} matches {1:?}",
                        authority_info.key,
                        authority,
                    ),
                );
                res
            },
        );
        match authority {
            Authority::None => {}
            Authority::Owner => {
                if asset.owner() == authority_info.key {
                    return Ok(());
                }
            }
            Authority::UpdateAuthority => {
                if asset.update_authority().key() == *authority_info.key {
                    return Ok(());
                }
            }
            Authority::Pubkey { address } => {
                if authority_info.key == address {
                    return Ok(());
                }
            }
        }
        Err(MplCoreError::InvalidAuthority.into())
    }
    /// Assert that the account info address is the same as the authority.
    pub fn assert_collection_authority(
        asset: &Collection,
        authority_info: &AccountInfo,
        authority: &Authority,
    ) -> ProgramResult {
        match authority {
            Authority::None | Authority::Owner => {}
            Authority::UpdateAuthority => {
                if &asset.update_authority == authority_info.key {
                    return Ok(());
                }
            }
            Authority::Pubkey { address } => {
                if authority_info.key == address {
                    return Ok(());
                }
            }
        }
        Err(MplCoreError::InvalidAuthority.into())
    }
    /// Fetch the core data from the account; asset, plugin header (if present), and plugin registry (if present).
    pub fn fetch_core_data<T: DataBlob + SolanaAccount>(
        account: &AccountInfo,
    ) -> Result<(T, Option<PluginHeader>, Option<PluginRegistry>), ProgramError> {
        let asset = T::load(account, 0)?;
        if asset.get_size() != account.data_len() {
            let plugin_header = PluginHeader::load(account, asset.get_size())?;
            let plugin_registry = PluginRegistry::load(
                account,
                plugin_header.plugin_registry_offset,
            )?;
            Ok((asset, Some(plugin_header), Some(plugin_registry)))
        } else {
            Ok((asset, None, None))
        }
    }
    /// Check that a compression proof results in same on-chain hash.
    pub fn verify_proof(
        hashed_asset: &AccountInfo,
        compression_proof: &CompressionProof,
    ) -> Result<(Asset, Vec<HashablePluginSchema>), ProgramError> {
        let asset = Asset::from(compression_proof.clone());
        let asset_hash = asset.hash()?;
        let mut sorted_plugins = compression_proof.plugins.clone();
        sorted_plugins.sort_by(HashablePluginSchema::compare_indeces);
        let plugin_hashes = sorted_plugins
            .iter()
            .map(|plugin| plugin.hash())
            .collect::<Result<Vec<[u8; 32]>, ProgramError>>()?;
        let hashed_asset_schema = HashedAssetSchema {
            asset_hash,
            plugin_hashes,
        };
        let hashed_asset_schema_hash = hashed_asset_schema.hash()?;
        let current_account_hash = HashedAsset::load(hashed_asset, 0)?.hash;
        if hashed_asset_schema_hash != current_account_hash {
            return Err(MplCoreError::IncorrectAssetHash.into());
        }
        Ok((asset, sorted_plugins))
    }
    pub(crate) fn close_program_account<'a>(
        account_to_close_info: &AccountInfo<'a>,
        funds_dest_account_info: &AccountInfo<'a>,
    ) -> ProgramResult {
        let rent = Rent::get()?;
        let account_size = account_to_close_info.data_len();
        let account_rent = rent.minimum_balance(account_size);
        let one_byte_rent = rent.minimum_balance(1);
        let amount_to_return = account_rent
            .checked_sub(one_byte_rent)
            .ok_or(MplCoreError::NumericalOverflowError)?;
        let dest_starting_lamports = funds_dest_account_info.lamports();
        **funds_dest_account_info
            .lamports
            .borrow_mut() = dest_starting_lamports
            .checked_add(amount_to_return)
            .ok_or(MplCoreError::NumericalOverflowError)?;
        **account_to_close_info.try_borrow_mut_lamports()? -= amount_to_return;
        account_to_close_info.realloc(1, false)?;
        account_to_close_info.data.borrow_mut()[0] = Key::Uninitialized.to_u8().unwrap();
        Ok(())
    }
    /// Resize an account using realloc and retain any lamport overages, modified from Solana Cookbook
    pub(crate) fn resize_or_reallocate_account<'a>(
        target_account: &AccountInfo<'a>,
        funding_account: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        new_size: usize,
    ) -> ProgramResult {
        let rent = Rent::get()?;
        let new_minimum_balance = rent.minimum_balance(new_size);
        let current_minimum_balance = rent.minimum_balance(target_account.data_len());
        let account_infos = &[
            funding_account.clone(),
            target_account.clone(),
            system_program.clone(),
        ];
        if new_minimum_balance >= current_minimum_balance {
            let lamports_diff = new_minimum_balance
                .saturating_sub(current_minimum_balance);
            ::solana_program::log::sol_log(
                &{
                    let res = ::alloc::fmt::format(
                        format_args!(
                            "Transferring {0} from {1} to {2}.",
                            lamports_diff,
                            funding_account.key,
                            target_account.key,
                        ),
                    );
                    res
                },
            );
            invoke(
                &system_instruction::transfer(
                    funding_account.key,
                    target_account.key,
                    lamports_diff,
                ),
                account_infos,
            )?;
        } else {
            let lamports_diff = current_minimum_balance
                .saturating_sub(new_minimum_balance);
            ::solana_program::log::sol_log(
                &{
                    let res = ::alloc::fmt::format(
                        format_args!(
                            "Transferring {0} from {1}:{2} to {3}:{4}.",
                            lamports_diff,
                            target_account.key,
                            target_account.lamports(),
                            funding_account.key,
                            funding_account.lamports(),
                        ),
                    );
                    res
                },
            );
            **funding_account.try_borrow_mut_lamports()? += lamports_diff;
            **target_account.try_borrow_mut_lamports()? -= lamports_diff;
        }
        target_account.realloc(new_size, false)?;
        Ok(())
    }
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    /// Validate asset permissions using lifecycle validations for asset, collection, and plugins.
    pub fn validate_asset_permissions<'a>(
        authority_info: &AccountInfo<'a>,
        asset: &AccountInfo<'a>,
        collection: Option<&AccountInfo<'a>>,
        new_owner: Option<&AccountInfo<'a>>,
        new_plugin: Option<&Plugin>,
        asset_check_fp: fn() -> CheckResult,
        collection_check_fp: fn() -> CheckResult,
        plugin_check_fp: fn(&PluginType) -> CheckResult,
        asset_validate_fp: fn(
            &Asset,
            &AccountInfo,
            Option<&Plugin>,
        ) -> Result<ValidationResult, ProgramError>,
        collection_validate_fp: fn(
            &Collection,
            &AccountInfo,
            Option<&Plugin>,
        ) -> Result<ValidationResult, ProgramError>,
        plugin_validate_fp: fn(
            &Plugin,
            &AccountInfo,
            Option<&AccountInfo>,
            &Authority,
            Option<&Plugin>,
            Option<&Authority>,
        ) -> Result<ValidationResult, ProgramError>,
    ) -> Result<(Asset, Option<PluginHeader>, Option<PluginRegistry>), ProgramError> {
        let (deserialized_asset, plugin_header, plugin_registry) = fetch_core_data::<
            Asset,
        >(asset)?;
        let resolved_authority = resolve_to_authority(
            authority_info,
            collection,
            &deserialized_asset,
        )?;
        if let UpdateAuthority::Collection(collection_address) = deserialized_asset
            .update_authority
        {
            if collection.is_none() {
                return Err(MplCoreError::MissingCollection.into());
            } else if collection.unwrap().key != &collection_address {
                return Err(MplCoreError::InvalidCollection.into());
            }
        }
        let mut checks: BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)> = BTreeMap::new();
        let asset_check = asset_check_fp();
        let collection_check = if collection.is_some() {
            collection_check_fp()
        } else {
            CheckResult::None
        };
        if let Some(collection_info) = collection {
            fetch_core_data::<Collection>(collection_info)
                .map(|(_, _, registry)| {
                    registry
                        .map(|r| {
                            r.check_registry(
                                Key::Collection,
                                plugin_check_fp,
                                &mut checks,
                            );
                            r
                        })
                })?;
        }
        if let Some(registry) = plugin_registry.as_ref() {
            registry.check_registry(Key::Asset, plugin_check_fp, &mut checks);
        }
        ::solana_program::log::sol_log(
            &{
                let res = ::alloc::fmt::format(format_args!("checks: {0:#?}", checks));
                res
            },
        );
        let mut approved = false;
        let mut rejected = false;
        if asset_check != CheckResult::None {
            match asset_validate_fp(
                &Asset::load(asset, 0)?,
                authority_info,
                new_plugin,
            )? {
                ValidationResult::Approved => approved = true,
                ValidationResult::Rejected => rejected = true,
                ValidationResult::Pass => {}
                ValidationResult::ForceApproved => {
                    return Ok((deserialized_asset, plugin_header, plugin_registry));
                }
            }
        }
        ::solana_program::log::sol_log(
            &{
                let res = ::alloc::fmt::format(
                    format_args!("approved: {0:?} rejected {1:?}", approved, rejected),
                );
                res
            },
        );
        if collection_check != CheckResult::None {
            match collection_validate_fp(
                &Collection::load(
                    collection.ok_or(MplCoreError::MissingCollection)?,
                    0,
                )?,
                authority_info,
                new_plugin,
            )? {
                ValidationResult::Approved => approved = true,
                ValidationResult::Rejected => rejected = true,
                ValidationResult::Pass => {}
                ValidationResult::ForceApproved => {
                    return Ok((deserialized_asset, plugin_header, plugin_registry));
                }
            }
        }
        ::solana_program::log::sol_log(
            &{
                let res = ::alloc::fmt::format(
                    format_args!("approved: {0:?} rejected {1:?}", approved, rejected),
                );
                res
            },
        );
        match validate_plugin_checks(
            Key::Collection,
            &checks,
            authority_info,
            new_owner,
            new_plugin,
            Some(asset),
            collection,
            &resolved_authority,
            plugin_validate_fp,
        )? {
            ValidationResult::Approved => approved = true,
            ValidationResult::Rejected => rejected = true,
            ValidationResult::Pass => {}
            ValidationResult::ForceApproved => {
                return Ok((deserialized_asset, plugin_header, plugin_registry));
            }
        };
        ::solana_program::log::sol_log(
            &{
                let res = ::alloc::fmt::format(
                    format_args!("approved: {0:?} rejected {1:?}", approved, rejected),
                );
                res
            },
        );
        match validate_plugin_checks(
            Key::Asset,
            &checks,
            authority_info,
            new_owner,
            new_plugin,
            Some(asset),
            collection,
            &resolved_authority,
            plugin_validate_fp,
        )? {
            ValidationResult::Approved => approved = true,
            ValidationResult::Rejected => rejected = true,
            ValidationResult::Pass => {}
            ValidationResult::ForceApproved => {
                return Ok((deserialized_asset, plugin_header, plugin_registry));
            }
        };
        ::solana_program::log::sol_log(
            &{
                let res = ::alloc::fmt::format(
                    format_args!("approved: {0:?} rejected {1:?}", approved, rejected),
                );
                res
            },
        );
        if rejected {
            return Err(MplCoreError::InvalidAuthority.into());
        } else if !approved {
            return Err(MplCoreError::NoApprovals.into());
        }
        Ok((deserialized_asset, plugin_header, plugin_registry))
    }
    /// Validate collection permissions using lifecycle validations for collection and plugins.
    #[allow(clippy::type_complexity)]
    pub fn validate_collection_permissions<'a>(
        authority_info: &AccountInfo<'a>,
        collection: &AccountInfo<'a>,
        new_plugin: Option<&Plugin>,
        collection_check_fp: fn() -> CheckResult,
        plugin_check_fp: fn(&PluginType) -> CheckResult,
        collection_validate_fp: fn(
            &Collection,
            &AccountInfo,
            Option<&Plugin>,
        ) -> Result<ValidationResult, ProgramError>,
        plugin_validate_fp: fn(
            &Plugin,
            &AccountInfo,
            Option<&AccountInfo>,
            &Authority,
            Option<&Plugin>,
            Option<&Authority>,
        ) -> Result<ValidationResult, ProgramError>,
    ) -> Result<
        (Collection, Option<PluginHeader>, Option<PluginRegistry>),
        ProgramError,
    > {
        let (deserialized_collection, plugin_header, plugin_registry) = fetch_core_data::<
            Collection,
        >(collection)?;
        let resolved_authority = resolve_collection_authority(
            authority_info,
            collection,
        )?;
        let mut checks: BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)> = BTreeMap::new();
        let core_check = (Key::Collection, collection_check_fp());
        if let Some(registry) = plugin_registry.as_ref() {
            registry.check_registry(Key::Collection, plugin_check_fp, &mut checks);
        }
        ::solana_program::log::sol_log(
            &{
                let res = ::alloc::fmt::format(format_args!("checks: {0:#?}", checks));
                res
            },
        );
        let mut approved = false;
        let mut rejected = false;
        if match core_check {
            (
                Key::Collection,
                CheckResult::CanApprove
                | CheckResult::CanReject
                | CheckResult::CanForceApprove,
            ) => true,
            _ => false,
        } {
            let result = match core_check.0 {
                Key::Collection => {
                    collection_validate_fp(
                        &deserialized_collection,
                        authority_info,
                        new_plugin,
                    )?
                }
                _ => return Err(MplCoreError::IncorrectAccount.into()),
            };
            match result {
                ValidationResult::Approved => approved = true,
                ValidationResult::Rejected => rejected = true,
                ValidationResult::Pass => {}
                ValidationResult::ForceApproved => {
                    return Ok((deserialized_collection, plugin_header, plugin_registry));
                }
            }
        }
        ::solana_program::log::sol_log(
            &{
                let res = ::alloc::fmt::format(
                    format_args!("approved: {0:?} rejected {1:?}", approved, rejected),
                );
                res
            },
        );
        match validate_plugin_checks(
            Key::Collection,
            &checks,
            authority_info,
            None,
            new_plugin,
            None,
            Some(collection),
            &resolved_authority,
            plugin_validate_fp,
        )? {
            ValidationResult::Approved => approved = true,
            ValidationResult::Rejected => rejected = true,
            ValidationResult::Pass => {}
            ValidationResult::ForceApproved => {
                return Ok((deserialized_collection, plugin_header, plugin_registry));
            }
        };
        ::solana_program::log::sol_log(
            &{
                let res = ::alloc::fmt::format(
                    format_args!("approved: {0:?} rejected {1:?}", approved, rejected),
                );
                res
            },
        );
        if rejected || !approved {
            return Err(MplCoreError::InvalidAuthority.into());
        }
        Ok((deserialized_collection, plugin_header, plugin_registry))
    }
    /// Take an `Asset` and Vec of `HashablePluginSchema` and rebuild the asset in account space.
    pub fn rebuild_account_state_from_proof_data<'a>(
        asset: Asset,
        plugins: Vec<HashablePluginSchema>,
        asset_info: &AccountInfo<'a>,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
    ) -> ProgramResult {
        let serialized_data = asset.try_to_vec()?;
        resize_or_reallocate_account(
            asset_info,
            payer,
            system_program,
            serialized_data.len(),
        )?;
        sol_memcpy(
            &mut asset_info.try_borrow_mut_data()?,
            &serialized_data,
            serialized_data.len(),
        );
        if !plugins.is_empty() {
            create_meta_idempotent::<Asset>(asset_info, payer, system_program)?;
            for plugin in plugins {
                initialize_plugin::<
                    Asset,
                >(&plugin.plugin, &plugin.authority, asset_info, payer, system_program)?;
            }
        }
        Ok(())
    }
    /// Take `Asset` and `PluginRegistry` for a decompressed asset, and compress into account space.
    pub fn compress_into_account_space<'a>(
        mut asset: Asset,
        plugin_registry: Option<PluginRegistry>,
        asset_info: &AccountInfo<'a>,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
    ) -> Result<CompressionProof, ProgramError> {
        let seq = asset.seq.unwrap_or(0).saturating_add(1);
        asset.seq = Some(seq);
        let asset_hash = asset.hash()?;
        let mut compression_proof = CompressionProof::new(
            asset,
            seq,
            ::alloc::vec::Vec::new(),
        );
        let mut plugin_hashes = ::alloc::vec::Vec::new();
        if let Some(plugin_registry) = plugin_registry {
            let mut registry_records = plugin_registry.registry;
            registry_records.sort_by(RegistryRecord::compare_offsets);
            for (i, record) in registry_records.into_iter().enumerate() {
                let plugin = Plugin::deserialize(
                    &mut &(*asset_info.data).borrow()[record.offset..],
                )?;
                let hashable_plugin_schema = HashablePluginSchema {
                    index: i,
                    authority: record.authority,
                    plugin,
                };
                let plugin_hash = hashable_plugin_schema.hash()?;
                plugin_hashes.push(plugin_hash);
                compression_proof.plugins.push(hashable_plugin_schema);
            }
        }
        let hashed_asset_schema = HashedAssetSchema {
            asset_hash,
            plugin_hashes,
        };
        let hashed_asset = HashedAsset::new(hashed_asset_schema.hash()?);
        let serialized_data = hashed_asset.try_to_vec()?;
        resize_or_reallocate_account(
            asset_info,
            payer,
            system_program,
            serialized_data.len(),
        )?;
        sol_memcpy(
            &mut asset_info.try_borrow_mut_data()?,
            &serialized_data,
            serialized_data.len(),
        );
        Ok(compression_proof)
    }
    pub(crate) fn resolve_to_authority(
        authority_info: &AccountInfo,
        maybe_collection_info: Option<&AccountInfo>,
        asset: &Asset,
    ) -> Result<Authority, ProgramError> {
        let authority_type = if authority_info.key == &asset.owner {
            Authority::Owner
        } else if asset.update_authority == UpdateAuthority::Address(*authority_info.key)
        {
            Authority::UpdateAuthority
        } else if let UpdateAuthority::Collection(collection_address) = asset
            .update_authority
        {
            match maybe_collection_info {
                Some(collection_info) => {
                    if collection_info.key != &collection_address {
                        return Err(MplCoreError::InvalidCollection.into());
                    }
                    let collection: Collection = Collection::load(collection_info, 0)?;
                    if authority_info.key == &collection.update_authority {
                        Authority::UpdateAuthority
                    } else {
                        Authority::Pubkey {
                            address: *authority_info.key,
                        }
                    }
                }
                None => return Err(MplCoreError::InvalidCollection.into()),
            }
        } else {
            Authority::Pubkey {
                address: *authority_info.key,
            }
        };
        Ok(authority_type)
    }
    pub(crate) fn resolve_collection_authority(
        authority_info: &AccountInfo,
        collection_info: &AccountInfo,
    ) -> Result<Authority, ProgramError> {
        let collection: Collection = Collection::load(collection_info, 0)?;
        if authority_info.key == collection.owner() {
            Ok(Authority::Owner)
        } else if authority_info.key == &collection.update_authority {
            Ok(Authority::UpdateAuthority)
        } else {
            Ok(Authority::Pubkey {
                address: *authority_info.key,
            })
        }
    }
    /// Resolves the authority for the transaction for an optional authority pattern.
    pub(crate) fn resolve_authority<'a>(
        payer: &'a AccountInfo<'a>,
        authority: Option<&'a AccountInfo<'a>>,
    ) -> Result<&'a AccountInfo<'a>, ProgramError> {
        match authority {
            Some(authority) => {
                assert_signer(authority).unwrap();
                Ok(authority)
            }
            None => Ok(payer),
        }
    }
}
pub use solana_program;
/// The const program ID.
pub const ID: ::solana_program::pubkey::Pubkey = ::solana_program::pubkey::Pubkey::new_from_array([
    175u8,
    84u8,
    171u8,
    16u8,
    189u8,
    151u8,
    165u8,
    66u8,
    160u8,
    158u8,
    247u8,
    179u8,
    152u8,
    137u8,
    221u8,
    12u8,
    211u8,
    148u8,
    164u8,
    204u8,
    233u8,
    223u8,
    166u8,
    205u8,
    201u8,
    126u8,
    190u8,
    45u8,
    35u8,
    91u8,
    167u8,
    72u8,
]);
/// Returns `true` if given pubkey is the program ID.
pub fn check_id(id: &::solana_program::pubkey::Pubkey) -> bool {
    id == &ID
}
/// Returns the program ID.
pub const fn id() -> ::solana_program::pubkey::Pubkey {
    ID
}
