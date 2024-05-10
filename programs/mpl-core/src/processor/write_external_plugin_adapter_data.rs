use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{error::MplCoreError, plugins::ExternalPluginAdapterKey};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct WriteExternalPluginAdapterDataV1Args {
    /// External external plugin adapter key.
    pub key: ExternalPluginAdapterKey,
    /// The data to write.
    pub data: Vec<u8>,
}

pub(crate) fn write_external_plugin_adapter_data<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: WriteExternalPluginAdapterDataV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct WriteCollectionExternalPluginAdapterDataV1Args {
    /// External external plugin adapter key.
    pub key: ExternalPluginAdapterKey,
    /// The data to write.
    pub data: Vec<u8>,
}

pub(crate) fn write_collection_external_plugin_adapter_data<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: WriteCollectionExternalPluginAdapterDataV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}
