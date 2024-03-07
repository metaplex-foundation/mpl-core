import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  AssetWithPlugins,
  CollectionWithPlugins,
  DataState,
  PluginType,
  create,
  createCollection,
  fetchAssetWithPlugins,
  fetchCollectionWithPlugins,
  updateAuthority,
  addCollectionPlugin,
  approveCollectionPluginAuthority,
} from '../../../src';
import { createUmi } from '../../_setup';

test('it can create a new asset with a collection if it is the collection update delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);
  const assetAddress = generateSigner(umi);
  const updateDelegate = await generateSignerWithSol(umi);

  // When we create a new account.
  await createCollection(umi, {
    collection: collectionAddress,
    name: 'Test Bread Collection',
    uri: 'https://example.com/bread',
    plugins: [],
  }).sendAndConfirm(umi);

  await addCollectionPlugin(umi, {
    collection: collectionAddress.publicKey,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{}],
    },
  }).sendAndConfirm(umi);

  // console.log(JSON.stringify(await fetchCollectionWithPlugins(umi, collectionAddress.publicKey), (_, v) => typeof v === 'bigint' ? v.toString() : v, 2));

  await approveCollectionPluginAuthority(umi, {
    collection: collectionAddress.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: {
      __kind: 'Pubkey',
      address: updateDelegate.publicKey,
    },
  }).sendAndConfirm(umi);

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
      pluginRegistryOffset: BigInt(105),
    },
    pluginRegistry: {
      key: 4,
      registry: [
        {
          pluginType: PluginType.UpdateDelegate,
          offset: BigInt(104),
          authority:
            { __kind: 'Pubkey', address: updateDelegate.publicKey },
        },
      ],
    },
    plugins: [
      {
        authority:
          { __kind: 'Pubkey', address: updateDelegate.publicKey },
        plugin: {
          __kind: 'UpdateDelegate',
          fields: [{}],
        },
      },
    ],
  });

  umi.identity = updateDelegate;
  umi.payer = updateDelegate;
  const owner = generateSigner(umi);
  // When we create a new account.
  await create(umi, {
    owner: owner.publicKey,
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    collection: collectionAddress.publicKey,
    plugins: [],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Asset State:", asset);
  t.like(asset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Collection', [
      collectionAddress.publicKey,
    ]),
    owner: owner.publicKey,
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
