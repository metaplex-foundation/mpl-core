use mpl_core::{
    instructions::CreateBuilder,
    types::{DataState, Key, Plugin, PluginAuthorityPair, UpdateAuthority},
    Asset,
};
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction, system_program,
    transaction::Transaction,
};

pub fn program_test() -> ProgramTest {
    ProgramTest::new("mpl_core_program", mpl_core::ID, None)
}

const DEFAULT_ASSET_NAME: &str = "Test Asset";
const DEFAULT_ASSET_URI: &str = "https://example.com/asset";

pub struct CreateAssetHelperArgs<'a> {
    pub owner: Option<Pubkey>,
    pub payer: Option<&'a Keypair>,
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
    let payer = input.payer.unwrap_or(&context.payer);
    let create_ix = CreateBuilder::new()
        .asset(input.asset.pubkey())
        .collection(input.collection)
        .authority(input.authority)
        .payer(payer.pubkey())
        .owner(input.owner)
        .update_authority(input.update_authority)
        .system_program(system_program::ID)
        .data_state(input.data_state.unwrap_or(DataState::AccountState))
        .name(input.name.unwrap_or(DEFAULT_ASSET_NAME.to_owned()))
        .uri(input.uri.unwrap_or(DEFAULT_ASSET_URI.to_owned()))
        .plugins(input.plugins)
        .instruction();

    let mut signers = vec![input.asset, &context.payer];
    if let Some(payer) = input.payer {
        signers.push(payer);
    }

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        signers.as_slice(),
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

    let asset = Asset::from_bytes(&asset_account.data).unwrap();
    assert_eq!(asset.base.key, Key::Asset);
    assert_eq!(
        asset.base.name,
        input.name.unwrap_or(DEFAULT_ASSET_NAME.to_owned())
    );
    assert_eq!(
        asset.base.uri,
        input.uri.unwrap_or(DEFAULT_ASSET_URI.to_owned())
    );
    assert_eq!(asset.base.owner, input.owner);
    if let Some(update_authority) = input.update_authority {
        assert_eq!(asset.base.update_authority, update_authority);
    }

    for plugin in input.plugins {
        match plugin {
            PluginAuthorityPair {
                plugin: Plugin::Freeze(freeze),
                authority,
            } => {
                let plugin = asset.plugin_list.freeze.clone().unwrap();
                if let Some(authority) = authority {
                    assert_eq!(plugin.base.authority, authority.into());
                }
                assert_eq!(plugin.freeze, freeze);
            }
            _ => panic!("unsupported plugin type"),
        }
    }
}
pub async fn airdrop(
    context: &mut ProgramTestContext,
    receiver: &Pubkey,
    amount: u64,
) -> Result<(), BanksClientError> {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            receiver,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
    Ok(())
}
