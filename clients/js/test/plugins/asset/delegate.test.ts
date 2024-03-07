import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import {
  AssetWithPlugins,
  DataState,
  PluginType,
  approvePluginAuthority,
  addPlugin,
  create,
  fetchAssetWithPlugins,
  updateAuthority,
  updatePlugin,
} from '../../../src';
import { createUmi } from '../../_setup';

test('it can delegate a new authority', async (t) => {
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

  await addPlugin(umi, {
    asset: assetAddress.publicKey,
    plugin: {
      __kind: 'Freeze',
      fields: [{ frozen: false }],
    },
  }).sendAndConfirm(umi);

  await approvePluginAuthority(umi, {
    asset: assetAddress.publicKey,
    pluginType: PluginType.Freeze,
    newAuthority: {
      __kind: 'Pubkey',
      address: delegateAddress.publicKey,
    },
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

test('a delegate can freeze the token', async (t) => {
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

  await addPlugin(umi, {
    asset: assetAddress.publicKey,
    plugin: {
      __kind: 'Freeze',
      fields: [{ frozen: false }],
    },
  }).sendAndConfirm(umi);

  await approvePluginAuthority(umi, {
    asset: assetAddress.publicKey,
    pluginType: PluginType.Freeze,
    newAuthority: {
      __kind: 'Pubkey',
      address: delegateAddress.publicKey,
    },
  }).sendAndConfirm(umi);

  await updatePlugin(umi, {
    asset: assetAddress.publicKey,
    authority: delegateAddress,
    plugin: {
      __kind: 'Freeze',
      fields: [{ frozen: true }],
    },
  }).sendAndConfirm(umi);

  const asset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log(asset);
  t.like(asset, <AssetWithPlugins>{
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
          fields: [{ frozen: true }],
        },
      },
    ],
  });
});
