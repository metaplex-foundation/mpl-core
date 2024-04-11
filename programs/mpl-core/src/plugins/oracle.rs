use borsh::{BorshDeserialize, BorshSerialize};

use super::{Authority, ExternalCheckResult, ExtraAccount, HookableLifecycleEvent};

/// Oracle plugin that allows getting a `ValidationResult` for a lifecycle event from an arbitrary
/// account either specified by or derived from the `Pubkey` attached to the `ExternalPluginKey`.
/// This hook is used for any lifecycle events that were selected in the `ExternalPluginRecord` for
/// the plugin.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct Oracle {
    /// Optional PDA (derived from Pubkey attached to `ExternalPluginKey`).
    pub pda: Option<ExtraAccount>,
}

/// Oracle initialization info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct OracleInitInfo {
    /// Initial authority.
    pub init_authority: Option<Authority>,
    /// The lifecyle events for which the the external plugin is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// Optional PDA (derived from Pubkey attached to `ExternalPluginKey`).
    pub pda: Option<ExtraAccount>,
}

/// Oracle update info.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct OracleUpdateInfo {
    /// The lifecyle events for which the the external plugin is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// Optional PDA (derived from Pubkey attached to `ExternalPluginKey`).
    pub pda: Option<ExtraAccount>,
}
