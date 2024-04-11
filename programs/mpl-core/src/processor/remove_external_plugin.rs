use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{error::MplCoreError, plugins::ExternalPluginKey};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveExternalPluginV1Args {
    /// External plugin key.
    pub key: ExternalPluginKey,
}

pub(crate) fn remove_external_plugin<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: RemoveExternalPluginV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveCollectionExternalPluginV1Args {
    /// External plugin key.
    pub key: ExternalPluginKey,
}

pub(crate) fn remove_collection_external_plugin<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: RemoveCollectionExternalPluginV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}
