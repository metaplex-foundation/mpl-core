import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  AssetWithPlugins,
  DataState,
  addPlugin,
  create,
  fetchAssetWithPlugins,
  plugin,
  updatePlugin,
} from '../../../src';
import { createUmi } from '../../_setup';

test('it can freeze and unfreeze an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [],
  }).sendAndConfirm(umi);

  await addPlugin(umi, {
    assetAddress: assetAddress.publicKey,
    plugin: plugin('Freeze', [{ frozen: true }]),
  }).sendAndConfirm(umi);

  const asset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log(asset);
  t.like(asset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
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
          authorities: [
            { __kind: 'Owner' },
          ],
        },
      ],
    },
    plugins: [
      {
        authorities: [
          { __kind: 'Owner' },
        ],
        plugin: {
          __kind: 'Freeze',
          fields: [{ frozen: true }],
        },
      },
    ],
  });

  await updatePlugin(umi, {
    assetAddress: assetAddress.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
  }).sendAndConfirm(umi);


  const asset2 = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  t.like(asset2, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
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
          authorities: [
            { __kind: 'Owner' },
          ],
        },
      ],
    },
    plugins: [
      {
        authorities: [
          { __kind: 'Owner' },
        ],
        plugin: {
          __kind: 'Freeze',
          fields: [{ frozen: false }],
        },
      },
    ],
  });

});

