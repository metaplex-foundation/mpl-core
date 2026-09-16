//! Instruction builders.
//!
//! Thin wrappers over the generated Rust client (`mpl_core::instructions`),
//! one per instruction the tests use. Each takes the pubkeys the instruction
//! needs (optional accounts as `Option<Pubkey>`; the generated code fills the
//! sentinel with the program id) and the args as `mpl_core::types` values,
//! and returns a complete `Instruction`. The system program is always the
//! real one; use [`set_account`] to substitute a wrong one.
//!
//! To add an instruction: copy the closest wrapper below, swap in the
//! generated `XxxV1 { .. }` accounts struct and `XxxV1InstructionArgs`, and
//! keep the parameter order accounts-then-args.
//!
//! [`raw`] / [`raw_bytes`] exist for deliberately malformed instructions,
//! [`convert`] moves values between the program's and the client's
//! borsh-compatible types.

use {
    borsh::{BorshDeserialize, BorshSerialize},
    mpl_core::{
        instructions::{
            AddCollectionExternalPluginAdapterV1,
            AddCollectionExternalPluginAdapterV1InstructionArgs, AddCollectionPluginV1,
            AddCollectionPluginV1InstructionArgs, AddExternalPluginAdapterV1,
            AddExternalPluginAdapterV1InstructionArgs, AddPluginV1, AddPluginV1InstructionArgs,
            ApprovePluginAuthorityV1, ApprovePluginAuthorityV1InstructionArgs, BurnCollectionV1,
            BurnCollectionV1InstructionArgs, BurnV1, BurnV1InstructionArgs, CreateCollectionV2,
            CreateCollectionV2InstructionArgs, CreateV2, CreateV2InstructionArgs, ExecuteV1,
            ExecuteV1InstructionArgs, RemoveCollectionPluginV1,
            RemoveCollectionPluginV1InstructionArgs, RemoveExternalPluginAdapterV1,
            RemoveExternalPluginAdapterV1InstructionArgs, RemovePluginV1,
            RemovePluginV1InstructionArgs, RevokePluginAuthorityV1,
            RevokePluginAuthorityV1InstructionArgs, TransferV1, TransferV1InstructionArgs,
            UpdateCollectionV1, UpdateCollectionV1InstructionArgs, UpdateExternalPluginAdapterV1,
            UpdateExternalPluginAdapterV1InstructionArgs, UpdatePluginV1,
            UpdatePluginV1InstructionArgs, UpdateV1, UpdateV1InstructionArgs,
            WriteExternalPluginAdapterDataV1, WriteExternalPluginAdapterDataV1InstructionArgs,
        },
        types::{
            CompressionProof, DataState, ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey,
            ExternalPluginAdapterUpdateInfo, Plugin, PluginAuthority, PluginAuthorityPair,
            PluginType, UpdateAuthority,
        },
    },
    mpl_core_program::ID as MPL_CORE_ID,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    solana_system_interface::program as system_program,
};

/// Serializes `value` with borsh and deserializes it as `T`. The program's
/// and the client's types are generated from the same IDL, so this is how a
/// `mpl_core_program::plugins::Plugin` becomes a `mpl_core::types::Plugin`
/// and back.
pub fn convert<F: BorshSerialize, T: BorshDeserialize>(value: &F) -> T {
    let bytes = borsh::to_vec(value)
        .unwrap_or_else(|err| panic!("failed to serialize {}: {err}", std::any::type_name::<F>()));
    T::try_from_slice(&bytes).unwrap_or_else(|err| {
        panic!(
            "failed to deserialize {} as {}: {err} (bytes: {bytes:?})",
            std::any::type_name::<F>(),
            std::any::type_name::<T>()
        )
    })
}

/// An mpl-core instruction with a hand-picked discriminator and borsh args.
pub fn raw(discriminator: u8, args: &impl BorshSerialize, metas: Vec<AccountMeta>) -> Instruction {
    let mut data = vec![discriminator];
    data.extend(borsh::to_vec(args).expect("args serialize"));
    raw_bytes(data, metas)
}

/// An mpl-core instruction with arbitrary data bytes.
pub fn raw_bytes(data: Vec<u8>, metas: Vec<AccountMeta>) -> Instruction {
    Instruction {
        program_id: MPL_CORE_ID,
        accounts: metas,
        data,
    }
}

/// Clears `is_signer` on every meta for `key`. Mollusk performs no signature
/// verification, so this is how a "missing signer" case is expressed.
pub fn unsign(ix: &mut Instruction, key: &Pubkey) {
    let mut found = false;
    for meta in ix.accounts.iter_mut().filter(|meta| meta.pubkey == *key) {
        meta.is_signer = false;
        found = true;
    }
    assert!(found, "unsign: {key} is not an account of the instruction");
}

