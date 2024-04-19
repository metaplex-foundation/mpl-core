use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{error::MplCoreError, plugins::ExternalPluginKey};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct ClearExternalPluginDataV1Args {
    /// External plugin key.
    pub key: ExternalPluginKey,
}

pub(crate) fn clear_external_plugin_data<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: ClearExternalPluginDataV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct ClearCollectionExternalPluginDataV1Args {
    /// External plugin key.
    pub key: ExternalPluginKey,
}

pub(crate) fn clear_collection_external_plugin_data<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: ClearCollectionExternalPluginDataV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}
