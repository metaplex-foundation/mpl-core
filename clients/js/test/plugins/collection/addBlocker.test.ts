import test from 'ava';

import { generateSigner } from '@metaplex-foundation/umi';
import {
  DEFAULT_COLLECTION,
  assertCollection,
  createUmi,
} from '../../_setupRaw';
import { createAsset, createCollection } from '../../_setupSdk';
import { addCollectionPlugin, addPlugin } from '../../../src';

test('it can add addBlocker to collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);

  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'AddBlocker',
    },
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
      {
        type: 'AddBlocker',
      },
    ],
  });

  const result = addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it cannot add UA-managed plugin to an asset in a collection if addBlocker had been added on creation', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'AddBlocker',
      },
    ],
  });
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
  });

  const result = addPlugin(umi, {
    collection: collection.publicKey,
    asset: asset.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
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
      {
        type: 'AddBlocker',
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

  let result = addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  result = addPlugin(umi, {
    collection: collection.publicKey,
    asset: asset.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it prevents plugins from being added to both collection and plugins when AddBlocker is added to a collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);
  const asset = await createAsset(umi, { collection: collection.publicKey });

  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'AddBlocker',
    },
  }).sendAndConfirm(umi);

  let result = addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  result = addPlugin(umi, {
    collection: collection.publicKey,
    asset: asset.publicKey,
    plugin: {
      type: 'ImmutableMetadata',
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});
