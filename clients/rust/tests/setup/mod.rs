use mpl_core::{
    accounts::BaseAsset,
    instructions::CreateBuilder,
    types::{DataState, Key, PluginAuthorityPair, UpdateAuthority},
};
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, transaction::Transaction,
};

pub fn program_test() -> ProgramTest {
    ProgramTest::new("mpl_core_program", mpl_core::ID, None)
}

const DEFAULT_ASSET_NAME: &str = "Test Asset";
const DEFAULT_ASSET_URI: &str = "https://example.com/asset";

pub struct CreateAssetHelperArgs<'a> {
    pub owner: Option<Pubkey>,
    pub payer: Option<Pubkey>,
    pub asset: &'a Keypair,
    pub data_state: Option<DataState>,
    pub name: Option<String>,
    pub uri: Option<String>,
    pub authority: Option<Pubkey>,
    pub update_authority: Option<Pubkey>,
    pub collection: Option<Pubkey>,
    // TODO use PluginList type here
    pub plugins: Vec<PluginAuthorityPair>,
}

pub async fn create_asset<'a>(
    context: &mut ProgramTestContext,
    input: CreateAssetHelperArgs<'a>,
) -> Result<(), BanksClientError> {
    let payer = input.payer.unwrap_or(context.payer.pubkey());
    let create_ix = CreateBuilder::new()
        .asset(input.asset.pubkey())
        .collection(input.collection)
        .authority(input.authority)
        .payer(payer)
        .owner(input.owner)
        .update_authority(input.update_authority)
        .system_program(system_program::ID)
        .data_state(input.data_state.unwrap_or(DataState::AccountState))
        .name(input.name.unwrap_or(DEFAULT_ASSET_NAME.to_owned()))
        .uri(input.uri.unwrap_or(DEFAULT_ASSET_URI.to_owned()))
        .plugins(input.plugins)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[input.asset, &context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub struct AssertAssetHelperArgs {
    pub asset: Pubkey,
    pub owner: Pubkey,
    pub update_authority: Option<UpdateAuthority>,
    pub name: Option<String>,
    pub uri: Option<String>,
    // TODO use PluginList type here
    pub plugins: Vec<PluginAuthorityPair>,
}

pub async fn assert_asset(context: &mut ProgramTestContext, input: AssertAssetHelperArgs) {
    let asset_account = context
        .banks_client
        .get_account(input.asset)
        .await
        .expect("get_account")
        .expect("asset account not found");

    let asset = BaseAsset::from_bytes(&asset_account.data).unwrap();
    assert_eq!(asset.key, Key::Asset);
    assert_eq!(
        asset.name,
        input.name.unwrap_or(DEFAULT_ASSET_NAME.to_owned())
    );
    assert_eq!(asset.uri, input.uri.unwrap_or(DEFAULT_ASSET_URI.to_owned()));
    assert_eq!(asset.owner, input.owner);
    if let Some(update_authority) = input.update_authority {
        assert_eq!(asset.update_authority, update_authority);
    }
}
