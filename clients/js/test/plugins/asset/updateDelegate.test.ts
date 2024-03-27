import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  createPlugin,
  pluginAuthorityPair,
  addPluginV1,
  updatePluginV1,
} from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createUmi,
} from '../../_setup';

test('it can create an asset with updateDelegate', async (t) => {
  // Given a Umi instance.
  const umi = await createUmi();

  // When we attempt to create a new collection with updateDelegate with additional delegates.
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });
});

test('it cannot create an asset with updateDelegate with additional delegates', async (t) => {
  // Given a Umi instance.
  const umi = await createUmi();

  // When we attempt to create a new asset with updateDelegate with additional delegates.
  let result = createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [generateSigner(umi).publicKey] },
      }),
    ],
  });

  // The program does not allow it at creation.
  await t.throwsAsync(result, { name: 'NotAvailable' });
});

test('it cannot add updateDelegate to asset with additional delegates', async (t) => {
  // Given a Umi instance and a new asset.
  const umi = await createUmi();
  const asset = await createAsset(umi);

  // When we attempt to add an updateDelegate with additional delegates.
  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [generateSigner(umi).publicKey] },
    }),
  }).sendAndConfirm(umi);

  // The program does not allow it.
  await t.throwsAsync(result, { name: 'NotAvailable' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it cannot update updateDelegate on asset with additional delegates', async (t) => {
  // Given a Umi instance and a new collection.
  const umi = await createUmi();
  const asset = await createAsset(umi);

  // And the collection has an existing update delegate.
  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [] },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });

  // And we attempt to add an updateDelegate with additional delegates.
  const result = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [generateSigner(umi).publicKey] },
    }),
  }).sendAndConfirm(umi);

  // The program does not allow it.
  await t.throwsAsync(result, { name: 'NotAvailable' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });
});
