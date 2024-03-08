import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  AssetWithPlugins,
  CollectionWithPlugins,
  DataState,
  PluginType,
  addCollectionPlugin,
  approveCollectionPluginAuthority,
  authority,
  create,
  fetchAssetWithPlugins,
  createCollection as baseCreateCollection,
  fetchCollectionWithPlugins,
  plugin,
  updateAuthority,
} from '../src';
import { DEFAULT_ASSET, DEFAULT_COLLECTION, assertAsset, assertCollection, createAssetWithCollection, createCollection, createUmi } from './_setup';

test('it can create a new collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);

  await createCollection(umi, {
    collection: collectionAddress,
  })

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collectionAddress,
    updateAuthority: umi.identity.publicKey,
  })
});

test('it can create a new collection with plugins', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);

  // When we create a new account.
  await baseCreateCollection(umi, {
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
          authority: { __kind: 'Owner' },
        },
      ],
    },
    plugins: [
      {
        authority: { __kind: 'Owner' },
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
  const { asset, collection} = await createAssetWithCollection(umi, {
  }, {
    plugins: [plugin('UpdateDelegate', [{}])],
  })

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    plugins: [{
      authority: authority('UpdateAuthority'),
      plugin: plugin('UpdateDelegate', [{}])
    }]
  })

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  })

  t.assert(asset.pluginRegistry?.registry.length === 0);
  t.assert(asset.plugins?.length === 0);
});

test('it can create a new asset with a collection with collection delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);
  const assetAddress = generateSigner(umi);
  const delegate = generateSigner(umi);

  // When we create a new account.
  await baseCreateCollection(umi, {
    collection: collectionAddress,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
    plugins: [{ __kind: 'Freeze', fields: [{ frozen: false }] }],
  }).sendAndConfirm(umi);

  await addCollectionPlugin(umi, {
    collection: collectionAddress.publicKey,
    plugin: plugin('UpdateDelegate', [{}]),
    initAuthority: null
  }).sendAndConfirm(umi);

  await approveCollectionPluginAuthority(umi, {
    collection: collectionAddress.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: authority('Pubkey', { address: delegate.publicKey }),
  }).sendAndConfirm(umi);


  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    collection: collectionAddress.publicKey,
    authority: delegate,
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
  await baseCreateCollection(umi, {
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
          authority: { __kind: 'Owner' },
        },
      ],
    },
    plugins: [
      {
        authority: { __kind: 'Owner' },
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
