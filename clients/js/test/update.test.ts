import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import {
  AssetWithPlugins,
  DataState,
  Key,
  addPlugin,
  create,
  fetchAssetWithPlugins,
  update,
  updateAuthority,
  plugin,
  formPluginHeader,
} from '../src';
import { createUmi } from './_setup';

test('it can update an asset to be larger', async (t) => {
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

  await update(umi, {
    asset: assetAddress.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  t.like(asset, <AssetWithPlugins>{
    key: Key.Asset,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('it can update an asset to be smaller', async (t) => {
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

  await update(umi, {
    asset: assetAddress.publicKey,
    newName: '',
    newUri: '',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  t.like(asset, <AssetWithPlugins>{
    key: Key.Asset,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: '',
    uri: '',
  });
});

test('it can update an asset with plugins to be larger', async (t) => {
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

  await addPlugin(umi, {
    asset: assetAddress.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
    initAuthority: null,
  }).sendAndConfirm(umi);

  await update(umi, {
    asset: assetAddress.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log(JSON.stringify(asset, (_, v) => typeof v === 'bigint' ? v.toString() : v, 2));
  t.like(asset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    pluginHeader: formPluginHeader(BigInt(124)),
    freeze: {
      authority: {
        type: 'Owner',
      },
      offset: BigInt(122),
      frozen: false,
    },
  });
});

test('it can update an asset with plugins to be smaller', async (t) => {
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

  await addPlugin(umi, {
    asset: assetAddress.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
    initAuthority: null,
  }).sendAndConfirm(umi);

  await update(umi, {
    asset: assetAddress.publicKey,
    newName: '',
    newUri: '',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  t.like(asset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: '',
    uri: '',
    pluginHeader: formPluginHeader(BigInt(86)),
    freeze: {
      authority: {
        type: 'Owner',
      },
      offset: BigInt(84),
      frozen: false,
    },
  });
});
