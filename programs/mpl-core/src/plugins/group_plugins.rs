use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_memory::sol_memcpy, pubkey::Pubkey, system_program,
};

use mpl_utils::assert_signer;

use crate::error::MplCoreError;
use crate::state::{AllowedPlugin, GroupV1, PluginAuthorityType, SolanaAccount};

use borsh::{BorshDeserialize, BorshSerialize};

/// Return `true` if the provided plugin type string is supported at the group
/// level. Plugin programs SHOULD use the same canonical string identifiers
/// when creating or approving group-scoped plugins.
///
/// NOTE: Currently only a very small allow-list is required. Keeping this
/// logic here – alongside the group-plugin instruction handlers – ensures the
/// policy remains centralised and easy to evolve without touching multiple
/// modules.
pub fn is_plugin_allowed_for_groups(plugin_type: &str) -> bool {
    matches!(plugin_type, "attributes" | "verified_creator" | "autograph")
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddGroupPluginArgs {
    /// String identifier describing the plugin type (e.g. "attributes").
    pub plugin_type: String,
    /// Defines who may manage the plugin after creation.
    pub authority_type: PluginAuthorityType,
    /// Opaque plugin-specific initialization data.
    pub plugin_args: Vec<u8>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateGroupPluginArgs {
    /// Opaque plugin-specific update data.
    pub plugin_args: Vec<u8>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct ApproveGroupPluginArgs {
    /// Plugin-specific data supplied during approval – semantics defined by
    /// the plugin program.
    pub plugin_args: Vec<u8>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_group_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddGroupPluginArgs,
) -> ProgramResult {
    if accounts.len() < 5 {
        return Err(MplCoreError::MissingSystemProgram.into());
    }

    let authority_info = accounts
        .get(0)
        .ok_or::<ProgramError>(MplCoreError::MissingSigner.into())?;
    let group_info = accounts
        .get(1)
        .ok_or::<ProgramError>(MplCoreError::GroupNotFound.into())?;
    let _plugin_account_info = accounts
        .get(2)
        .ok_or::<ProgramError>(MplCoreError::PluginNotFound.into())?;
    let plugin_program_info = accounts
        .get(3)
        .ok_or::<ProgramError>(MplCoreError::InvalidPlugin.into())?;
    let system_program_info = accounts
        .get(4)
        .ok_or::<ProgramError>(MplCoreError::MissingSystemProgram.into())?;

    assert_signer(authority_info)?;

    if system_program_info.key != &system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    let mut group = GroupV1::load(group_info, 0)?;
    group.validate_authority(authority_info.key)?;

    if !is_plugin_allowed_for_groups(&args.plugin_type) {
        return Err(MplCoreError::GroupPluginNotAllowed.into());
    }

    if group
        .allowed_plugins
        .iter()
        .any(|p| p.address == *plugin_program_info.key)
    {
        return Err(MplCoreError::PluginAlreadyExists.into());
    }

    let allowed_plugin = AllowedPlugin {
        address: *plugin_program_info.key,
        authority_type: args.authority_type,
    };
    group.allowed_plugins.push(allowed_plugin);

    let serialized_group = group.try_to_vec()?;
    if serialized_group.len() > group_info.data_len() {
        return Err(MplCoreError::NumericalOverflowError.into());
    }

    sol_memcpy(
        &mut group_info.try_borrow_mut_data()?[..serialized_group.len()],
        &serialized_group,
        serialized_group.len(),
    );

    Ok(())
}

pub(crate) fn remove_group_plugin<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    if accounts.len() < 3 {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let authority_info = accounts.get(0).ok_or(MplCoreError::MissingSigner)?;
    let group_info = accounts.get(1).ok_or(MplCoreError::GroupNotFound)?;
    let plugin_account_info = accounts.get(2).ok_or(MplCoreError::PluginNotFound)?;

    assert_signer(authority_info)?;

    let mut group = GroupV1::load(group_info, 0)?;
    group.validate_authority(authority_info.key)?;

    let pos = group
        .allowed_plugins
        .iter()
        .position(|p| p.address == *plugin_account_info.key)
        .ok_or(MplCoreError::PluginNotFound)?;
    group.allowed_plugins.swap_remove(pos);

    let serialized_group = group.try_to_vec()?;
    if serialized_group.len() > group_info.data_len() {
        return Err(MplCoreError::NumericalOverflowError.into());
    }

    {
        let mut data = group_info.try_borrow_mut_data()?;
        for byte in data.iter_mut() {
            *byte = 0;
        }
        sol_memcpy(
            &mut data[..serialized_group.len()],
            &serialized_group,
            serialized_group.len(),
        );
    }

    Ok(())
}

pub(crate) fn update_group_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: UpdateGroupPluginArgs,
) -> ProgramResult {
    if accounts.len() < 3 {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let authority_info = accounts.get(0).ok_or(MplCoreError::MissingSigner)?;
    let group_info = accounts.get(1).ok_or(MplCoreError::GroupNotFound)?;
    let plugin_account_info = accounts.get(2).ok_or(MplCoreError::PluginNotFound)?;

    assert_signer(authority_info)?;

    let group = GroupV1::load(group_info, 0)?;

    let allowed_plugin = group
        .allowed_plugins
        .iter()
        .find(|p| p.address == *plugin_account_info.key)
        .ok_or(MplCoreError::PluginNotFound)?;

    match &allowed_plugin.authority_type {
        PluginAuthorityType::None => return Err(MplCoreError::InvalidAuthority.into()),
        PluginAuthorityType::UpdateAuthority => group.validate_authority(authority_info.key)?,
        PluginAuthorityType::SpecificAddress(addr) => {
            if addr != authority_info.key {
                return Err(MplCoreError::InvalidAuthority.into());
            }
        }
    }

    Ok(())
}

pub(crate) fn approve_group_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: ApproveGroupPluginArgs,
) -> ProgramResult {
    if accounts.len() < 3 {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let authority_info = accounts.get(0).ok_or(MplCoreError::MissingSigner)?;
    let group_info = accounts.get(1).ok_or(MplCoreError::GroupNotFound)?;
    let plugin_account_info = accounts.get(2).ok_or(MplCoreError::PluginNotFound)?;

    assert_signer(authority_info)?;

    let mut group = GroupV1::load(group_info, 0)?;
    group.validate_authority(authority_info.key)?;

    let plugin_entry = group
        .allowed_plugins
        .iter_mut()
        .find(|p| p.address == *plugin_account_info.key)
        .ok_or(MplCoreError::PluginNotFound)?;

    if plugin_entry.authority_type == PluginAuthorityType::None {
        if args.plugin_args.len() == 32 {
            let pubkey_bytes: [u8; 32] = args.plugin_args[..32]
                .try_into()
                .map_err(|_| MplCoreError::InvalidPluginOperation)?;
            plugin_entry.authority_type =
                PluginAuthorityType::SpecificAddress(Pubkey::new_from_array(pubkey_bytes));
        } else {
            plugin_entry.authority_type = PluginAuthorityType::UpdateAuthority;
        }
    }

    let serialized_group = group.try_to_vec()?;
    if serialized_group.len() > group_info.data_len() {
        return Err(MplCoreError::NumericalOverflowError.into());
    }

    {
        let mut data = group_info.try_borrow_mut_data()?;
        for byte in data.iter_mut() {
            *byte = 0;
        }
        sol_memcpy(
            &mut data[..serialized_group.len()],
            &serialized_group,
            serialized_group.len(),
        );
    }

    Ok(())
}

pub(crate) fn revoke_group_plugin<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    if accounts.len() < 3 {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let authority_info = accounts.get(0).ok_or(MplCoreError::MissingSigner)?;
    let group_info = accounts.get(1).ok_or(MplCoreError::GroupNotFound)?;
    let plugin_account_info = accounts.get(2).ok_or(MplCoreError::PluginNotFound)?;

    assert_signer(authority_info)?;

    let mut group = GroupV1::load(group_info, 0)?;
    group.validate_authority(authority_info.key)?;

    let plugin_entry = group
        .allowed_plugins
        .iter_mut()
        .find(|p| p.address == *plugin_account_info.key)
        .ok_or(MplCoreError::PluginNotFound)?;

    match &plugin_entry.authority_type {
        PluginAuthorityType::None => return Err(MplCoreError::InvalidAuthority.into()),
        PluginAuthorityType::UpdateAuthority => {}
        PluginAuthorityType::SpecificAddress(addr) => {
            if addr != authority_info.key {
                return Err(MplCoreError::InvalidAuthority.into());
            }
        }
    }

    plugin_entry.authority_type = PluginAuthorityType::None;

    let serialized_group = group.try_to_vec()?;
    if serialized_group.len() > group_info.data_len() {
        return Err(MplCoreError::NumericalOverflowError.into());
    }

    {
        let mut data = group_info.try_borrow_mut_data()?;
        for byte in data.iter_mut() {
            *byte = 0;
        }
        sol_memcpy(
            &mut data[..serialized_group.len()],
            &serialized_group,
            serialized_group.len(),
        );
    }

    Ok(())
}
