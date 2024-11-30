use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memmove,
};

use crate::{
    error::MplCoreError,
    plugins::PluginRegistryV1,
    state::{CoreAsset, DataBlob, SolanaAccount},
};

use super::resize_or_reallocate_account;

pub(crate) struct PluginSystem<'a> {
    account: &'a AccountInfo<'a>,
    root: Option<&'a dyn CoreAsset>,
    registry_offset: Option<usize>,
    plugin_registry: Option<&'a PluginRegistryV1>,
}

impl<'a> PluginSystem<'a> {
    /// Create a new dynamic account info.
    pub(crate) fn new(account_info: &'a AccountInfo<'a>) -> Self {
        Self {
            account: account_info,
            root: None,
            registry_offset: None,
            plugin_registry: None,
        }
    }

    /// Insert data into the account at the given offset.
    pub(crate) fn insert(
        &self,
        offset: usize,
        data: &(impl DataBlob + SolanaAccount),
        funder: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
    ) -> ProgramResult {
        // Realloc the account to the new size.
        let new_size = offset + data.get_size();
        resize_or_reallocate_account(self.account, funder, system_program, new_size)?;
        // Insert the data.
        data.save(self.account, offset)?;

        Ok(())
    }

    pub(crate) fn append(
        &self,
        data: impl DataBlob + SolanaAccount,
        funder: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
    ) -> ProgramResult {
        // Realloc the account to the new size.
        let current_size = self.account.data_len();
        let new_size = current_size + data.get_size();
        resize_or_reallocate_account(self.account, funder, system_program, new_size)?;
        // Insert the data.
        data.save(self.account, current_size)?;

        Ok(())
    }

    pub(crate) fn replace(
        &self,
        offset: usize,
        len: usize,
        data: impl DataBlob + SolanaAccount,
        funder: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
    ) -> ProgramResult {
        // Check if the data is larger or smaller than the current data.
        let data_len = data.get_size();
        let post_offset = offset
            .checked_add(len)
            .ok_or(MplCoreError::NumericalOverflow)?;
        let new_post_offset = offset
            .checked_add(data_len)
            .ok_or(MplCoreError::NumericalOverflow)?;
        let data_to_move_len = self.account.data_len().saturating_sub(post_offset);
        #[allow(clippy::comparison_chain)]
        if len > data_len {
            // Move the data after the data being replaced.
            unsafe {
                let base = self.account.data.borrow_mut().as_mut_ptr();
                sol_memmove(
                    base.add(new_post_offset),
                    base.add(post_offset),
                    data_to_move_len,
                );
            }

            // Realloc the account to the new size.
            let new_size = offset + data_len;
            resize_or_reallocate_account(self.account, funder, system_program, new_size)?;
        } else if len < data_len {
            // Realloc the account to the new size.
            let new_size = offset + data_len;
            resize_or_reallocate_account(self.account, funder, system_program, new_size)?;

            // Move the data after the data being replaced.
            unsafe {
                let base = self.account.data.borrow_mut().as_mut_ptr();
                sol_memmove(
                    base.add(new_post_offset),
                    base.add(post_offset),
                    data_to_move_len,
                );
            }
        }

        // Save the data.
        data.save(self.account, offset)?;

        Ok(())
    }

    pub(crate) fn remove(
        &self,
        offset: usize,
        len: usize,
        funder: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
    ) -> ProgramResult {
        let post_offset = offset.checked_add(len).unwrap();
        let data_to_move_len = self.account.data_len().saturating_sub(post_offset);

        // Move the data.
        unsafe {
            let base = self.account.data.borrow_mut().as_mut_ptr();
            sol_memmove(base.add(offset), base.add(post_offset), data_to_move_len);
        }

        // Realloc the account to the new size.
        let new_size = self.account.data_len().saturating_sub(len);
        resize_or_reallocate_account(self.account, funder, system_program, new_size)?;

        Ok(())
    }
}
