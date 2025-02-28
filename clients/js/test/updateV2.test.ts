import test from 'ava';

import { generateSigner } from '@metaplex-foundation/umi';
import {
  update,
  updateAuthority,
  updateCollection,
  addCollectionPlugin,
  approveCollectionPluginAuthority,
  addPlugin,
  approvePluginAuthority,
} from '../src';
import {
  assertAsset,
  assertCollection,
  createUmi,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
} from './_setupRaw';

import {
  createAsset,
  createAssetWithCollection,
  createCollection,
} from './_setupSdk';

test('it can update an asset to be larger', async (t) => {
  // Given a Umi instance and a new signer.
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
});

test('it cannot update an asset using asset as authority', async (t) => {
  const umi = await createUmi();
  const myAsset = generateSigner(umi);

  const asset = await createAsset(umi, {
    asset: myAsset,
    name: 'short',
    uri: 'https://short.com',
  });

  const result = update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    authority: myAsset,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });
});

test('it can update an asset to be smaller', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);

  await update(umi, {
    asset,
    name: '',
    uri: '',
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
      {
        type: 'FreezeDelegate',
        frozen: false,
      },
    ],
  });

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
      {
        type: 'FreezeDelegate',
        frozen: false,
      },
    ],
  });

  await update(umi, {
    asset,
    name: '',
    uri: '',
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

test('it can update an asset update authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const newUpdateAuthority = generateSigner(umi);

  await update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Address', [
      newUpdateAuthority.publicKey,
    ]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: newUpdateAuthority.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('it cannot update an asset using wrong authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateAuth = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: updateAuth,
  });

  const newUpdateAuthority = generateSigner(umi);
  const result = update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Address', [
      newUpdateAuthority.publicKey,
    ]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: updateAuth.publicKey },
  });
});

test('it cannot use an invalid system program for assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const fakeSystemProgram = generateSigner(umi);

  const result = update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program for assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const fakeLogWrapper = generateSigner(umi);

  const result = update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});

test('it cannot use an invalid system program for collections', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collection = await createCollection(umi);
  const fakeSystemProgram = generateSigner(umi);

  const result = updateCollection(umi, {
    collection: collection.publicKey,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program for collections', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collection = await createCollection(umi);
  const fakeLogWrapper = generateSigner(umi);

  const result = updateCollection(umi, {
    collection: collection.publicKey,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});

test('it can remove an asset from a collection using update', async (t) => {
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(umi, {}, {});

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 1,
    numMinted: 1,
  });

  await update(umi, {
    asset,
    collection,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 1,
  });
});

test('it cannot remove an asset from a collection if not collection update auth', async (t) => {
  const umi = await createUmi();
  const collectionAuthority = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    { authority: collectionAuthority },
    { updateAuthority: collectionAuthority }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 1,
  });

  const result = update(umi, {
    asset,
    collection,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 1,
  });
});

test('it cannot remove an asset from a collection when missing collection account', async (t) => {
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(umi, {}, {});

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 1,
    numMinted: 1,
  });

  const result = update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'MissingCollection' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 1,
    numMinted: 1,
  });
});

test('it cannot remove an asset from a collection when using incorrect collection account', async (t) => {
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(umi, {}, {});

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  const randomCollection = await createCollection(umi);

  const result = update(umi, {
    asset,
    collection: randomCollection,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });
});

test('it can add asset to collection using update', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const collection = await createCollection(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
  });

  await update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
    newCollection: collection.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 1,
    numMinted: 0,
  });
});

test('it cannot add asset to collection when missing collection account', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const collection = await createCollection(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
  });

  const result = update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'MissingCollection' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
  });
});

test('it cannot add asset to collection when using incorrect collection account', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const collection = await createCollection(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const result = update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
    newCollection: generateSigner(umi).publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });
});

test('it cannot add asset to collection using only new collection authority', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const newCollectionAuthority = generateSigner(umi);
  const collection = await createCollection(umi, {
    updateAuthority: newCollectionAuthority,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const result = update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
    newCollection: collection.publicKey,
    authority: newCollectionAuthority,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });
});

test('it cannot add asset to collection if not both asset and collection auth', async (t) => {
  const umi = await createUmi();

  const assetAuthority = generateSigner(umi);
  const asset = await createAsset(umi, { updateAuthority: assetAuthority });

  const collectionAuthority = generateSigner(umi);
  const collection = await createCollection(umi, {
    updateAuthority: collectionAuthority,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuthority.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 0,
    numMinted: 0,
  });

  // Attempt to update using asset authority.
  const result = update(umi, {
    asset,
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
    newCollection: collection.publicKey,
    authority: assetAuthority,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuthority.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 0,
    numMinted: 0,
  });

  // Attempt to update using collection authority.
  const result2 = update(umi, {
    asset,
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
    newCollection: collection.publicKey,
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result2, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuthority.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 0,
    numMinted: 0,
  });
});

