import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';

import {
  addCollectionPluginV1,
  addPluginV1,
  createPlugin,
  pluginAuthorityPair,
  addressPluginAuthority,
  ruleSet,
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

test('it can add a plugin to an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
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
      frozen: false,
    },
  });
});

test('it can add an authority managed plugin to an asset via update auth', async (t) => {
  const umi = await createUmi();
  const updateAuth = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: updateAuth,
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
    }),
    authority: updateAuth,
    payer: umi.identity,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: updateAuth.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });
});

test('it can add a plugin to an asset with a different authority than the default', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
    initAuthority: addressPluginAuthority(delegateAddress.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });
});

test('it can add plugin to asset with a plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'TransferDelegate' }),
    initAuthority: addressPluginAuthority(delegate.publicKey),
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
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
    },
  });
});

test('it can add a plugin to a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const collection = await createCollection(umi, {});

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [{ address: umi.identity.publicKey, percentage: 100 }],
        ruleSet: ruleSet('None'),
      },
    }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('None'),
    },
  });
});

test('it cannot add an owner-managed plugin to a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const collection = await createCollection(umi, {});

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can add an authority-managed plugin to an asset via delegate authority', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          authority: addressPluginAuthority(delegate.publicKey),
        }),
      ],
    }
  );

  await addPluginV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          {
            address: umi.identity.publicKey,
            percentage: 100,
          },
        ],
        ruleSet: ruleSet('None'),
      },
    }),
    authority: delegate,
    payer: umi.identity,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: ruleSet('None'),
    },
  });
});

test('it can add an authority-managed plugin to an asset with the collection update authority', async (t) => {
  const umi = await createUmi();
  const collectionAuth = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    { authority: collectionAuth },
    { updateAuthority: collectionAuth }
  );

  await addPluginV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          {
            address: umi.identity.publicKey,
            percentage: 100,
          },
        ],
        ruleSet: ruleSet('None'),
      },
    }),
    authority: collectionAuth,
    payer: umi.identity,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: ruleSet('None'),
    },
  });
});

test('it cannot add a owner-managed plugin to an asset via delegate authority', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);
  const collectionAuth = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      authority: collectionAuth,
    },
    {
      updateAuthority: collectionAuth,
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          authority: addressPluginAuthority(delegate.publicKey),
        }),
      ],
    }
  );

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'FreezeDelegate',
      data: {
        frozen: false,
      },
    }),
    authority: delegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'NoApprovals',
  });
});

test('it cannot add authority-managed plugin to an asset by owner', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(umi, { owner });

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    authority: owner,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          {
            address: umi.identity.publicKey,
            percentage: 100,
          },
        ],
        ruleSet: ruleSet('None'),
      },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'NoApprovals',
  });
});

test('it can add a plugin to a collection with a plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
          creators: [
            {
              address: umi.identity.publicKey,
              percentage: 100,
            },
          ],
          ruleSet: ruleSet('None'),
        },
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: ruleSet('None'),
    },
  });

  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
    }),
    initAuthority: addressPluginAuthority(delegate.publicKey),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: ruleSet('None'),
    },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      additionalDelegates: [],
    },
  });
});

test('it can add a plugin to an asset that is part of a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const { asset, collection } = await createAssetWithCollection(umi, {});

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it cannot add a plugin to an asset if the collection is wrong', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const { asset, collection } = await createAssetWithCollection(umi, {});

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'MissingCollection',
  });
});

test('it cannot add a plugin to an asset if the collection is missing', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const wrongCollection = await createCollection(umi);
  const { asset, collection } = await createAssetWithCollection(umi, {});

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    collection: wrongCollection.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidCollection',
  });
});

test('it cannot use an invalid system program for assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi);
  const fakeSystemProgram = generateSigner(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidSystemProgram',
  });
});

test('it cannot use an invalid noop program for assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi);
  const fakeLogWrapper = generateSigner(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidLogWrapperProgram',
  });
});

test('it cannot use an invalid system program for collections', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const collection = await createCollection(umi);
  const fakeSystemProgram = generateSigner(umi);

  // Then an account was created with the correct data.
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidSystemProgram',
  });
});

test('it cannot use an invalid noop program for collections', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const collection = await createCollection(umi);
  const fakeLogWrapper = generateSigner(umi);

  // Then an account was created with the correct data.
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidLogWrapperProgram',
  });
});
