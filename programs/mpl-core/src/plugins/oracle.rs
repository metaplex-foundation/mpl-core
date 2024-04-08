use borsh::{BorshDeserialize, BorshSerialize};

use super::ExtraAccount;

/// Oracle plugin that allows getting a `ValidationResult` for a lifecycle event from an arbitrary
/// account either specified by or derived from the `Pubkey` attached to the `ExternalPluginKey`.
/// This hook is used for any lifecycle events that were selected in the `ExternalPluginRecord` for
/// the plugin.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub struct Oracle {
    /// Optional PDA (derived from Pubkey attached to `ExternalPluginKey`).
    pub pda: Option<ExtraAccount>,
}
