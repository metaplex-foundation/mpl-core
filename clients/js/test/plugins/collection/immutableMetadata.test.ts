import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  createPlugin,
  addCollectionPluginV1,
  updateV1,
  updateCollectionV1,
  updatePluginAuthority,
  pluginAuthorityPair,
} from '../../../src';
import {
  DEFAULT_COLLECTION,
  assertCollection,
  createAsset,
  createCollection,
  createUmi,
} from '../../_setup';

test('it can add immutableMetadata to collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);

  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    immutableMetadata: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can prevent collection assets metadata from being updated', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);

  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  }).sendAndConfirm(umi);

  const asset = await createAsset(umi, {
    collection: collection.publicKey,
  });

  const result = updateV1(umi, {
    collection: collection.publicKey,
    asset: asset.publicKey,
    newName: 'Test Bread 3',
    newUri: 'https://example.com/bread3',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it states that UA is the only one who can add the ImmutableMetadata', async (t) => {
  const umi = await createUmi();
  const updateAuthority = generateSigner(umi);
  const randomUser = generateSigner(umi);
  const collection = await createCollection(umi, { updateAuthority });

  // random keypair can't add ImmutableMetadata
  let result = addCollectionPluginV1(umi, {
    authority: randomUser,
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  // Owner can't add ImmutableMetadata
  result = addCollectionPluginV1(umi, {
    authority: umi.identity,
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  // UA CAN add ImmutableMetadata
  await addCollectionPluginV1(umi, {
    authority: updateAuthority,
    collection: collection.publicKey,
    plugin: createPlugin({ type: 'ImmutableMetadata' }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    immutableMetadata: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it prevents both collection and asset from their meta updating when ImmutableMetadata is added', async (t) => {
  const umi = await createUmi();
  const updateAuthority = generateSigner(umi);
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'ImmutableMetadata',
        authority: updatePluginAuthority(),
      }),
    ],
    updateAuthority,
  });
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    authority: updateAuthority,
  });

  let result = updateV1(umi, {
    collection: collection.publicKey,
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  result = updateCollectionV1(umi, {
    authority: updateAuthority,
    collection: collection.publicKey,
    newName: 'Test',
    newUri: 'Test',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});
