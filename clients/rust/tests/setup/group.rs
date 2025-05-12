use mpl_core::instructions::CreateGroupBuilder;
use solana_program::system_program;
use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

/// Default limit values used by the generic `create_test_group` helper.
const DEFAULT_MAX_COLLECTIONS: u32 = 10;
const DEFAULT_MAX_ASSETS: u32 = 10;
const DEFAULT_MAX_GROUPS: u32 = 5;
const DEFAULT_MAX_PLUGINS: u32 = 3;

/// Creates a `GroupV1` account on-chain for use in tests.
///
/// All capacity limits are optional. Any `None` value will be substituted with
/// a reasonable default so that the caller can concentrate on only the limits
/// relevant to its scenario.
///
/// The helper makes the newly-created `group` account a signer so that the
/// system program can allocate and assign it during the CPI.
#[allow(clippy::too_many_arguments)]
pub async fn create_test_group(
    context: &mut ProgramTestContext,
    group: &Keypair,
    update_authority: Option<Pubkey>,
    max_collections: Option<u32>,
    max_assets: Option<u32>,
    max_groups: Option<u32>,
    max_plugins: Option<u32>,
) -> Result<(), BanksClientError> {
    let mut create_group_ix = CreateGroupBuilder::new()
        .payer(context.payer.pubkey())
        .group(group.pubkey())
        .update_authority(update_authority.unwrap_or(context.payer.pubkey()))
        .system_program(system_program::ID)
        .name("Test Group".to_owned())
        .uri("https://example.com/group".to_owned())
        .max_collections(max_collections.unwrap_or(DEFAULT_MAX_COLLECTIONS))
        .max_assets(max_assets.unwrap_or(DEFAULT_MAX_ASSETS))
        .max_groups(max_groups.unwrap_or(DEFAULT_MAX_GROUPS))
        .max_plugins(max_plugins.unwrap_or(DEFAULT_MAX_PLUGINS))
        .initial_collections(vec![])
        .initial_parent_groups(vec![])
        .initial_assets(vec![])
        .instruction();

    // The on-chain handler expects the new account (index 1) to be a signer so
    // that it can be created via the system program.
    if let Some(meta) = create_group_ix.accounts.get_mut(1) {
        meta.is_signer = true;
    }

    let tx = Transaction::new_signed_with_payer(
        &[create_group_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, group],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}
