import test from 'ava';

import { generateSigner } from '@metaplex-foundation/umi';
import {
  updateV2,
  pluginAuthorityPair,
  updateAuthority,
  updateCollectionV1,
  addCollectionPlugin,
  approveCollectionPluginAuthority,
} from '../src';
import {
  assertAsset,
  assertCollection,
  createAsset,
  createAssetWithCollection,
  createCollection,
  createUmi,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
} from './_setupRaw';

test('it can update an asset to be larger', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);

  await updateV2(umi, {
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

test('it cannot update an asset using asset as authority', async (t) => {
  const umi = await createUmi();
  const myAsset = generateSigner(umi);

  const asset = await createAsset(umi, {
    asset: myAsset,
    name: 'short',
    uri: 'https://short.com',
  });

  const result = updateV2(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
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

  await updateV2(umi, {
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

  await updateV2(umi, {
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

  await updateV2(umi, {
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

test('it can update an asset update authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const newUpdateAuthority = generateSigner(umi);

  await updateV2(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
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
  const result = updateV2(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
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

  const result = updateV2(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program for assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const fakeLogWrapper = generateSigner(umi);

  const result = updateV2(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});

test('it cannot use an invalid system program for collections', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collection = await createCollection(umi);
  const fakeSystemProgram = generateSigner(umi);

  const result = updateCollectionV1(umi, {
    collection: collection.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program for collections', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collection = await createCollection(umi);
  const fakeLogWrapper = generateSigner(umi);

  const result = updateCollectionV1(umi, {
    collection: collection.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});

test('it can remove an asset from a collection using updateV2', async (t) => {
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

  await updateV2(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    collection: collection.publicKey,
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

test('it cannot remove an asset from a collection when missing collection account', async (t) => {
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(umi, {}, {});

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  const result = updateV2(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'MissingCollection' });
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

  const result = updateV2(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    collection: generateSigner(umi).publicKey,
    newUpdateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });
});

test('it can update an asset update authority to be part of a collection using updateV2', async (t) => {
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

  await updateV2(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
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
    numMinted: 1,
  });
});

test('it cannot update an asset update authority to be part of a collection when missing collection account', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const collection = await createCollection(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const result = updateV2(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'MissingCollection' });
});

test('it cannot update an asset update authority to be part of a collection when using incorrect collection account', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const collection = await createCollection(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const result = updateV2(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
    newCollection: generateSigner(umi).publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });
});

test('it cannot update an asset using only new collection authority', async (t) => {
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

  const result = updateV2(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthority('Collection', [collection.publicKey]),
    newCollection: collection.publicKey,
    authority: newCollectionAuthority,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });
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

  await updateV2(umi, {
    asset: asset.publicKey,
    collection: originalCollection.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
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
});
