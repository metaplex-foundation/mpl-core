use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplAssetError,
    instruction::accounts::UpdateAccounts,
    plugins::{CheckResult, Plugin, ValidationResult},
    state::{Asset, SolanaAccount},
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
    if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
    }

    let (mut asset, _, plugin_registry) = fetch_core_data(ctx.accounts.asset_address)?;

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

    if let Some(plugin_registry) = plugin_registry {
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
        //TODO: Handle resize!
        asset.save(ctx.accounts.asset_address, 0)?;
    }

    Ok(())
}
