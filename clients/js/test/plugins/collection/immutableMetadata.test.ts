import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';

import {
  DEFAULT_COLLECTION,
  assertCollection,
  createUmi,
} from '../../_setupRaw';
import { addCollectionPlugin, update, updateCollection } from '../../../src';
import { createAsset, createCollection } from '../../_setupSdk';

test('it can add immutableMetadata to collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);

  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
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

  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
  }).sendAndConfirm(umi);

  const asset = await createAsset(umi, {
    collection: collection.publicKey,
  });

  const result = update(umi, {
    collection,
    asset,
    name: 'Test Bread 3',
    uri: 'https://example.com/bread3',
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
  let result = addCollectionPlugin(umi, {
    authority: randomUser,
    collection: collection.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  // Payer for the the collection can't add ImmutableMetadata
  result = addCollectionPlugin(umi, {
    authority: umi.identity,
    collection: collection.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  // UA CAN add ImmutableMetadata
  await addCollectionPlugin(umi, {
    authority: updateAuthority,
    collection: collection.publicKey,
    plugin: { type: 'ImmutableMetadata' },
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
      {
        type: 'ImmutableMetadata',
        authority: {
          type: 'UpdateAuthority',
        },
      },
    ],
    updateAuthority,
  });
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    authority: updateAuthority,
  });

  let result = update(umi, {
    collection,
    asset,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  result = updateCollection(umi, {
    authority: updateAuthority,
    collection: collection.publicKey,
    name: 'Test',
    uri: 'Test',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});
