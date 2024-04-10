//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! [https://github.com/metaplex-foundation/kinobi]
//!

use crate::generated::types::ExternalPluginInitInfo;
use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct AddCollectionExternalPluginV1 {
    /// The address of the asset
    pub collection: solana_program::pubkey::Pubkey,
    /// The account paying for the storage fees
    pub payer: solana_program::pubkey::Pubkey,
    /// The owner or delegate of the asset
    pub authority: Option<solana_program::pubkey::Pubkey>,
    /// The system program
    pub system_program: solana_program::pubkey::Pubkey,
    /// The SPL Noop Program
    pub log_wrapper: Option<solana_program::pubkey::Pubkey>,
}

impl AddCollectionExternalPluginV1 {
    pub fn instruction(
        &self,
        args: AddCollectionExternalPluginV1InstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: AddCollectionExternalPluginV1InstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.collection,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.payer, true,
        ));
        if let Some(authority) = self.authority {
            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
                authority, true,
            ));
        } else {
            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
                crate::MPL_CORE_ID,
                false,
            ));
        }
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.system_program,
            false,
        ));
        if let Some(log_wrapper) = self.log_wrapper {
            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
                log_wrapper,
                false,
            ));
        } else {
            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
                crate::MPL_CORE_ID,
                false,
            ));
        }
        accounts.extend_from_slice(remaining_accounts);
        let mut data = AddCollectionExternalPluginV1InstructionData::new()
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

#[derive(BorshDeserialize, BorshSerialize)]
struct AddCollectionExternalPluginV1InstructionData {
    discriminator: u8,
}

impl AddCollectionExternalPluginV1InstructionData {
    fn new() -> Self {
        Self { discriminator: 23 }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AddCollectionExternalPluginV1InstructionArgs {
    pub external_plugin_init_info: ExternalPluginInitInfo,
}

/// Instruction builder for `AddCollectionExternalPluginV1`.
///
/// ### Accounts:
///
///   0. `[writable]` collection
///   1. `[writable, signer]` payer
///   2. `[signer, optional]` authority
///   3. `[optional]` system_program (default to `11111111111111111111111111111111`)
///   4. `[optional]` log_wrapper
#[derive(Default)]
pub struct AddCollectionExternalPluginV1Builder {
    collection: Option<solana_program::pubkey::Pubkey>,
    payer: Option<solana_program::pubkey::Pubkey>,
    authority: Option<solana_program::pubkey::Pubkey>,
    system_program: Option<solana_program::pubkey::Pubkey>,
    log_wrapper: Option<solana_program::pubkey::Pubkey>,
    external_plugin_init_info: Option<ExternalPluginInitInfo>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl AddCollectionExternalPluginV1Builder {
    pub fn new() -> Self {
        Self::default()
    }
    /// The address of the asset
    #[inline(always)]
    pub fn collection(&mut self, collection: solana_program::pubkey::Pubkey) -> &mut Self {
        self.collection = Some(collection);
        self
    }
    /// The account paying for the storage fees
    #[inline(always)]
    pub fn payer(&mut self, payer: solana_program::pubkey::Pubkey) -> &mut Self {
        self.payer = Some(payer);
        self
    }
    /// `[optional account]`
    /// The owner or delegate of the asset
    #[inline(always)]
    pub fn authority(&mut self, authority: Option<solana_program::pubkey::Pubkey>) -> &mut Self {
        self.authority = authority;
        self
    }
    /// `[optional account, default to '11111111111111111111111111111111']`
    /// The system program
    #[inline(always)]
    pub fn system_program(&mut self, system_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }
    /// `[optional account]`
    /// The SPL Noop Program
    #[inline(always)]
    pub fn log_wrapper(
        &mut self,
        log_wrapper: Option<solana_program::pubkey::Pubkey>,
    ) -> &mut Self {
        self.log_wrapper = log_wrapper;
        self
    }
    #[inline(always)]
    pub fn external_plugin_init_info(
        &mut self,
        external_plugin_init_info: ExternalPluginInitInfo,
    ) -> &mut Self {
        self.external_plugin_init_info = Some(external_plugin_init_info);
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
        let accounts = AddCollectionExternalPluginV1 {
            collection: self.collection.expect("collection is not set"),
            payer: self.payer.expect("payer is not set"),
            authority: self.authority,
            system_program: self
                .system_program
                .unwrap_or(solana_program::pubkey!("11111111111111111111111111111111")),
            log_wrapper: self.log_wrapper,
        };
        let args = AddCollectionExternalPluginV1InstructionArgs {
            external_plugin_init_info: self
                .external_plugin_init_info
                .clone()
                .expect("external_plugin_init_info is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `add_collection_external_plugin_v1` CPI accounts.
pub struct AddCollectionExternalPluginV1CpiAccounts<'a, 'b> {
    /// The address of the asset
    pub collection: &'b solana_program::account_info::AccountInfo<'a>,
    /// The account paying for the storage fees
    pub payer: &'b solana_program::account_info::AccountInfo<'a>,
    /// The owner or delegate of the asset
    pub authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// The system program
    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The SPL Noop Program
    pub log_wrapper: Option<&'b solana_program::account_info::AccountInfo<'a>>,
}

/// `add_collection_external_plugin_v1` CPI instruction.
pub struct AddCollectionExternalPluginV1Cpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the asset
    pub collection: &'b solana_program::account_info::AccountInfo<'a>,
    /// The account paying for the storage fees
    pub payer: &'b solana_program::account_info::AccountInfo<'a>,
    /// The owner or delegate of the asset
    pub authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// The system program
    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The SPL Noop Program
    pub log_wrapper: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// The arguments for the instruction.
    pub __args: AddCollectionExternalPluginV1InstructionArgs,
}

impl<'a, 'b> AddCollectionExternalPluginV1Cpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: AddCollectionExternalPluginV1CpiAccounts<'a, 'b>,
        args: AddCollectionExternalPluginV1InstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            collection: accounts.collection,
            payer: accounts.payer,
            authority: accounts.authority,
            system_program: accounts.system_program,
            log_wrapper: accounts.log_wrapper,
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
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.collection.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.payer.key,
            true,
        ));
        if let Some(authority) = self.authority {
            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
                *authority.key,
                true,
            ));
        } else {
            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
                crate::MPL_CORE_ID,
                false,
            ));
        }
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.system_program.key,
            false,
        ));
        if let Some(log_wrapper) = self.log_wrapper {
            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
                *log_wrapper.key,
                false,
            ));
        } else {
            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
                crate::MPL_CORE_ID,
                false,
            ));
        }
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = AddCollectionExternalPluginV1InstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::MPL_CORE_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(5 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.collection.clone());
        account_infos.push(self.payer.clone());
        if let Some(authority) = self.authority {
            account_infos.push(authority.clone());
        }
        account_infos.push(self.system_program.clone());
        if let Some(log_wrapper) = self.log_wrapper {
            account_infos.push(log_wrapper.clone());
        }
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

