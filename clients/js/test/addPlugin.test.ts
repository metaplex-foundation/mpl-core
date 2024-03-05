import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import {
  Asset,
  AssetWithPlugins,
  CollectionData,
  CollectionWithPlugins,
  DataState,
  PluginType,
  addCollectionPlugin,
  addPlugin,
  create,
  createCollection,
  fetchAsset,
  fetchAssetWithPlugins,
  fetchCollectionData,
  fetchCollectionWithPlugins,
  updateAuthority,
} from '../src';
import { createUmi } from './_setup';

test('it can add a plugin to an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

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
    plugin: {
      __kind: 'Freeze',
      fields: [{ frozen: false }],
    }
  }).sendAndConfirm(umi);

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
          authorities: [{ __kind: 'Owner' }],
        },
      ],
    },
    plugins: [
      {
        authorities: [{ __kind: 'Owner' }],
        plugin: {
          __kind: 'Freeze',
          fields: [{ frozen: false }],
        },
      },
    ],
  });
});

test('it can add a plugin to a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);

  // When we create a new account.
  await createCollection(umi, {
    collection: collectionAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const collection = await fetchCollectionData(umi, collectionAddress.publicKey);
  // console.log("Account State:", collection);
  t.like(collection, <CollectionData>{
    publicKey: collectionAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await addCollectionPlugin(umi, {
    collection: collectionAddress.publicKey,
    plugin: {
      __kind: 'Freeze',
      fields: [{ frozen: false }],
    }
  }).sendAndConfirm(umi);

  const asset1 = await fetchCollectionWithPlugins(umi, collectionAddress.publicKey);
  // console.log(JSON.stringify(asset1, (_, v) => typeof v === 'bigint' ? v.toString() : v, 2));
  t.like(asset1, <CollectionWithPlugins>{
    publicKey: collectionAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    pluginHeader: {
      key: 3,
      pluginRegistryOffset: BigInt(95),
    },
    pluginRegistry: {
      key: 4,
      registry: [
        {
          pluginType: PluginType.Freeze,
          offset: BigInt(93),
          authorities: [{ __kind: 'Owner' }],
        },
      ],
    },
    plugins: [
      {
        authorities: [{ __kind: 'Owner' }],
        plugin: {
          __kind: 'Freeze',
          fields: [{ frozen: false }],
        },
      },
    ],
  });
});
