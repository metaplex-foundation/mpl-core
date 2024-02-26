use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::{assert_signer, resize_or_reallocate_account_raw};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memcpy,
};

use crate::{
    error::MplAssetError,
    instruction::accounts::UpdateAccounts,
    plugins::{CheckResult, Plugin, RegistryData, RegistryRecord, ValidationResult},
    state::{Asset, DataBlob, SolanaAccount},
    utils::fetch_core_data,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct UpdateArgs {
    pub new_name: Option<String>,
    pub new_uri: Option<String>,
}

pub(crate) fn update<'a>(accounts: &'a [AccountInfo<'a>], args: UpdateArgs) -> ProgramResult {
    // Accounts.
    let ctx = UpdateAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = match ctx.accounts.payer {
        Some(payer) => {
            assert_signer(payer)?;
            payer
        }
        None => ctx.accounts.authority,
    };

    let (mut asset, plugin_header, plugin_registry) = fetch_core_data(ctx.accounts.asset_address)?;
    let asset_size = asset.get_size() as isize;

    let mut approved = false;
    match Asset::check_update() {
        CheckResult::CanApprove => {
            if asset.validate_update(&ctx.accounts)? == ValidationResult::Approved {
                approved = true;
            }
        }
        CheckResult::CanReject => return Err(MplAssetError::InvalidAuthority.into()),
        CheckResult::None => (),
    };

    if let Some(plugin_registry) = plugin_registry.clone() {
        for record in plugin_registry.registry {
            if matches!(
                record.plugin_type.check_transfer(),
                CheckResult::CanApprove | CheckResult::CanReject
            ) {
                let result = Plugin::load(ctx.accounts.asset_address, record.data.offset)?
                    .validate_update(&ctx.accounts, &args, &record.data.authorities)?;
                if result == ValidationResult::Rejected {
                    return Err(MplAssetError::InvalidAuthority.into());
                } else if result == ValidationResult::Approved {
                    approved = true;
                }
            }
        }
    };

    if !approved {
        return Err(MplAssetError::InvalidAuthority.into());
    }

    let mut dirty = false;
    if let Some(new_update_authority) = ctx.accounts.new_update_authority {
        asset.update_authority = *new_update_authority.key;
        dirty = true;
    }
    if let Some(new_name) = &args.new_name {
        asset.name = new_name.clone();
        dirty = true;
    }
    if let Some(new_uri) = &args.new_uri {
        asset.uri = new_uri.clone();
        dirty = true;
    }
    if dirty {
        if let (Some(mut plugin_header), Some(mut plugin_registry)) =
            (plugin_header, plugin_registry)
        {
            let new_asset_size = asset.get_size() as isize;
            let size_diff = new_asset_size
                .checked_sub(asset_size)
                .ok_or(MplAssetError::NumericalOverflow)?;
            let new_size = (ctx.accounts.asset_address.data_len() as isize)
                .checked_add(size_diff)
                .ok_or(MplAssetError::NumericalOverflow)?;
            let new_registry_offset = (plugin_header.plugin_registry_offset as isize)
                .checked_add(size_diff)
                .ok_or(MplAssetError::NumericalOverflow)?;
            let registry_offset = plugin_header.plugin_registry_offset;
            plugin_header.plugin_registry_offset = new_registry_offset as usize;

            let plugin_offset = asset_size
                .checked_add(size_diff)
                .ok_or(MplAssetError::NumericalOverflow)?;
            let new_plugin_offset = new_asset_size
                .checked_add(size_diff)
                .ok_or(MplAssetError::NumericalOverflow)?;

            // //TODO: This is memory intensive, we should use memmove instead probably.
            let src = ctx.accounts.asset_address.data.borrow()
                [(plugin_offset as usize)..registry_offset]
                .to_vec();

            resize_or_reallocate_account_raw(
                ctx.accounts.asset_address,
                payer,
                ctx.accounts.system_program,
                new_size as usize,
            )?;

            sol_memcpy(
                &mut ctx.accounts.asset_address.data.borrow_mut()[(new_plugin_offset as usize)..],
                &src,
                src.len(),
            );

            plugin_header.save(ctx.accounts.asset_address, new_asset_size as usize)?;
            plugin_registry.registry = plugin_registry
                .registry
                .iter_mut()
                .map(|record| {
                    let new_offset = (record.data.offset as isize)
                        .checked_add(size_diff)
                        .ok_or(MplAssetError::NumericalOverflow)?;
                    Ok(RegistryRecord {
                        plugin_type: record.plugin_type,
                        data: RegistryData {
                            offset: new_offset as usize,
                            authorities: record.data.authorities.clone(),
                        },
                    })
                })
                .collect::<Result<Vec<_>, MplAssetError>>()?;
            plugin_registry.save(ctx.accounts.asset_address, new_registry_offset as usize)?;
        } else {
            resize_or_reallocate_account_raw(
                ctx.accounts.asset_address,
                payer,
                ctx.accounts.system_program,
                asset.get_size(),
            )?;
        }

        asset.save(ctx.accounts.asset_address, 0)?;
    }

    Ok(())
}
