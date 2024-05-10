use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{error::MplCoreError, plugins::PluginAdapterKey};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct WritePluginAdapterDataV1Args {
    /// Plugin adapter key.
    pub key: PluginAdapterKey,
    /// The data to write.
    pub data: Vec<u8>,
}

pub(crate) fn write_plugin_adapter_data<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: WritePluginAdapterDataV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct WriteCollectionPluginAdapterDataV1Args {
    /// Plugin adapter key.
    pub key: PluginAdapterKey,
    /// The data to write.
    pub data: Vec<u8>,
}

pub(crate) fn write_collection_plugin_adapter_data<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: WriteCollectionPluginAdapterDataV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}