/// Instruction builder for `AddCollectionExternalPluginV1` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` collection
///   1. `[writable, signer]` payer
///   2. `[signer, optional]` authority
///   3. `[]` system_program
///   4. `[optional]` log_wrapper
pub struct AddCollectionExternalPluginV1CpiBuilder<'a, 'b> {
    instruction: Box<AddCollectionExternalPluginV1CpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> AddCollectionExternalPluginV1CpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(AddCollectionExternalPluginV1CpiBuilderInstruction {
            __program: program,
            collection: None,
            payer: None,
            authority: None,
            system_program: None,
            log_wrapper: None,
            external_plugin_init_info: None,
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
    /// The account paying for the storage fees
    #[inline(always)]
    pub fn payer(&mut self, payer: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.payer = Some(payer);
        self
    }
    /// `[optional account]`
    /// The owner or delegate of the asset
    #[inline(always)]
    pub fn authority(
        &mut self,
        authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ) -> &mut Self {
        self.instruction.authority = authority;
        self
    }
    /// The system program
    #[inline(always)]
    pub fn system_program(
        &mut self,
        system_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.system_program = Some(system_program);
        self
    }
    /// `[optional account]`
    /// The SPL Noop Program
    #[inline(always)]
    pub fn log_wrapper(
        &mut self,
        log_wrapper: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ) -> &mut Self {
        self.instruction.log_wrapper = log_wrapper;
        self
    }
    #[inline(always)]
    pub fn external_plugin_init_info(
        &mut self,
        external_plugin_init_info: ExternalPluginInitInfo,
    ) -> &mut Self {
        self.instruction.external_plugin_init_info = Some(external_plugin_init_info);
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
        let args = AddCollectionExternalPluginV1InstructionArgs {
            external_plugin_init_info: self
                .instruction
                .external_plugin_init_info
                .clone()
                .expect("external_plugin_init_info is not set"),
        };
        let instruction = AddCollectionExternalPluginV1Cpi {
            __program: self.instruction.__program,

            collection: self.instruction.collection.expect("collection is not set"),

            payer: self.instruction.payer.expect("payer is not set"),

            authority: self.instruction.authority,

            system_program: self
                .instruction
                .system_program
                .expect("system_program is not set"),

            log_wrapper: self.instruction.log_wrapper,
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

struct AddCollectionExternalPluginV1CpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    collection: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    payer: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    log_wrapper: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    external_plugin_init_info: Option<ExternalPluginInitInfo>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
