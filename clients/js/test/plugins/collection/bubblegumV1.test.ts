import test from 'ava';
import { PublicKey } from '@metaplex-foundation/umi';
import {
  addCollectionPluginV1,
  createPlugin,
  removeCollectionPluginV1,
  PluginType,
  addCollectionPlugin,
  ExternalPluginAdapterSchema,
} from '../../../src';
import {
  DEFAULT_COLLECTION,
  assertCollection,
  createUmi,
} from '../../_setupRaw';
import { createCollection } from '../../_setupSdk';

const MPL_BUBBLEGUM_PROGRAM_ID =
  'BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY' as PublicKey<'BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY'>;

test('it can create collection with BubblegumV1 plugin', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV1',
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    bubblegumV1: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
  });
});

test('it cannot add BubblegumV1 to collection after creation', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);

  const result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'BubblegumV1',
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    bubblegumV1: undefined,
  });
});

test('Update Authority cannot remove BubblegumV1 from collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV1',
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    bubblegumV1: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
  });

  const result = removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.BubblegumV1,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    bubblegumV1: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
  });
});

test('it can create collection with BubblegumV1 plugin and other allow-listed plugins', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV1',
      },
      {
        type: 'UpdateDelegate',
        additionalDelegates: [],
      },
      {
        type: 'PermanentFreezeDelegate',
        frozen: false,
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    bubblegumV1: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('it cannot create collection with BubblegumV1 plugin and non-allow-listed plugins', async (t) => {
  const umi = await createUmi();
  const result = createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV1',
      },
      {
        type: 'UpdateDelegate',
        additionalDelegates: [],
      },
      {
        type: 'PermanentFreezeDelegate',
        frozen: false,
      },
      {
        type: 'MasterEdition',
        maxSupply: 100,
        name: 'master',
        uri: 'uri master',
      },
    ],
  });

  await t.throwsAsync(result, {
    name: 'BlockedByBubblegumV1',
  });
});

test('it can add allow-listed plugins to collection with BubblegumV1 plugin', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV1',
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    bubblegumV1: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
  });

  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'Royalties',
      basisPoints: 500,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: {
        type: 'ProgramDenyList',
        addresses: [umi.identity.publicKey],
      },
    },
  }).sendAndConfirm(umi);
});

test('it cannot add non-allow-listed plugins to collection with BubblegumV1 plugin', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV1',
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    bubblegumV1: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
  });

  const result = addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'AddBlocker',
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it cannot create collection with BubblegumV1 plugin and external plugin', async (t) => {
  const umi = await createUmi();
  const result = createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV1',
      },
      {
        type: 'AppData',
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  await t.throwsAsync(result, {
    name: 'BlockedByBubblegumV1',
  });
});

test('it cannot add external plugin to collection with BubblegumV1 plugin', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV1',
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    bubblegumV1: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
  });

  const result = addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'AppData',
      dataAuthority: { type: 'UpdateAuthority' },
      schema: ExternalPluginAdapterSchema.Json,
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});
