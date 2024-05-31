import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  PluginType,
  addCollectionPluginV1,
  approveCollectionPluginAuthorityV1,
  createPlugin,
  pluginAuthorityPair,
  addressPluginAuthority,
  updateV1,
  updateCollectionPluginV1,
  revokeCollectionPluginAuthorityV1,
} from '../../../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createCollection,
  createUmi,
} from '../../_setupRaw';

test('it can create a new asset with a collection if it is the collection updateDelegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  // When we create a new account.
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
      }),
    ],
  });

  await approveCollectionPluginAuthorityV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: addressPluginAuthority(updateDelegate.publicKey),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
  });

  umi.identity = updateDelegate;
  umi.payer = updateDelegate;
  const owner = generateSigner(umi);
  // When we create a new account.
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    owner,
    authority: updateDelegate,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('it can create a new asset with a collection if it is a collection updateDelegate additionalDelegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  // When we create a new account.
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.publicKey] },
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });

  umi.identity = updateDelegate;
  umi.payer = updateDelegate;
  const owner = generateSigner(umi);
  // When we create a new account.
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    owner,
    authority: updateDelegate,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('it can add updateDelegate to collection and then approve', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateDelegate = generateSigner(umi);

  const collection = await createCollection(umi);
  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
    }),
  }).sendAndConfirm(umi);

  await approveCollectionPluginAuthorityV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: addressPluginAuthority(updateDelegate.publicKey),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
  });
});

test('it can create a collection with updateDelegate with additional delegates', async (t) => {
  const umi = await createUmi();
  const updateDelegate = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.publicKey] },
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: { type: 'UpdateAuthority' },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });
});

test('it can add updateDelegate to collection with additional delegates', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);
  const updateDelegate = generateSigner(umi);

  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate.publicKey] },
    }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: { type: 'UpdateAuthority' },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });
});

test('it can update updateDelegate on collection with additional delegates', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);
  const updateDelegate = generateSigner(umi);

  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
    }),
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

  await updateCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate.publicKey] },
    }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });
});

test('an updateDelegate on collection can update an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  // When we create a new account.
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
      }),
    ],
  });

  await approveCollectionPluginAuthorityV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: addressPluginAuthority(updateDelegate.publicKey),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
  });

  umi.identity = updateDelegate;
  umi.payer = updateDelegate;
  const owner = generateSigner(umi);
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    owner,
    authority: updateDelegate,
    name: 'short',
    uri: 'https://short.com',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });

  await updateV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('an updateDelegate additionalDelegate on collection can update an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  // When we create a new account.
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.publicKey] },
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });

  umi.identity = updateDelegate;
  umi.payer = updateDelegate;
  const owner = generateSigner(umi);
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    owner,
    authority: updateDelegate,
    name: 'short',
    uri: 'https://short.com',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });

  await updateV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('an updateDelegate on collection cannot update an asset after delegate authority revoked', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  // When we create a new account.
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
      }),
    ],
  });

  await approveCollectionPluginAuthorityV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: addressPluginAuthority(updateDelegate.publicKey),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
  });

  umi.identity = updateDelegate;
  umi.payer = updateDelegate;
  const owner = generateSigner(umi);
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    owner,
    authority: updateDelegate,
    name: 'short',
    uri: 'https://short.com',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });

  await revokeCollectionPluginAuthorityV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
  }).sendAndConfirm(umi);

  const result = updateV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });
});

test('an updateDelegate additionalDelegate on collection cannot update an asset after delegate authority revoked', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const { identity } = umi;
  const updateDelegate = await generateSignerWithSol(umi);

  // When we create a new account.
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.publicKey] },
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });

  umi.identity = updateDelegate;
  umi.payer = updateDelegate;
  const owner = generateSigner(umi);
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    owner,
    authority: updateDelegate,
    name: 'short',
    uri: 'https://short.com',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });

  await updateCollectionPluginV1(umi, {
    collection: collection.publicKey,
    authority: identity,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [] },
    })
  }).sendAndConfirm(umi);

  const result = updateV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });
});