test('it can change an asset collection using same update authority', async (t) => {
  const umi = await createUmi();
  const collectionAuthority = generateSigner(umi);
  const { asset, collection: originalCollection } =
    await createAssetWithCollection(
      umi,
      { authority: collectionAuthority },
      { updateAuthority: collectionAuthority }
    );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: {
      type: 'Collection',
      address: originalCollection.publicKey,
    },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: originalCollection.publicKey,
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 1,
  });

  const newCollection = await createCollection(umi, {
    updateAuthority: collectionAuthority,
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.publicKey,
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 0,
    numMinted: 0,
  });

  await update(umi, {
    asset,
    collection: originalCollection,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [
      newCollection.publicKey,
    ]),
    newCollection: newCollection.publicKey,
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: newCollection.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: originalCollection.publicKey,
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 0,
    numMinted: 1,
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.publicKey,
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 0,
  });
});

test('it cannot change an asset collection if not both asset and collection auth', async (t) => {
  const umi = await createUmi();
  const originalCollectionAuthority = generateSigner(umi);
  const { asset, collection: originalCollection } =
    await createAssetWithCollection(
      umi,
      { authority: originalCollectionAuthority },
      { updateAuthority: originalCollectionAuthority }
    );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: {
      type: 'Collection',
      address: originalCollection.publicKey,
    },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: originalCollection.publicKey,
    updateAuthority: originalCollectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 1,
  });

  const newCollectionAuthority = generateSigner(umi);
  const newCollection = await createCollection(umi, {
    updateAuthority: newCollectionAuthority,
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.publicKey,
    updateAuthority: newCollectionAuthority.publicKey,
    currentSize: 0,
    numMinted: 0,
  });

  // Attempt to update using original collection authority.
  const result = update(umi, {
    asset,
    collection: originalCollection,
    newUpdateAuthority: updateAuthority('Collection', [
      newCollection.publicKey,
    ]),
    newCollection: newCollection.publicKey,
    authority: originalCollectionAuthority,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: {
      type: 'Collection',
      address: originalCollection.publicKey,
    },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: originalCollection.publicKey,
    updateAuthority: originalCollectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 1,
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.publicKey,
    updateAuthority: newCollectionAuthority.publicKey,
    currentSize: 0,
    numMinted: 0,
  });

  // Attempt to update using new collection authority.
  const result2 = update(umi, {
    asset,
    collection: originalCollection,
    newUpdateAuthority: updateAuthority('Collection', [
      newCollection.publicKey,
    ]),
    newCollection: newCollection.publicKey,
    authority: newCollectionAuthority,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result2, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: {
      type: 'Collection',
      address: originalCollection.publicKey,
    },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: originalCollection.publicKey,
    updateAuthority: originalCollectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 1,
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.publicKey,
    updateAuthority: newCollectionAuthority.publicKey,
    currentSize: 0,
    numMinted: 0,
  });
});

test('it can change an asset collection using delegate', async (t) => {
  const umi = await createUmi();
  const originalCollectionAuthority = generateSigner(umi);
  const { asset, collection: originalCollection } =
    await createAssetWithCollection(
      umi,
      { authority: originalCollectionAuthority },
      { updateAuthority: originalCollectionAuthority }
    );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: {
      type: 'Collection',
      address: originalCollection.publicKey,
    },
  });

  const newCollectionAuthority = generateSigner(umi);
  const newCollection = await createCollection(umi, {
    updateAuthority: newCollectionAuthority,
  });

  await addCollectionPlugin(umi, {
    collection: newCollection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [],
    },
    authority: newCollectionAuthority,
  }).sendAndConfirm(umi);

  await approveCollectionPluginAuthority(umi, {
    collection: newCollection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
    },
    newAuthority: {
      type: 'Address',
      address: originalCollectionAuthority.publicKey,
    },
    authority: newCollectionAuthority,
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.publicKey,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: originalCollectionAuthority.publicKey,
      },
      additionalDelegates: [],
    },
    updateAuthority: newCollectionAuthority.publicKey,
    currentSize: 0,
    numMinted: 0,
  });

  await update(umi, {
    asset,
    collection: originalCollection,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [
      newCollection.publicKey,
    ]),
    newCollection: newCollection.publicKey,
    authority: originalCollectionAuthority,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: newCollection.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.publicKey,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: originalCollectionAuthority.publicKey,
      },
      additionalDelegates: [],
    },
    updateAuthority: newCollectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 0,
  });
});

