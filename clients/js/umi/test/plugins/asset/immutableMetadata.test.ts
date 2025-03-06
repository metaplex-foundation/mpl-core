import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';

import { addPlugin, update } from '../../../src';
import { DEFAULT_ASSET, assertAsset, createUmi } from '../../_setupRaw';
import { createAsset } from '../../_setupSdk';

test('it can prevent the asset from metadata updating', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'ImmutableMetadata',
      },
    ],
  });

  const result = update(umi, {
    asset,
    name: 'bread',
    uri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can mutate its metadata unless ImmutableMetadata plugin is added', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);

  await update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
  }).sendAndConfirm(umi);

  const result = update(umi, {
    asset,
    name: 'Test Bread 3',
    uri: 'https://example.com/bread3',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it states that UA is the only one who can add the ImmutableMetadata', async (t) => {
  const umi = await createUmi();
  const updateAuthority = generateSigner(umi);
  const randomUser = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: updateAuthority.publicKey,
  });

  // random keypair can't add ImmutableMetadata
  let result = addPlugin(umi, {
    authority: randomUser,
    asset: asset.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'NoApprovals',
  });

  // Owner can't add ImmutableMetadata
  result = addPlugin(umi, {
    authority: umi.identity,
    asset: asset.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'NoApprovals',
  });

  // UA CAN add ImmutableMetadata
  await addPlugin(umi, {
    authority: updateAuthority,
    asset: asset.publicKey,
    plugin: { type: 'ImmutableMetadata' },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    immutableMetadata: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});
