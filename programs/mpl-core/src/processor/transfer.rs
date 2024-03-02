use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::TransferAccounts,
    plugins::{CheckResult, Plugin, PluginType, ValidationResult},
    state::{Asset, Authority, Compressible, CompressionProof, HashedAsset, Key, SolanaAccount},
    utils::{fetch_core_data, load_key, verify_proof},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct TransferArgs {
    compression_proof: Option<CompressionProof>,
}

pub(crate) fn transfer<'a>(accounts: &'a [AccountInfo<'a>], args: TransferArgs) -> ProgramResult {
    // Accounts.
    let ctx = TransferAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
    }

    let mut approved = false;
    let (mut asset_outer, mut key_type) = (None, Key::HashedAsset);
    let plugins: Option<Vec<(Plugin, PluginType, Vec<Authority>)>> =
        match load_key(ctx.accounts.asset_address, 0)? {
            Key::HashedAsset => {
                let compression_proof = args
                    .compression_proof
                    .as_ref()
                    .ok_or(MplCoreError::MissingCompressionProof)?;
                let (asset, plugin_schemes) =
                    verify_proof(ctx.accounts.asset_address, compression_proof)?;
                if ctx.accounts.authority.key != &asset.owner {
                    return Err(MplCoreError::InvalidAuthority.into());
                }
                asset_outer = Some(asset);

                Some(
                    plugin_schemes
                        .into_iter()
                        .map(|plugin_schema| {
                            (
                                plugin_schema.plugin.clone(),
                                PluginType::from(&plugin_schema.plugin),
                                plugin_schema.authorities,
                            )
                        })
                        .collect(),
                )
            }
            Key::Asset => {
                let (asset, _, plugin_registry) =
                    fetch_core_data::<Asset>(ctx.accounts.asset_address)?;
                match asset.validate_transfer(&ctx.accounts)? {
                    ValidationResult::Approved => {
                        approved = true;
                    }
                    ValidationResult::Rejected => return Err(MplCoreError::InvalidAuthority.into()),
                    ValidationResult::Pass => (),
                }
                (asset_outer, key_type) = (Some(asset), Key::Asset);

                if let Some(plugin_registry) = plugin_registry {
                    let mut plugins = vec![];

                    for record in plugin_registry.registry {
                        let plugin = Plugin::load(ctx.accounts.asset_address, record.offset)?;
                        plugins.push((plugin, record.plugin_type, record.authorities))
                    }
                    Some(plugins)
                } else {
                    None
                }
            }
            _ => return Err(MplCoreError::IncorrectAccount.into()),
        };

    if plugins.is_some() {
        for (plugin, plugin_type, authorities) in plugins.unwrap() {
            if matches!(
                plugin_type.check_transfer(),
                CheckResult::CanApprove | CheckResult::CanReject
            ) {
                let result = plugin.validate_transfer(&ctx.accounts, &args, &authorities)?;

                match result {
                    ValidationResult::Approved => approved = true,
                    ValidationResult::Pass => (),
                    ValidationResult::Rejected => return Err(MplCoreError::InvalidAuthority.into()),
                }
            }
        }
    }

    if !approved {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // cannot be None
    let mut asset = asset_outer.unwrap();
    asset.owner = *ctx.accounts.new_owner.key;
    match key_type {
        Key::Asset => asset.save(ctx.accounts.asset_address, 0),
        Key::HashedAsset => HashedAsset::new(asset.hash()?).save(ctx.accounts.asset_address, 0),
        _ => unreachable!()
    }
}
