use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{error::MplCoreError, plugins::ExternalPluginKey};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AllocateExternalPluginDataV1Args {
    /// External plugin key.
    pub key: ExternalPluginKey,
    /// External plugin initialization info.
    pub target_size: usize,
}

pub(crate) fn allocate_external_plugin_data<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: AllocateExternalPluginDataV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AllocateCollectionExternalPluginDataV1Args {
    /// External plugin key.
    pub key: ExternalPluginKey,
    /// External plugin initialization info.
    pub target_size: usize,
}

pub(crate) fn allocate_collection_external_plugin_data<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: AllocateCollectionExternalPluginDataV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}
