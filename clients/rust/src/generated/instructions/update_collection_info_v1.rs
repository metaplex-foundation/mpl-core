//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! [https://github.com/metaplex-foundation/kinobi]
//!

use crate::generated::types::UpdateType;
#[cfg(feature = "anchor")]
use anchor_lang::prelude::{AnchorDeserialize, AnchorSerialize};
#[cfg(not(feature = "anchor"))]
use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct UpdateCollectionInfoV1 {
    /// The address of the asset
    pub collection: solana_program::pubkey::Pubkey,
    /// Bubblegum PDA signer
    pub bubblegum_signer: solana_program::pubkey::Pubkey,
}

impl UpdateCollectionInfoV1 {
    pub fn instruction(
        &self,
        args: UpdateCollectionInfoV1InstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: UpdateCollectionInfoV1InstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(2 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.collection,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.bubblegum_signer,
            true,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = UpdateCollectionInfoV1InstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = args.try_to_vec().unwrap();
        data.append(&mut args);

        solana_program::instruction::Instruction {
            program_id: crate::MPL_CORE_ID,
            accounts,
            data,
        }
    }
}

#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
pub struct UpdateCollectionInfoV1InstructionData {
    discriminator: u8,
}

impl UpdateCollectionInfoV1InstructionData {
    pub fn new() -> Self {
        Self { discriminator: 32 }
    }
}

#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateCollectionInfoV1InstructionArgs {
    pub update_type: UpdateType,
    pub amount: u32,
}

/// Instruction builder for `UpdateCollectionInfoV1`.
///
/// ### Accounts:
///
///   0. `[writable]` collection
///   1. `[signer]` bubblegum_signer
#[derive(Default)]
pub struct UpdateCollectionInfoV1Builder {
    collection: Option<solana_program::pubkey::Pubkey>,
    bubblegum_signer: Option<solana_program::pubkey::Pubkey>,
    update_type: Option<UpdateType>,
    amount: Option<u32>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl UpdateCollectionInfoV1Builder {
    pub fn new() -> Self {
        Self::default()
    }
    /// The address of the asset
    #[inline(always)]
    pub fn collection(&mut self, collection: solana_program::pubkey::Pubkey) -> &mut Self {
        self.collection = Some(collection);
        self
    }
    /// Bubblegum PDA signer
    #[inline(always)]
    pub fn bubblegum_signer(
        &mut self,
        bubblegum_signer: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.bubblegum_signer = Some(bubblegum_signer);
        self
    }
    #[inline(always)]
    pub fn update_type(&mut self, update_type: UpdateType) -> &mut Self {
        self.update_type = Some(update_type);
        self
    }
    #[inline(always)]
    pub fn amount(&mut self, amount: u32) -> &mut Self {
        self.amount = Some(amount);
        self
    }
    /// Add an aditional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: solana_program::instruction::AccountMeta,
    ) -> &mut Self {
        self.__remaining_accounts.push(account);
        self
    }
    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[solana_program::instruction::AccountMeta],
    ) -> &mut Self {
        self.__remaining_accounts.extend_from_slice(accounts);
        self
    }
    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = UpdateCollectionInfoV1 {
            collection: self.collection.expect("collection is not set"),
            bubblegum_signer: self.bubblegum_signer.expect("bubblegum_signer is not set"),
        };
        let args = UpdateCollectionInfoV1InstructionArgs {
            update_type: self.update_type.clone().expect("update_type is not set"),
            amount: self.amount.clone().expect("amount is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `update_collection_info_v1` CPI accounts.
pub struct UpdateCollectionInfoV1CpiAccounts<'a, 'b> {
    /// The address of the asset
    pub collection: &'b solana_program::account_info::AccountInfo<'a>,
    /// Bubblegum PDA signer
    pub bubblegum_signer: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `update_collection_info_v1` CPI instruction.
pub struct UpdateCollectionInfoV1Cpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the asset
    pub collection: &'b solana_program::account_info::AccountInfo<'a>,
    /// Bubblegum PDA signer
    pub bubblegum_signer: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: UpdateCollectionInfoV1InstructionArgs,
}

impl<'a, 'b> UpdateCollectionInfoV1Cpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: UpdateCollectionInfoV1CpiAccounts<'a, 'b>,
        args: UpdateCollectionInfoV1InstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            collection: accounts.collection,
            bubblegum_signer: accounts.bubblegum_signer,
            __args: args,
        }
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        let mut accounts = Vec::with_capacity(2 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.collection.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.bubblegum_signer.key,
            true,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = UpdateCollectionInfoV1InstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::MPL_CORE_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(2 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.collection.clone());
        account_infos.push(self.bubblegum_signer.clone());
        remaining_accounts
            .iter()
            .for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

        if signers_seeds.is_empty() {
            solana_program::program::invoke(&instruction, &account_infos)
        } else {
            solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

/// Instruction builder for `UpdateCollectionInfoV1` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` collection
///   1. `[signer]` bubblegum_signer
pub struct UpdateCollectionInfoV1CpiBuilder<'a, 'b> {
    instruction: Box<UpdateCollectionInfoV1CpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> UpdateCollectionInfoV1CpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(UpdateCollectionInfoV1CpiBuilderInstruction {
            __program: program,
            collection: None,
            bubblegum_signer: None,
            update_type: None,
            amount: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    /// The address of the asset
    #[inline(always)]
    pub fn collection(
        &mut self,
        collection: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.collection = Some(collection);
        self
    }
    /// Bubblegum PDA signer
    #[inline(always)]
    pub fn bubblegum_signer(
        &mut self,
        bubblegum_signer: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.bubblegum_signer = Some(bubblegum_signer);
        self
    }
    #[inline(always)]
    pub fn update_type(&mut self, update_type: UpdateType) -> &mut Self {
        self.instruction.update_type = Some(update_type);
        self
    }
    #[inline(always)]
    pub fn amount(&mut self, amount: u32) -> &mut Self {
        self.instruction.amount = Some(amount);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: &'b solana_program::account_info::AccountInfo<'a>,
        is_writable: bool,
        is_signer: bool,
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .push((account, is_writable, is_signer));
        self
    }
    /// Add additional accounts to the instruction.
    ///
    /// Each account is represented by a tuple of the `AccountInfo`, a `bool` indicating whether the account is writable or not,
    /// and a `bool` indicating whether the account is a signer or not.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed(&[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        let args = UpdateCollectionInfoV1InstructionArgs {
            update_type: self
                .instruction
                .update_type
                .clone()
                .expect("update_type is not set"),
            amount: self.instruction.amount.clone().expect("amount is not set"),
        };
        let instruction = UpdateCollectionInfoV1Cpi {
            __program: self.instruction.__program,

            collection: self.instruction.collection.expect("collection is not set"),

            bubblegum_signer: self
                .instruction
                .bubblegum_signer
                .expect("bubblegum_signer is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

struct UpdateCollectionInfoV1CpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    collection: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    bubblegum_signer: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    update_type: Option<UpdateType>,
    amount: Option<u32>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
