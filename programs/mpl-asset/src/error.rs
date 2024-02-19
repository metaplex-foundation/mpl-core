use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum MplAssetError {
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

    /// 5 - Incorrect account
    #[error("Incorrect account")]
    IncorrectAccount,

    /// 5 - Provided data does not match asset hash.
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
}

impl PrintProgramError for MplAssetError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<MplAssetError> for ProgramError {
    fn from(e: MplAssetError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MplAssetError {
    fn type_of() -> &'static str {
        "Mpl Asset Error"
    }
}
