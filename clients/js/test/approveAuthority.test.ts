import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import {
  Asset,
  AssetWithPlugins,
  DataState,
  PluginType,
  approvePluginAuthority,
  addPlugin,
  create,
  fetchAsset,
  fetchAssetWithPlugins,
  updateAuthority,
  plugin,
} from '../src';
import { createUmi } from './_setup';

test('it can add an authority to a plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const delegateAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", asset);
  t.like(asset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await addPlugin(umi, {
    asset: assetAddress.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
    initAuthority: null
  })
    .append(
      approvePluginAuthority(umi, {
        asset: assetAddress.publicKey,
        pluginType: PluginType.Freeze,
        newAuthority: {
          __kind: 'Pubkey',
          address: delegateAddress.publicKey,
        },
      })
    )
    .sendAndConfirm(umi);

  const asset1 = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log(JSON.stringify(asset1, (_, v) => typeof v === 'bigint' ? v.toString() : v, 2));
  t.like(asset1, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    pluginHeader: {
      key: 3,
      pluginRegistryOffset: BigInt(120),
    },
    pluginRegistry: {
      key: 4,
      registry: [
        {
          pluginType: 2,
          offset: BigInt(118),
          authority:
            { __kind: 'Pubkey', address: delegateAddress.publicKey },
        },
      ],
    },
    plugins: [
      {
        authority:
          { __kind: 'Pubkey', address: delegateAddress.publicKey },
        plugin: {
          __kind: 'Freeze',
          fields: [{ frozen: false }],
        },
      },
    ],
  });
});
