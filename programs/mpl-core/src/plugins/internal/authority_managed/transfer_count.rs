use crate::{
    plugins::{Plugin, PluginValidation},
    state::DataBlob,
};
use borsh::{BorshDeserialize, BorshSerialize};

/// The TransferCount plugin tracks the number of times an asset has been transferred.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Default)]
pub struct TransferCount {
    /// The number of times the asset has been transferred.
    pub count: u64,
}

impl TransferCount {
    const BASE_LEN: usize = 8; // The size of u64

    /// Initialize the TransferCount plugin with a count of 0.
    pub fn new() -> Self {
        Self::default()
    }
}

impl DataBlob for TransferCount {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

impl PluginValidation for TransferCount {
    fn side_effects_transfer(
        &self,
        _ctx: &crate::plugins::PluginSideEffectContext,
    ) -> Result<Plugin, solana_program::program_error::ProgramError> {
        Ok(Plugin::TransferCount(TransferCount {
            count: self.count + 1,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_count_len() {
        let transfer_count = TransferCount { count: 0 };
        let serialized = transfer_count.try_to_vec().unwrap();
        assert_eq!(serialized.len(), transfer_count.len());
    }

    #[test]
    fn test_transfer_count_default_len() {
        let transfer_count = TransferCount::new();
        let serialized = transfer_count.try_to_vec().unwrap();
        assert_eq!(serialized.len(), transfer_count.len());
    }
}
