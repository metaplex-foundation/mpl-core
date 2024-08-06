use mpl_core::{
    instructions::{CreateCollectionV2Builder, CreateV2Builder},
    types::{
        DataState, ExternalPluginAdapter, ExternalPluginAdapterInitInfo, Key, Plugin,
        PluginAuthorityPair, UpdateAuthority,
    },
    Asset, Collection,
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
const DEFAULT_COLLECTION_NAME: &str = "Test Collection";
const DEFAULT_COLLECTION_URI: &str = "https://example.com/collection";

#[derive(Debug)]
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
    pub external_plugin_adapters: Vec<ExternalPluginAdapterInitInfo>,
}

pub async fn create_asset<'a>(
    context: &mut ProgramTestContext,
    input: CreateAssetHelperArgs<'a>,
) -> Result<(), BanksClientError> {
    let payer = input.payer.unwrap_or(&context.payer);
    let create_ix = CreateV2Builder::new()
        .asset(input.asset.pubkey())
        .collection(input.collection)
        .authority(input.authority)
        .payer(payer.pubkey())
        .owner(Some(input.owner.unwrap_or(payer.pubkey())))
        .update_authority(input.update_authority)
        .system_program(system_program::ID)
        .data_state(input.data_state.unwrap_or(DataState::AccountState))
        .name(input.name.unwrap_or(DEFAULT_ASSET_NAME.to_owned()))
        .uri(input.uri.unwrap_or(DEFAULT_ASSET_URI.to_owned()))
        .plugins(input.plugins)
        .external_plugin_adapters(input.external_plugin_adapters)
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
    pub external_plugin_adapters: Vec<ExternalPluginAdapter>,
}

pub async fn assert_asset(context: &mut ProgramTestContext, input: AssertAssetHelperArgs) {
    let asset_account = context
        .banks_client
        .get_account(input.asset)
        .await
        .expect("get_account")
        .expect("asset account not found");

    let asset = Asset::from_bytes(&asset_account.data).unwrap();
    assert_eq!(asset.base.key, Key::AssetV1);
    assert_eq!(asset.base.owner, input.owner);
    if let Some(update_authority) = input.update_authority {
        assert_eq!(asset.base.update_authority, update_authority);
    }
    assert_eq!(
        asset.base.name,
        input.name.unwrap_or(DEFAULT_ASSET_NAME.to_owned())
    );
    assert_eq!(
        asset.base.uri,
        input.uri.unwrap_or(DEFAULT_ASSET_URI.to_owned())
    );

    for plugin in input.plugins {
        match plugin {
            PluginAuthorityPair {
                plugin: Plugin::FreezeDelegate(freeze),
                authority,
            } => {
                let plugin = asset.plugin_list.freeze_delegate.clone().unwrap();
                if let Some(authority) = authority {
                    assert_eq!(plugin.base.authority, authority.into());
                }
                assert_eq!(plugin.freeze_delegate, freeze);
            }
            PluginAuthorityPair {
                plugin: Plugin::Royalties(royalties),
                authority,
            } => {
                let plugin = asset.plugin_list.royalties.clone().unwrap();
                if let Some(authority) = authority {
                    assert_eq!(plugin.base.authority, authority.into());
                }
                assert_eq!(plugin.royalties, royalties);
            }
            _ => panic!("unsupported plugin type"),
        }
    }

    assert_eq!(
        input.external_plugin_adapters.len(),
        asset.external_plugin_adapter_list.lifecycle_hooks.len()
            + asset.external_plugin_adapter_list.oracles.len()
            + asset.external_plugin_adapter_list.app_data.len()
    );
    for plugin in input.external_plugin_adapters {
        match plugin {
            ExternalPluginAdapter::LifecycleHook(hook) => {
                assert!(asset
                    .external_plugin_adapter_list
                    .lifecycle_hooks
                    .iter()
                    .any(|lifecyle_hook_with_data| lifecyle_hook_with_data.base == hook))
            }
            ExternalPluginAdapter::Oracle(oracle) => {
                assert!(asset.external_plugin_adapter_list.oracles.contains(&oracle))
            }
            ExternalPluginAdapter::AppData(app_data) => {
                assert!(asset
                    .external_plugin_adapter_list
                    .app_data
                    .iter()
                    .any(|app_data_with_data| app_data_with_data.base == app_data))
            }
            ExternalPluginAdapter::LinkedLifecycleHook(hook) => {
                assert!(asset
                    .external_plugin_adapter_list
                    .linked_lifecycle_hooks
                    .contains(&hook))
            }
            ExternalPluginAdapter::LinkedAppData(app_data) => {
                assert!(asset
                    .external_plugin_adapter_list
                    .linked_app_data
                    .contains(&app_data))
            }
            ExternalPluginAdapter::DataSection(data) => {
                assert!(asset
                    .external_plugin_adapter_list
                    .data_sections
                    .iter()
                    .any(|data_sections_with_data| data_sections_with_data.base == data))
            }
        }
    }
}

#[derive(Debug)]
pub struct CreateCollectionHelperArgs<'a> {
    pub collection: &'a Keypair,
    pub update_authority: Option<Pubkey>,
    pub payer: Option<&'a Keypair>,
    pub name: Option<String>,
    pub uri: Option<String>,
    // TODO use PluginList type here
    pub plugins: Vec<PluginAuthorityPair>,
    pub external_plugin_adapters: Vec<ExternalPluginAdapterInitInfo>,
}