test('it can change an asset collection using same update authority (delegate exists but not used)', async (t) => {
  const umi = await createUmi();
  const collectionAuthority = generateSigner(umi);
  const { asset, collection: originalCollection } =
    await createAssetWithCollection(
      umi,
      { authority: collectionAuthority },
      { updateAuthority: collectionAuthority }
    );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: {
      type: 'Collection',
      address: originalCollection.publicKey,
    },
  });

  const newCollection = await createCollection(umi, {
    updateAuthority: collectionAuthority,
  });

  await addCollectionPlugin(umi, {
    collection: newCollection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [],
    },
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  const updateDelegate = generateSigner(umi);
  await approveCollectionPluginAuthority(umi, {
    collection: newCollection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
    },
    newAuthority: {
      type: 'Address',
      address: updateDelegate.publicKey,
    },
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.publicKey,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 0,
    numMinted: 0,
  });

  await update(umi, {
    asset,
    collection: originalCollection,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [
      newCollection.publicKey,
    ]),
    newCollection: newCollection.publicKey,
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: newCollection.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.publicKey,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 0,
  });
});

test('it can remove an asset from collection as Collection authority with update delegate on asset', async (t) => {
  const umi = await createUmi();
  const collectionAuthority = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    { authority: collectionAuthority },
    { updateAuthority: collectionAuthority }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 1,
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [],
    },
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  const updateDelegate = generateSigner(umi);
  await approvePluginAuthority(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
    },
    newAuthority: {
      type: 'Address',
      address: updateDelegate.publicKey,
    },
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
  });

  await update(umi, {
    asset,
    collection,
    newUpdateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
  });
});

test('it can remove an asset from collection using update delegate', async (t) => {
  const umi = await createUmi();
  const collectionAuthority = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    { authority: collectionAuthority },
    { updateAuthority: collectionAuthority }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 1,
  });

  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [],
    },
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  const updateDelegate = generateSigner(umi);
  await approveCollectionPluginAuthority(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
    },
    newAuthority: {
      type: 'Address',
      address: updateDelegate.publicKey,
    },
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 1,
  });

  await update(umi, {
    asset,
    collection,
    newUpdateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it can remove an asset from collection using additional update delegate', async (t) => {
  const umi = await createUmi();
  const collectionAuthority = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    { authority: collectionAuthority },
    { updateAuthority: collectionAuthority }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  const additionalDelegate = generateSigner(umi);
  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [additionalDelegate.publicKey],
    },
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [additionalDelegate.publicKey],
    },
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 1,
  });

  await update(umi, {
    asset,
    collection,
    newUpdateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    authority: additionalDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it cannot remove an asset from collection using update delegate on the asset', async (t) => {
  const umi = await createUmi();
  const collectionAuthority = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    { authority: collectionAuthority },
    { updateAuthority: collectionAuthority }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 1,
    numMinted: 1,
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [],
    },
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  const updateDelegate = generateSigner(umi);
  await approvePluginAuthority(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
    },
    newAuthority: {
      type: 'Address',
      address: updateDelegate.publicKey,
    },
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
  });

  const result = update(umi, {
    asset,
    collection,
    newUpdateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('it cannot remove an asset from collection using additional update delegate on the asset', async (t) => {
  const umi = await createUmi();
  const collectionAuthority = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    { authority: collectionAuthority },
    { updateAuthority: collectionAuthority }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  const additionalDelegate = generateSigner(umi);
  await addPlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [additionalDelegate.publicKey],
    },
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [additionalDelegate.publicKey],
    },
  });

  const result = update(umi, {
    asset,
    collection,
    newUpdateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    authority: additionalDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('it can add asset to collection using additional update delegate on new collection', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const collectionAuthority = generateSigner(umi);
  const collection = await createCollection(umi, {
    updateAuthority: collectionAuthority,
  });

  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [umi.identity.publicKey],
    },
    authority: collectionAuthority,
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [umi.identity.publicKey],
    },
    updateAuthority: collectionAuthority.publicKey,
    currentSize: 0,
    numMinted: 0,
  });

  await update(umi, {
    asset,
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
    newCollection: collection.publicKey,
    authority: umi.identity,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('it cannot add asset to collection if new collection contains permanent freeze delegate', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'PermanentFreezeDelegate',
        frozen: false,
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  const result = update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
    newCollection: collection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'PermanentDelegatesPreventMove' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('it cannot add asset to collection if new collection contains permanent transfer delegate', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'PermanentTransferDelegate',
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const result = update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
    newCollection: collection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'PermanentDelegatesPreventMove' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it cannot add asset to collection if new collection contains permanent burn delegate', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'PermanentBurnDelegate',
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    permanentBurnDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const result = update(umi, {
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
    newCollection: collection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'PermanentDelegatesPreventMove' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    permanentBurnDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});
