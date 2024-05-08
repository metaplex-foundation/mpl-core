import test from 'ava';

import { generateSigner } from '@metaplex-foundation/umi';
import {
  createPlugin,
  addCollectionPluginV1,
  pluginAuthorityPair,
  addPluginV1,
  updatePluginAuthority,
} from '../../../src';
import {
  DEFAULT_COLLECTION,
  assertCollection,
  createAsset,
  createCollection,
  createUmi,
} from '../../_setup';

test('it can add addBlocker to collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);

  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'AddBlocker',
    }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    addBlocker: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it cannot add UA-managed plugin to a collection if addBlocker had been added on creation', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'AddBlocker',
      }),
    ],
  });

  const result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it cannot add UA-managed plugin to an asset in a collection if addBlocker had been added on creation', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'AddBlocker',
      }),
    ],
  });
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
  });

  const result = addPluginV1(umi, {
    collection: collection.publicKey,
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it prevents plugins from being added to both collection and plugins when collection is created with AddBlocker', async (t) => {
  const umi = await createUmi();
  const updateAuthority = generateSigner(umi);
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'AddBlocker',
        authority: updatePluginAuthority(),
      }),
    ],
    updateAuthority,
  });
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    authority: updateAuthority,
  });

  let result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  result = addPluginV1(umi, {
    collection: collection.publicKey,
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it prevents plugins from being added to both collection and plugins when AddBlocker is added to a collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);
  const asset = await createAsset(umi, { collection: collection.publicKey });

  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'AddBlocker',
    }),
  }).sendAndConfirm(umi);

  let result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  result = addPluginV1(umi, {
    collection: collection.publicKey,
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});