/// Replaces the account meta at `index`.
pub fn set_account(ix: &mut Instruction, index: usize, meta: AccountMeta) {
    assert!(
        index < ix.accounts.len(),
        "set_account: index {index} is out of range for {} accounts",
        ix.accounts.len()
    );
    ix.accounts[index] = meta;
}

/// Appends remaining accounts (oracle accounts, delegate records, ...).
pub fn with_remaining(
    mut ix: Instruction,
    metas: impl IntoIterator<Item = AccountMeta>,
) -> Instruction {
    ix.accounts.extend(metas);
    ix
}

/// `CreateV2`.
#[allow(clippy::too_many_arguments)] // one parameter per instruction account / arg
pub fn create_v2(
    asset: Pubkey,
    collection: Option<Pubkey>,
    authority: Option<Pubkey>,
    payer: Pubkey,
    owner: Option<Pubkey>,
    update_authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    name: &str,
    uri: &str,
    plugins: Option<Vec<PluginAuthorityPair>>,
    external_plugin_adapters: Option<Vec<ExternalPluginAdapterInitInfo>>,
    remaining: &[AccountMeta],
) -> Instruction {
    CreateV2 {
        asset,
        collection,
        authority,
        payer,
        owner,
        update_authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction_with_remaining_accounts(
        CreateV2InstructionArgs {
            data_state: DataState::AccountState,
            name: name.to_string(),
            uri: uri.to_string(),
            plugins,
            external_plugin_adapters,
        },
        remaining,
    )
}

/// `CreateCollectionV2`.
#[allow(clippy::too_many_arguments)] // one parameter per instruction account / arg
pub fn create_collection_v2(
    collection: Pubkey,
    update_authority: Option<Pubkey>,
    payer: Pubkey,
    name: &str,
    uri: &str,
    plugins: Option<Vec<PluginAuthorityPair>>,
    external_plugin_adapters: Option<Vec<ExternalPluginAdapterInitInfo>>,
    remaining: &[AccountMeta],
) -> Instruction {
    CreateCollectionV2 {
        collection,
        update_authority,
        payer,
        system_program: system_program::ID,
    }
    .instruction_with_remaining_accounts(
        CreateCollectionV2InstructionArgs {
            name: name.to_string(),
            uri: uri.to_string(),
            plugins,
            external_plugin_adapters,
        },
        remaining,
    )
}

/// `TransferV1`.
pub fn transfer_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Option<Pubkey>,
    new_owner: Pubkey,
    log_wrapper: Option<Pubkey>,
    compression_proof: Option<CompressionProof>,
) -> Instruction {
    TransferV1 {
        asset,
        collection,
        payer,
        authority,
        new_owner,
        system_program: Some(system_program::ID),
        log_wrapper,
    }
    .instruction(TransferV1InstructionArgs { compression_proof })
}

/// `BurnV1`.
pub fn burn_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    compression_proof: Option<CompressionProof>,
) -> Instruction {
    BurnV1 {
        asset,
        collection,
        payer,
        authority,
        system_program: Some(system_program::ID),
        log_wrapper,
    }
    .instruction(BurnV1InstructionArgs { compression_proof })
}

/// `BurnCollectionV1`.
pub fn burn_collection_v1(
    collection: Pubkey,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    compression_proof: Option<CompressionProof>,
) -> Instruction {
    BurnCollectionV1 {
        collection,
        payer,
        authority,
        log_wrapper,
    }
    .instruction(BurnCollectionV1InstructionArgs { compression_proof })
}

