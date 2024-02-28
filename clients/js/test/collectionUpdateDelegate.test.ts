import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  AssetWithPlugins,
  CollectionWithPlugins,
  DataState,
  PluginType,
  addAuthority,
  addPlugin,
  create,
  createCollection,
  fetchAssetWithPlugins,
  fetchCollectionWithPlugins,
} from '../src';
import { createUmi } from './_setup';

test('it can create a new asset with a collection if it is the collection update delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);
  const assetAddress = generateSigner(umi);
  const updateDelegate = generateSigner(umi);

  // When we create a new account.
  await createCollection(umi, {
    collectionAddress,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
    plugins: []
  }).sendAndConfirm(umi);

  await addPlugin(umi, {
    assetAddress: collectionAddress.publicKey,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{}],
    }
  }).sendAndConfirm(umi);

  // console.log(JSON.stringify(await fetchCollectionWithPlugins(umi, collectionAddress.publicKey), (_, v) => typeof v === 'bigint' ? v.toString() : v, 2));

  await addAuthority(umi, {
    assetAddress: collectionAddress.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: {
      __kind: 'Pubkey',
      address: updateDelegate.publicKey,
    }
  }).sendAndConfirm(umi);

  const collection = await fetchCollectionWithPlugins(umi, collectionAddress.publicKey);
  // console.log("Account State:", collection);
  t.like(collection, <CollectionWithPlugins>{
    publicKey: collectionAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
    pluginHeader: {
      key: 3,
      pluginRegistryOffset: BigInt(105),
    },
    pluginRegistry: {
      key: 4,
      registry: [
        {
          pluginType: PluginType.UpdateDelegate,
          offset: BigInt(104),
          authorities: [
            { __kind: 'UpdateAuthority' },
            { __kind: 'Pubkey', address: updateDelegate.publicKey }
          ],
        },
      ],
    },
    plugins: [
      {
        authorities: [
          { __kind: 'UpdateAuthority' },
          { __kind: 'Pubkey', address: updateDelegate.publicKey }
        ],
        plugin: {
          __kind: 'UpdateDelegate',
          fields: [{}],
        },
      },
    ],
  });

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    collection: collectionAddress.publicKey,
    plugins: [{
      __kind: 'Collection', fields: [{
        collectionAddress: collectionAddress.publicKey,
        managed: true
      }]
    }],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  console.log("Asset State:", asset);
  t.like(asset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    pluginHeader: {
      key: 3,
      pluginRegistryOffset: BigInt(151),
    },
    pluginRegistry: {
      key: 4,
      registry: [
        {
          pluginType: PluginType.Collection,
          offset: BigInt(117),
          authorities: [{ __kind: 'UpdateAuthority' }],
        },
      ],
    },
    plugins: [
      {
        authorities: [{ __kind: 'UpdateAuthority' }],
        plugin: {
          __kind: 'Collection',
          fields: [{ collectionAddress: collectionAddress.publicKey, managed: true }],
        },
      },
    ],
  });
});