use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::BurnAccounts,
    plugins::{CheckResult, Plugin, PluginType, ValidationResult},
    state::{Asset, Authority, Compressible, CompressionProof, Key},
    utils::{close_program_account, fetch_core_data, load_key, verify_proof},
};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct BurnArgs {
    compression_proof: Option<CompressionProof>,
}

pub(crate) fn burn<'a>(accounts: &'a [AccountInfo<'a>], args: BurnArgs) -> ProgramResult {
    // Accounts.
    let ctx = BurnAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    if let Some(payer) = ctx.accounts.payer {
        assert_signer(payer)?;
    }

    let mut approved = false;

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
                asset.wrap()?;

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
                match asset.validate_burn(&ctx.accounts)? {
                    ValidationResult::Approved => {
                        approved = true;
                    }
                    ValidationResult::Rejected => return Err(MplCoreError::InvalidAuthority.into()),
                    ValidationResult::Pass => (),
                }

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
                plugin_type.check_burn(),
                CheckResult::CanApprove | CheckResult::CanReject
            ) {
                let result = plugin.validate_burn(&ctx.accounts, &args, &authorities)?;

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

    close_program_account(ctx.accounts.asset_address, ctx.accounts.authority)
}
