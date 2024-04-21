import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  addCollectionPluginV1,
  addPluginV1,
  createPlugin,
  updateCollectionPluginV1,
  updatePluginV1,
  pluginAuthorityPair,
} from '../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createCollection,
  createUmi,
} from './_setupRaw';

test('it cannot use an invalid system program for assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi);
  const fakeSystemProgram = generateSigner(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });

  const result = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program for assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi);
  const fakeLogWrapper = generateSigner(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });

  const result = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});

test('it cannot use an invalid system program for collections', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const collection = await createCollection(umi);
  const fakeSystemProgram = generateSigner(umi);

  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });

  const result = updateCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program for collections', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const collection = await createCollection(umi);
  const fakeLogWrapper = generateSigner(umi);

  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });

  const result = updateCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});

test('it cannot use an invalid collection to update a plugin on an asset', async (t) => {
  const umi = await createUmi();

  const assetAuth = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'initial' }] },
      }),
    ],
  });

  const collectionAuth = generateSigner(umi);
  const wrongCollection = await createCollection(umi, {
    updateAuthority: collectionAuth.publicKey,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'initial' }],
    },
  });

  const result = updatePluginV1(umi, {
    asset: asset.publicKey,
    collection: wrongCollection.publicKey,
    plugin: createPlugin({
      type: 'Attributes',
      data: {
        attributeList: [{ key: 'key', value: 'updated' }],
      },
    }),
    authority: collectionAuth,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'initial' }],
    },
  });
});
