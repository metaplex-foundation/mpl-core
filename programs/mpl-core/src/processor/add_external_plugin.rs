use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    plugins::{ExternalCheckResult, ExternalPlugin, ExternalPluginKey, LifecycleEvent},
    state::Authority,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddExternalPluginV1Args {
    /// External plugin key.
    pub plugin_key: ExternalPluginKey,
    /// External plugin.
    pub plugin: ExternalPlugin,
    /// Initial authority.
    pub init_authority: Option<Authority>,
    /// The lifecyle events for which the the external plugin is active.
    pub lifecycle_checks: Option<Vec<(LifecycleEvent, ExternalCheckResult)>>,
    /// External plugin initial data.
    pub data: Option<Vec<u8>>,
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
    pub plugin_key: ExternalPluginKey,
    /// External plugin.
    pub plugin: ExternalPlugin,
    /// Initial authority.
    pub init_authority: Option<Authority>,
    /// The lifecyle events for which the the external plugin is active.
    pub lifecycle_checks: Option<Vec<(LifecycleEvent, ExternalCheckResult)>>,
    /// External plugin initial data.
    pub data: Option<Vec<u8>>,
}

pub(crate) fn add_collection_external_plugin<'a>(
    _accounts: &'a [AccountInfo<'a>],
    _args: AddCollectionExternalPluginV1Args,
) -> ProgramResult {
    Err(MplCoreError::NotAvailable.into())
}