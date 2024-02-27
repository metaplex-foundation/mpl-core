import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  AssetWithPlugins,
  CollectionData,
  CollectionWithPlugins,
  DataState,
  create,
  createCollection,
  fetchAssetWithPlugins,
  fetchCollectionData,
  fetchCollectionWithPlugins,
} from '../src';
import { createUmi } from './_setup';

test('it can create a new collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);

  // When we create a new account.
  await createCollection(umi, {
    collectionAddress,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
    plugins: []
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const collection = await fetchCollectionData(umi, collectionAddress.publicKey);
  // console.log("Account State:", collection);
  t.like(collection, <CollectionData>{
    publicKey: collectionAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
  });
});

test('it can create a new collection with plugins', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);

  // When we create a new account.
  await createCollection(umi, {
    collectionAddress,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
    plugins: [{ __kind: 'Freeze', fields: [{ frozen: false }] }]
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const collection = await fetchCollectionWithPlugins(umi, collectionAddress.publicKey);
  // console.log("Account State:", collection);
  t.like(collection, <CollectionWithPlugins>{
    publicKey: collectionAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
    pluginHeader: {
      key: 3,
      pluginRegistryOffset: BigInt(106),
    },
    pluginRegistry: {
      key: 4,
      registry: [
        {
          pluginType: 2,
          offset: BigInt(104),
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

test('it can create a new asset with a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await createCollection(umi, {
    collectionAddress,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
    plugins: [{ __kind: 'Freeze', fields: [{ frozen: false }] }]
  }).sendAndConfirm(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [{
      __kind: 'Collection', fields: [{
        collectionAddress: collectionAddress.publicKey,
        managed: true
      }]
    }],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Account State:", asset);
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
          pluginType: 5,
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