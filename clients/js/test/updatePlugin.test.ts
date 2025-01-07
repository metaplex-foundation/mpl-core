import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  addCollectionPluginV1,
  addPluginV1,
  addressPluginAuthority,
  createPlugin,
  ownerPluginAuthority,
  pluginAuthorityPair,
  updateCollectionPluginV1,
  updatePluginAuthority,
  updatePluginV1,
} from '../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createAssetWithCollection,
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

test('it cannot update a plugin with Owner authority on an Asset if a plugin with UpdateAuthority authority is present on a Collection', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi, {
    name: 'test',
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  const randomAuthority = generateSigner(umi);

  const asset = await createAssetWithCollection(umi, {
    name: 'test',
    collection: collection.publicKey,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: addressPluginAuthority(randomAuthority.publicKey),
      }),
    ],
  });

  const res = updatePluginV1(umi, {
    asset: asset.asset.publicKey,
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'FreezeDelegate',
      data: {
        frozen: false,
      },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'NoApprovals' });
});

test('it can update a plugin with Owner authority on an Asset if a plugin with UpdateAuthority authority is present on a Collection', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi, {
    ...DEFAULT_COLLECTION,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  const asset = await createAssetWithCollection(umi, {
    ...DEFAULT_ASSET,
    collection: collection.publicKey,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await updatePluginV1(umi, {
    asset: asset.asset.publicKey,
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'FreezeDelegate',
      data: {
        frozen: false,
      },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    freezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('it cannot update a plugin with Owner authority on an Asset if a plugin with UpdateAuthority authority is present', async (t) => {
  const umi = await createUmi();

  const randomAuthority = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: addressPluginAuthority(randomAuthority.publicKey),
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
        authority: updatePluginAuthority(),
      }),
    ],
  });

  const res = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'FreezeDelegate',
      data: {
        frozen: false,
      },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'NoApprovals' });
});

test('it can update a plugin with Owner authority on an Asset if a plugin with UpdateAuthority authority is present', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    ...DEFAULT_ASSET,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: updatePluginAuthority(),
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'FreezeDelegate',
      data: {
        frozen: false,
      },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });
});

test('it cannot update a plugin with UpdateAuthority authority on an Asset if a plugin with Owner authority is present', async (t) => {
  const umi = await createUmi();

  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner: owner.publicKey,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: ownerPluginAuthority(),
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
        authority: updatePluginAuthority(),
      }),
    ],
  });

  const res = updatePluginV1(umi, {
    asset: asset.publicKey,
    authority: owner,
    plugin: createPlugin({
      type: 'Attributes',
      data: { attributeList: [{ key: 'key2', value: 'value2' }] },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'NoApprovals' });
});

test('it can update a plugin with UpdateAuthority authority on an Asset if a plugin with Owner authority is present', async (t) => {
  const umi = await createUmi();

  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    ...DEFAULT_ASSET,
    owner: owner.publicKey,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: ownerPluginAuthority(),
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
        authority: ownerPluginAuthority(),
      }),
    ],
  });

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    authority: owner,
    plugin: createPlugin({
      type: 'Attributes',
      data: { attributeList: [{ key: 'key2', value: 'value2' }] },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
    attributes: {
      authority: {
        type: 'Owner',
      },
      attributeList: [{ key: 'key2', value: 'value2' }],
    },
  });
});
