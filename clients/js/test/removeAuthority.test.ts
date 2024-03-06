import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  Asset,
  AssetWithPlugins,
  DataState,
  PluginType,
  addPluginAuthority,
  addPlugin,
  create,
  fetchAsset,
  fetchAssetWithPlugins,
  plugin,
  removePluginAuthority,
  updateAuthority,
  createCollection,
} from '../src';
import { createUmi } from './_setup';

test('it can remove an authority from a plugin', async (t) => {
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
    plugin: {
      __kind: 'Freeze',
      fields: [{ frozen: false }],
    },
  })
    .append(
      addPluginAuthority(umi, {
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
          authorities: [
            { __kind: 'Owner' },
            { __kind: 'Pubkey', address: delegateAddress.publicKey },
          ],
        },
      ],
    },
    plugins: [
      {
        authorities: [
          { __kind: 'Owner' },
          { __kind: 'Pubkey', address: delegateAddress.publicKey },
        ],
        plugin: {
          __kind: 'Freeze',
          fields: [{ frozen: false }],
        },
      },
    ],
  });

  await removePluginAuthority(umi, {
    asset: assetAddress.publicKey,
    pluginType: PluginType.Freeze,
    authorityToRemove: {
      __kind: 'Pubkey',
      address: delegateAddress.publicKey,
    },
  }).sendAndConfirm(umi);

  const asset2 = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log(JSON.stringify(asset1, (_, v) => typeof v === 'bigint' ? v.toString() : v, 2));
  t.like(asset2, <AssetWithPlugins>{
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

test('it can remove the default authority from a plugin to make it immutable', async (t) => {
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
    },
  }).sendAndConfirm(umi);

  await removePluginAuthority(umi, {
    asset: assetAddress.publicKey,
    pluginType: PluginType.Freeze,
    authorityToRemove: {
      __kind: 'Owner',
    },
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
          authorities: [{ __kind: 'None' }],
        },
      ],
    },
    plugins: [
      {
        authorities: [{ __kind: 'None' }],
        plugin: {
          __kind: 'Freeze',
          fields: [{ frozen: false }],
        },
      },
    ],
  });
});

test('it can remove a pubkey authority from a plugin if that pubkey is the signer authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const pubkeyAuth = await generateSignerWithSol(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [],
  }).sendAndConfirm(umi);

  await addPlugin(umi, {
    asset: assetAddress.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
  }).sendAndConfirm(umi);

  await addPluginAuthority(umi, {
    asset: assetAddress.publicKey,
    pluginType: PluginType.Freeze,
    newAuthority: {
      __kind: 'Pubkey',
      address: pubkeyAuth.publicKey,
    },
  }).sendAndConfirm(umi);

  const asset1 = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log(JSON.stringify(asset1, (_, v) => typeof v === 'bigint' ? v.toString() : v, 2));
  t.like(asset1, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [
      {
        authorities: [{ __kind: 'Owner' }, { __kind: 'Pubkey', address: pubkeyAuth.publicKey }],
        plugin: {
          __kind: 'Freeze',
          fields: [{ frozen: false }],
        },
      },
    ],
  });

  const umi2 = await createUmi();

  await removePluginAuthority(umi2, {
    payer: umi2.identity,
    asset: assetAddress.publicKey,
    authority: pubkeyAuth,
    pluginType: PluginType.Freeze,
    authorityToRemove: {
      __kind: 'Pubkey',
      address: pubkeyAuth.publicKey,
    },
  }).sendAndConfirm(umi);

  const asset2 = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log(JSON.stringify(asset1, (_, v) => typeof v === 'bigint' ? v.toString() : v, 2));
  t.like(asset2, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
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

test('it can remove a pubkey authority from a asset in collection if that pubkey is the signer authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const pubkeyAuth = await generateSignerWithSol(umi);
  const collectionAddress = generateSigner(umi);
  
  await createCollection(umi, {
    collection: collectionAddress,
    name: 'Test Collection',
    uri: 'https://example.com/collection',
    plugins: []
  }).sendAndConfirm(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    collection: collectionAddress.publicKey,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [],
  }).sendAndConfirm(umi);

  await addPlugin(umi, {
    asset: assetAddress.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
  }).sendAndConfirm(umi);

  await addPluginAuthority(umi, {
    asset: assetAddress.publicKey,
    pluginType: PluginType.Freeze,
    newAuthority: {
      __kind: 'Pubkey',
      address: pubkeyAuth.publicKey,
    },
  }).sendAndConfirm(umi);

  const asset1 = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log(JSON.stringify(asset1, (_, v) => typeof v === 'bigint' ? v.toString() : v, 2));
  t.like(asset1, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Collection', [collectionAddress.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [
      {
        authorities: [{ __kind: 'Owner' }, { __kind: 'Pubkey', address: pubkeyAuth.publicKey }],
        plugin: {
          __kind: 'Freeze',
          fields: [{ frozen: false }],
        },
      },
    ],
  });

  const umi2 = await createUmi();

  await removePluginAuthority(umi2, {
    payer: umi2.identity,
    asset: assetAddress.publicKey,
    authority: pubkeyAuth,
    pluginType: PluginType.Freeze,
    authorityToRemove: {
      __kind: 'Pubkey',
      address: pubkeyAuth.publicKey,
    },
  }).sendAndConfirm(umi);

  const asset2 = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log(JSON.stringify(asset1, (_, v) => typeof v === 'bigint' ? v.toString() : v, 2));
  t.like(asset2, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Collection', [collectionAddress.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
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
