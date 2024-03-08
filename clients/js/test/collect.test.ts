import { PublicKey, Umi, generateSigner, sol } from '@metaplex-foundation/umi';
import test from 'ava';

import {
  AssetWithPlugins,
  DataState,
  PluginType,
  addPlugin,
  create,
  fetchAssetWithPlugins,
  plugin,
  removePlugin,
  updateAuthority,
} from '../src';
import { createUmi } from './_setup';

const hasCollectAmount = async (umi: Umi, address: PublicKey) => {
  const account = await umi.rpc.getAccount(address);
  if (account.exists) {
    const rent = await umi.rpc.getRent(account.data.length)
    const diff = account.lamports.basisPoints - rent.basisPoints
    return diff === sol(0.0015).basisPoints
  }
  return false
}

test('it can create a new asset with collect amount', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: []
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Account State:", asset);
  t.like(asset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  t.assert(await hasCollectAmount(umi, assetAddress.publicKey), 'Collect amount not found')
});

test('it can add asset plugin with collect amount', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: []
  }).sendAndConfirm(umi);

  await addPlugin(umi, {
    asset: assetAddress.publicKey,
    plugin: plugin('Freeze', [{ frozen: true }]),
    initAuthority: null
  }).sendAndConfirm(umi);

  t.assert(await hasCollectAmount(umi, assetAddress.publicKey), 'Collect amount not found')
});

test('it can add remove asset plugin with collect amount', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [
      plugin('Freeze', [{ frozen: false }])
    ]
  }).sendAndConfirm(umi);
  t.assert(await hasCollectAmount(umi, assetAddress.publicKey), 'Collect amount not found')

  await removePlugin(umi, {
    asset: assetAddress.publicKey,
    pluginType: PluginType.Freeze,
  }).sendAndConfirm(umi);
  t.assert(await hasCollectAmount(umi, assetAddress.publicKey), 'Collect amount not found')
});