use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{error::MplCoreError, plugins::ExternalPluginKey};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateExternalPluginV1Args {
    /// External plugin key.
    pub plugin_key: ExternalPluginKey,
    /// Data to update.
    pub data: Vec<u8>,
}

pub(crate) fn update_external_plugin<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: UpdateExternalPluginV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateCollectionExternalPluginV1Args {
    /// External plugin key.
    pub plugin_key: ExternalPluginKey,
    /// Data to update.
    pub data: Vec<u8>,
}

pub(crate) fn update_collection_external_plugin<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: UpdateCollectionExternalPluginV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}
