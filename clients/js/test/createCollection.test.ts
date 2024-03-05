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
  updateAuthority,
} from '../src';
import { createUmi } from './_setup';

test('it can create a new collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);

  // When we create a new account.
  await createCollection(umi, {
    collection: collectionAddress,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
    plugins: [],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const collection = await fetchCollectionData(
    umi,
    collectionAddress.publicKey
  );
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
    collection: collectionAddress,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
    plugins: [{ __kind: 'Freeze', fields: [{ frozen: false }] }],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const collection = await fetchCollectionWithPlugins(
    umi,
    collectionAddress.publicKey
  );
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
    collection: collectionAddress,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
    plugins: [{ __kind: 'Freeze', fields: [{ frozen: false }] }],
  }).sendAndConfirm(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    collection: collectionAddress.publicKey,
    plugins: [],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Account State:", asset);
  t.like(asset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Collection', [
      collectionAddress.publicKey,
    ]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    pluginHeader: {
      key: 3,
      pluginRegistryOffset: BigInt(118),
    },
    pluginRegistry: {
      key: 4,
    },
  });

  t.assert(asset.pluginRegistry?.registry.length === 0);
  t.assert(asset.plugins?.length === 0);
});

// TODO: Add test
test('it cannot create a new asset with an update authority that is not the collection', async (t) => {
  t.pass();
});

test('it cannot create a new asset with a collection if it is not the collection auth', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);
  const assetAddress = generateSigner(umi);
  const collectionAuth = generateSigner(umi);

  // When we create a new account.
  await createCollection(umi, {
    collection: collectionAddress,
    updateAuthority: collectionAuth.publicKey,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
    plugins: [{ __kind: 'Freeze', fields: [{ frozen: false }] }],
  }).sendAndConfirm(umi);

  const collection = await fetchCollectionWithPlugins(
    umi,
    collectionAddress.publicKey
  );
  // console.log("Account State:", collection);
  t.like(collection, <CollectionWithPlugins>{
    publicKey: collectionAddress.publicKey,
    updateAuthority: collectionAuth.publicKey,
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

  // When we create a new account.
  const result = create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    collection: collectionAddress.publicKey,
    plugins: [],
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});
