#![cfg(feature = "test-sbf")]
pub mod setup;
use mpl_core::{
    accounts::{BaseAssetV1, PluginHeaderV1},
    fetch_plugin, fetch_plugins, list_plugins,
    types::{
        Creator, FreezeDelegate, Plugin, PluginAuthority, PluginAuthorityPair, PluginType,
        RegistryRecord, Royalties, RuleSet, UpdateAuthority,
    },
    DataBlob,
};
pub use setup::*;

use solana_program::account_info::AccountInfo;
use solana_program_test::tokio;
use solana_sdk::{signature::Keypair, signer::Signer};
use std::mem::size_of;

#[tokio::test]
async fn test_fetch_plugin() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();
    let creator = context.payer.pubkey();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset,
            data_state: None,
            name: None,
            uri: None,
            authority: None,
            update_authority: None,
            collection: None,
            plugins: vec![
                PluginAuthorityPair {
                    plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                    authority: None,
                },
                PluginAuthorityPair {
                    authority: None,
                    plugin: Plugin::Royalties(Royalties {
                        basis_points: 500,
                        creators: vec![Creator {
                            address: creator,
                            percentage: 100,
                        }],
                        rule_set: RuleSet::ProgramDenyList(vec![]),
                    }),
                },
            ],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    let owner = context.payer.pubkey();
    let update_authority = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![
                PluginAuthorityPair {
                    plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                    authority: Some(PluginAuthority::Owner),
                },
                PluginAuthorityPair {
                    authority: None,
                    plugin: Plugin::Royalties(Royalties {
                        basis_points: 500,
                        creators: vec![Creator {
                            address: creator,
                            percentage: 100,
                        }],
                        rule_set: RuleSet::ProgramDenyList(vec![]),
                    }),
                },
            ],
            external_plugin_adapters: vec![],
        },
    )
    .await;

    let mut asset_account = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .expect("get_account")
        .expect("asset account not found");

    let asset_pubkey = asset.pubkey();
    let mut lamports = 1_000_000_000;
    let account_info = AccountInfo::new(
        &asset_pubkey,
        false,
        false,
        &mut lamports,
        &mut asset_account.data,
        &asset_account.owner,
        false,
        1_000_000_000,
    );

    let plugin =
        fetch_plugin::<BaseAssetV1, FreezeDelegate>(&account_info, PluginType::FreezeDelegate)
            .unwrap();

    let expected_plugin_offset =
        BaseAssetV1::from_bytes(&asset_account.data).unwrap().len() + PluginHeaderV1::LEN;

    let expected = (
        PluginAuthority::Owner,
        FreezeDelegate { frozen: false },
        expected_plugin_offset,
    );
    assert_eq!(plugin, expected);
}

#[tokio::test]
async fn test_fetch_plugins() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();
    let creator = context.payer.pubkey();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset,
            data_state: None,
            name: None,
            uri: None,
            authority: None,
            update_authority: None,
            collection: None,
            plugins: vec![
                PluginAuthorityPair {
                    plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                    authority: None,
                },
                PluginAuthorityPair {
                    authority: None,
                    plugin: Plugin::Royalties(Royalties {
                        basis_points: 500,
                        creators: vec![Creator {
                            address: creator,
                            percentage: 100,
                        }],
                        rule_set: RuleSet::ProgramDenyList(vec![]),
                    }),
                },
            ],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    let owner = context.payer.pubkey();
    let update_authority = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![
                PluginAuthorityPair {
                    plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                    authority: Some(PluginAuthority::Owner),
                },
                PluginAuthorityPair {
                    authority: None,
                    plugin: Plugin::Royalties(Royalties {
                        basis_points: 500,
                        creators: vec![Creator {
                            address: creator,
                            percentage: 100,
                        }],
                        rule_set: RuleSet::ProgramDenyList(vec![]),
                    }),
                },
            ],
            external_plugin_adapters: vec![],
        },
    )
    .await;

    let asset_account = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .expect("get_account")
        .expect("asset account not found");

    let plugins = fetch_plugins(&asset_account.data).unwrap();

    let expected_first_plugin_offset =
        BaseAssetV1::from_bytes(&asset_account.data).unwrap().len() + PluginHeaderV1::LEN;

    let first_expected_registry_record = RegistryRecord {
        plugin_type: PluginType::FreezeDelegate,
        authority: PluginAuthority::Owner,
        offset: expected_first_plugin_offset as u64,
    };

    let expected_second_plugin_offset =
        expected_first_plugin_offset + size_of::<u8>() + size_of::<FreezeDelegate>();

    let second_expected_registry_record = RegistryRecord {
        plugin_type: PluginType::Royalties,
        authority: PluginAuthority::UpdateAuthority,
        offset: expected_second_plugin_offset as u64,
    };

    assert_eq!(
        plugins,
        vec![
            first_expected_registry_record,
            second_expected_registry_record
        ]
    )
}

#[tokio::test]
async fn test_list_plugins() {
    let mut context = program_test().start_with_context().await;

    let asset = Keypair::new();
    let creator = context.payer.pubkey();
    create_asset(
        &mut context,
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset,
            data_state: None,
            name: None,
            uri: None,
            authority: None,
            update_authority: None,
            collection: None,
            plugins: vec![
                PluginAuthorityPair {
                    plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                    authority: None,
                },
                PluginAuthorityPair {
                    authority: None,
                    plugin: Plugin::Royalties(Royalties {
                        basis_points: 500,
                        creators: vec![Creator {
                            address: creator,
                            percentage: 100,
                        }],
                        rule_set: RuleSet::ProgramDenyList(vec![]),
                    }),
                },
            ],
            external_plugin_adapters: vec![],
        },
    )
    .await
    .unwrap();

    let owner = context.payer.pubkey();
    let update_authority = context.payer.pubkey();
    assert_asset(
        &mut context,
        AssertAssetHelperArgs {
            asset: asset.pubkey(),
            owner,
            update_authority: Some(UpdateAuthority::Address(update_authority)),
            name: None,
            uri: None,
            plugins: vec![
                PluginAuthorityPair {
                    plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                    authority: Some(PluginAuthority::Owner),
                },
                PluginAuthorityPair {
                    authority: None,
                    plugin: Plugin::Royalties(Royalties {
                        basis_points: 500,
                        creators: vec![Creator {
                            address: creator,
                            percentage: 100,
                        }],
                        rule_set: RuleSet::ProgramDenyList(vec![]),
                    }),
                },
            ],
            external_plugin_adapters: vec![],
        },
    )
    .await;

    let asset_account = context
        .banks_client
        .get_account(asset.pubkey())
        .await
        .expect("get_account")
        .expect("asset account not found");

    let plugins = list_plugins(&asset_account.data).unwrap();
    assert_eq!(
        plugins,
        vec![PluginType::FreezeDelegate, PluginType::Royalties]
    )
}
