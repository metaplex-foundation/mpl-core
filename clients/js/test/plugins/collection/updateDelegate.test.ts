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
  updateCollectionPluginV1,
} from '../../../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createCollection,
  createUmi,
} from '../../_setup';

test('it can create a new asset with a collection if it is the collection update delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  // When we create a new account.
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
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

test('it can add updateDelegate to collection and then approve', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateDelegate = generateSigner(umi);

  const collection = await createCollection(umi);
  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [] },
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


test('it cannot create a collection with updateDelegate with additional delegates', async (t) => {
  // Given a Umi instance.
  const umi = await createUmi();

  // When we attempt to create a new collection with update delegate with additional delegates.
  let result = createCollection(umi, {
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

test('it cannot add updateDelegate with additional delegates', async (t) => {
  // Given a Umi instance and a new collection.
  const umi = await createUmi();
  const collection = await createCollection(umi);

  // When we attempt to add an update delegate with additional delegates.
  const result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [generateSigner(umi).publicKey] },
    }),
  }).sendAndConfirm(umi);

  // The program does not allow it.
  await t.throwsAsync(result, { name: 'NotAvailable' });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});

test('it cannot update updateDelegate with additional delegates', async (t) => {
  // Given a Umi instance and a new collection.
  const umi = await createUmi();
  const collection = await createCollection(umi);

  // And the collection has an existing update delegate.
  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [] },
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

  // And we attempt to add an update delegate with additional delegates.
  const result = updateCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [generateSigner(umi).publicKey] },
    }),
  }).sendAndConfirm(umi);

  // The program does not allow it.
  await t.throwsAsync(result, { name: 'NotAvailable' });

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
});
