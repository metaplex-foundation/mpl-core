use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    plugins::{ExternalPluginInitInfo, ExternalPluginKey},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddExternalPluginV1Args {
    /// External plugin key.
    pub key: ExternalPluginKey,
    /// External plugin initialization info.
    pub init_info: ExternalPluginInitInfo,
}

pub(crate) fn add_external_plugin<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: AddExternalPluginV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddCollectionExternalPluginV1Args {
    /// External plugin key.
    pub key: ExternalPluginKey,
    /// External plugin initialization info.
    pub init_info: ExternalPluginInitInfo,
}

pub(crate) fn add_collection_external_plugin<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: AddCollectionExternalPluginV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}