/// `UpdateV1`.
#[allow(clippy::too_many_arguments)] // one parameter per instruction account / arg
pub fn update_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    new_name: Option<String>,
    new_uri: Option<String>,
    new_update_authority: Option<UpdateAuthority>,
) -> Instruction {
    UpdateV1 {
        asset,
        collection,
        payer,
        authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(UpdateV1InstructionArgs {
        new_name,
        new_uri,
        new_update_authority,
    })
}

/// `UpdateCollectionV1`.
pub fn update_collection_v1(
    collection: Pubkey,
    payer: Pubkey,
    authority: Option<Pubkey>,
    new_update_authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    new_name: Option<String>,
    new_uri: Option<String>,
) -> Instruction {
    UpdateCollectionV1 {
        collection,
        payer,
        authority,
        new_update_authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(UpdateCollectionV1InstructionArgs { new_name, new_uri })
}

/// `AddPluginV1`.
pub fn add_plugin_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    plugin: Plugin,
    init_authority: Option<PluginAuthority>,
) -> Instruction {
    AddPluginV1 {
        asset,
        collection,
        payer,
        authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(AddPluginV1InstructionArgs {
        plugin,
        init_authority,
    })
}

/// `AddCollectionPluginV1`.
pub fn add_collection_plugin_v1(
    collection: Pubkey,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    plugin: Plugin,
    init_authority: Option<PluginAuthority>,
) -> Instruction {
    AddCollectionPluginV1 {
        collection,
        payer,
        authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(AddCollectionPluginV1InstructionArgs {
        plugin,
        init_authority,
    })
}

/// `RemovePluginV1`.
pub fn remove_plugin_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    plugin_type: PluginType,
) -> Instruction {
    RemovePluginV1 {
        asset,
        collection,
        payer,
        authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(RemovePluginV1InstructionArgs { plugin_type })
}

/// `RemoveCollectionPluginV1`.
pub fn remove_collection_plugin_v1(
    collection: Pubkey,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    plugin_type: PluginType,
) -> Instruction {
    RemoveCollectionPluginV1 {
        collection,
        payer,
        authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(RemoveCollectionPluginV1InstructionArgs { plugin_type })
}

/// `UpdatePluginV1`.
pub fn update_plugin_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    plugin: Plugin,
) -> Instruction {
    UpdatePluginV1 {
        asset,
        collection,
        payer,
        authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(UpdatePluginV1InstructionArgs { plugin })
}

/// `ApprovePluginAuthorityV1`.
pub fn approve_plugin_authority_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    plugin_type: PluginType,
    new_authority: PluginAuthority,
) -> Instruction {
    ApprovePluginAuthorityV1 {
        asset,
        collection,
        payer,
        authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(ApprovePluginAuthorityV1InstructionArgs {
        plugin_type,
        new_authority,
    })
}

/// `RevokePluginAuthorityV1`.
pub fn revoke_plugin_authority_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    plugin_type: PluginType,
) -> Instruction {
    RevokePluginAuthorityV1 {
        asset,
        collection,
        payer,
        authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(RevokePluginAuthorityV1InstructionArgs { plugin_type })
}

/// `AddExternalPluginAdapterV1`.
pub fn add_external_plugin_adapter_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    init_info: ExternalPluginAdapterInitInfo,
) -> Instruction {
    AddExternalPluginAdapterV1 {
        asset,
        collection,
        payer,
        authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(AddExternalPluginAdapterV1InstructionArgs { init_info })
}

/// `AddCollectionExternalPluginAdapterV1`.
pub fn add_collection_external_plugin_adapter_v1(
    collection: Pubkey,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    init_info: ExternalPluginAdapterInitInfo,
) -> Instruction {
    AddCollectionExternalPluginAdapterV1 {
        collection,
        payer,
        authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(AddCollectionExternalPluginAdapterV1InstructionArgs { init_info })
}

/// `UpdateExternalPluginAdapterV1`.
pub fn update_external_plugin_adapter_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    key: ExternalPluginAdapterKey,
    update_info: ExternalPluginAdapterUpdateInfo,
) -> Instruction {
    UpdateExternalPluginAdapterV1 {
        asset,
        collection,
        payer,
        authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(UpdateExternalPluginAdapterV1InstructionArgs { key, update_info })
}

/// `RemoveExternalPluginAdapterV1`.
pub fn remove_external_plugin_adapter_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    key: ExternalPluginAdapterKey,
) -> Instruction {
    RemoveExternalPluginAdapterV1 {
        asset,
        collection,
        payer,
        authority,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(RemoveExternalPluginAdapterV1InstructionArgs { key })
}

/// `WriteExternalPluginAdapterDataV1`. Pass either inline `data` or a
/// `buffer` account.
#[allow(clippy::too_many_arguments)] // one parameter per instruction account / arg
pub fn write_external_plugin_adapter_data_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Option<Pubkey>,
    buffer: Option<Pubkey>,
    log_wrapper: Option<Pubkey>,
    key: ExternalPluginAdapterKey,
    data: Option<Vec<u8>>,
) -> Instruction {
    WriteExternalPluginAdapterDataV1 {
        asset,
        collection,
        payer,
        authority,
        buffer,
        system_program: system_program::ID,
        log_wrapper,
    }
    .instruction(WriteExternalPluginAdapterDataV1InstructionArgs { key, data })
}

/// `ExecuteV1`. `remaining` are the accounts forwarded to `program_id`
/// (preceded by an execution delegate record when one is used).
#[allow(clippy::too_many_arguments)] // one parameter per instruction account / arg
pub fn execute_v1(
    asset: Pubkey,
    collection: Option<Pubkey>,
    asset_signer: Pubkey,
    payer: Pubkey,
    authority: Option<Pubkey>,
    program_id: Pubkey,
    instruction_data: Vec<u8>,
    remaining: &[AccountMeta],
) -> Instruction {
    ExecuteV1 {
        asset,
        collection,
        asset_signer,
        payer: (payer, true),
        authority,
        system_program: system_program::ID,
        program_id,
    }
    .instruction_with_remaining_accounts(ExecuteV1InstructionArgs { instruction_data }, remaining)
}
