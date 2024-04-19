use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{error::MplCoreError, plugins::ExternalPluginKey};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct WriteExternalPluginDataV1Args {
    /// External plugin key.
    pub key: ExternalPluginKey,
    /// The offset of the data to write.
    pub offset: usize,
    /// The data to write.
    pub data: Vec<u8>,
}

pub(crate) fn write_external_plugin_data<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: WriteExternalPluginDataV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct WriteCollectionExternalPluginDataV1Args {
    /// External plugin key.
    pub key: ExternalPluginKey,
    /// The offset of the data to write.
    pub offset: usize,
    /// The data to write.
    pub data: Vec<u8>,
}

pub(crate) fn write_collection_external_plugin_data<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: WriteCollectionExternalPluginDataV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}
