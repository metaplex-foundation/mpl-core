import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  Asset,
  DataState,
  PluginType,
  addPluginAuthority,
  addPlugin,
  create,
  fetchAsset,
  updateAuthority,
  plugin,
  fetchAssetWithPluginsTest,
  AssetWithPluginsTest,
  formPluginHeader,
  getPubkeyAuthority,
} from '../src';
import { createUmi } from './_setup';

test('TEST it can add an authority to a plugin TEST', async (t) => {
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
  t.like(asset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await addPlugin(umi, {
    asset: assetAddress.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
  })
    .append(
      addPluginAuthority(umi, {
        asset: assetAddress.publicKey,
        pluginType: PluginType.Freeze,
        newAuthority: getPubkeyAuthority(delegateAddress.publicKey),
      })
    )
    .sendAndConfirm(umi);

  const asset1 = await fetchAssetWithPluginsTest(umi, assetAddress.publicKey);
  t.like(asset1, <AssetWithPluginsTest>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    pluginHeader: formPluginHeader(BigInt(120)),
    freeze: {
      authorities: {
        owner: true,
        pubkey: [delegateAddress.publicKey],
      },
      frozen: false,
      offset: BigInt(118),
    },
  });
});