pub async fn create_collection<'a>(
    context: &mut ProgramTestContext,
    input: CreateCollectionHelperArgs<'a>,
) -> Result<(), BanksClientError> {
    let payer = input.payer.unwrap_or(&context.payer);
    let create_ix = CreateCollectionV2Builder::new()
        .collection(input.collection.pubkey())
        .update_authority(input.update_authority)
        .payer(payer.pubkey())
        .system_program(system_program::ID)
        .name(input.name.unwrap_or(DEFAULT_COLLECTION_NAME.to_owned()))
        .uri(input.uri.unwrap_or(DEFAULT_COLLECTION_URI.to_owned()))
        .plugins(input.plugins)
        .external_plugin_adapters(input.external_plugin_adapters)
        .instruction();

    let mut signers = vec![input.collection, &context.payer];
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

pub struct AssertCollectionHelperArgs {
    pub collection: Pubkey,
    pub update_authority: Pubkey,
    pub name: Option<String>,
    pub uri: Option<String>,
    pub num_minted: u32,
    pub current_size: u32,
    // TODO use PluginList type here
    pub plugins: Vec<PluginAuthorityPair>,
    pub external_plugin_adapters: Vec<ExternalPluginAdapter>,
}

pub async fn assert_collection(
    context: &mut ProgramTestContext,
    input: AssertCollectionHelperArgs,
) {
    let collection_account = context
        .banks_client
        .get_account(input.collection)
        .await
        .expect("get_account")
        .expect("collection account not found");

    let collection = Collection::from_bytes(&collection_account.data).unwrap();
    assert_eq!(collection.base.key, Key::CollectionV1);
    assert_eq!(collection.base.update_authority, input.update_authority);
    assert_eq!(
        collection.base.name,
        input.name.unwrap_or(DEFAULT_COLLECTION_NAME.to_owned())
    );
    assert_eq!(
        collection.base.uri,
        input.uri.unwrap_or(DEFAULT_COLLECTION_URI.to_owned())
    );
    assert_eq!(collection.base.num_minted, input.num_minted);
    assert_eq!(collection.base.current_size, input.current_size);

    for plugin in input.plugins {
        match plugin {
            PluginAuthorityPair {
                plugin: Plugin::FreezeDelegate(freeze),
                authority,
            } => {
                let plugin = collection.plugin_list.freeze_delegate.clone().unwrap();
                if let Some(authority) = authority {
                    assert_eq!(plugin.base.authority, authority.into());
                }
                assert_eq!(plugin.freeze_delegate, freeze);
            }
            PluginAuthorityPair {
                plugin: Plugin::Royalties(royalties),
                authority,
            } => {
                let plugin = collection.plugin_list.royalties.clone().unwrap();
                if let Some(authority) = authority {
                    assert_eq!(plugin.base.authority, authority.into());
                }
                assert_eq!(plugin.royalties, royalties);
            }
            _ => panic!("unsupported plugin type"),
        }
    }

    assert_eq!(
        input.external_plugin_adapters.len(),
        collection
            .external_plugin_adapter_list
            .lifecycle_hooks
            .len()
            + collection.external_plugin_adapter_list.oracles.len()
            + collection.external_plugin_adapter_list.app_data.len()
    );
    for plugin in input.external_plugin_adapters {
        match plugin {
            ExternalPluginAdapter::LifecycleHook(hook) => {
                assert!(collection
                    .external_plugin_adapter_list
                    .lifecycle_hooks
                    .iter()
                    .any(|lifecyle_hook_with_data| lifecyle_hook_with_data.base == hook))
            }
            ExternalPluginAdapter::Oracle(oracle) => {
                assert!(collection
                    .external_plugin_adapter_list
                    .oracles
                    .contains(&oracle))
            }
            ExternalPluginAdapter::AppData(app_data) => {
                assert!(collection
                    .external_plugin_adapter_list
                    .app_data
                    .iter()
                    .any(|app_data_with_data| app_data_with_data.base == app_data))
            }
            ExternalPluginAdapter::LinkedLifecycleHook(hook) => {
                assert!(collection
                    .external_plugin_adapter_list
                    .linked_lifecycle_hooks
                    .contains(&hook))
            }
            ExternalPluginAdapter::LinkedAppData(app_data) => {
                assert!(collection
                    .external_plugin_adapter_list
                    .linked_app_data
                    .contains(&app_data))
            }
            ExternalPluginAdapter::DataSection(data) => {
                assert!(collection
                    .external_plugin_adapter_list
                    .data_sections
                    .iter()
                    .any(|data_sections_with_data| data_sections_with_data.base == data))
            }
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

#[macro_export]
macro_rules! assert_custom_instruction_error {
    ($ix:expr, $error:expr, $matcher:pat) => {
        match $error {
            solana_program_test::BanksClientError::TransactionError(
                solana_sdk::transaction::TransactionError::InstructionError(
                    $ix,
                    solana_sdk::instruction::InstructionError::Custom(x),
                ),
            ) => match num_traits::FromPrimitive::from_i32(x as i32) {
                Some($matcher) => assert!(true),
                Some(other) => {
                    assert!(
                        false,
                        "Expected another custom instruction error than '{:#?}'",
                        other
                    )
                }
                None => assert!(false, "Expected custom instruction error"),
            },
            err => assert!(
                false,
                "Expected custom instruction error but got '{:#?}'",
                err
            ),
        };
    };
}
