import test from 'ava';

import { updateV1, pluginAuthorityPair } from '../src';
import { assertAsset, createAsset, createUmi } from './_setup';

test('it can update an asset to be larger', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);

  await updateV1(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('it can update an asset to be smaller', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);

  await updateV1(umi, {
    asset: asset.publicKey,
    newName: '',
    newUri: '',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    name: '',
    uri: '',
  });
});

test('it can update an asset with plugins to be larger', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    name: 'short',
    uri: 'https://short.com',
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  // const asset = await createAsset(umi, {
  //   name: 'Test Bread',
  //   uri: 'https://example.com/bread',
  //   plugins: [{
  //     plugin: createPlugin({ type: 'Freeze', data: { frozen: true }}),
  //     authority: null,
  //   }],
  // });

  await updateV1(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it can update an asset with plugins to be smaller', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await updateV1(umi, {
    asset: asset.publicKey,
    newName: '',
    newUri: '',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    name: '',
    uri: '',
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});
