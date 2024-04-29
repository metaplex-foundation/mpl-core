use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the Mpl Core program.
#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
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

    /// 27 - Plugin Manager cannot redelegate a delegated plugin without revoking first.
    #[error("Plugin Manager cannot redelegate a delegated plugin without revoking first")]
    CannotRedelegate,

    /// 28 - Invalid setting for plugin
    #[error("Invalid setting for plugin")]
    InvalidPluginSetting,

    /// 29 - Conflicting Authority
    #[error("Cannot specify both an update authority and collection on an asset")]
    ConflictingAuthority,

    /// 30 - Invalid Log Wrapper Program
    #[error("Invalid Log Wrapper Program")]
    InvalidLogWrapperProgram,

    /// 31 - External Plugin not found
    #[error("External Plugin not found")]
    ExternalPluginNotFound,

    /// 32 - External Plugin already exists
    #[error("External Plugin already exists")]
    ExternalPluginAlreadyExists,

    /// 33 - Missing asset needed for extra account PDA derivation
    #[error("Missing asset needed for extra account PDA derivation")]
    MissingAsset,

    /// 34 - Missing account needed for external plugin
    #[error("Missing account needed for external plugin")]
    MissingExternalAccount,
}

impl PrintProgramError for MplCoreError {
    fn print<E>(&self) {
        msg!(&self.to_string());
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
