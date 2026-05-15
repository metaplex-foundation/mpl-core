import test from 'ava';
import { generateSigner, PublicKey } from '@metaplex-foundation/umi';
import {
  addCollectionPluginV1,
  createPlugin,
  removeCollectionPluginV1,
  PluginType,
  addCollectionPlugin,
  addPlugin,
  ExternalPluginAdapterSchema,
} from '../../../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createUmi,
} from '../../_setupRaw';
import { createAsset, createCollection } from '../../_setupSdk';

const MPL_BUBBLEGUM_PROGRAM_ID =
  'BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY' as PublicKey<'BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY'>;

test('it can create collection with BubblegumV2 plugin', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    bubblegumV2: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
  });
});

test('it cannot add BubblegumV2 to collection after creation', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);

  const result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'BubblegumV2',
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    bubblegumV2: undefined,
  });
});

test('Update Authority cannot remove BubblegumV2 from collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    bubblegumV2: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
  });

  const result = removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.BubblegumV2,
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
    bubblegumV2: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
  });
});

test('it can create collection with BubblegumV2 plugin and other allow-listed plugins', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
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
    bubblegumV2: {
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

test('it cannot create collection with BubblegumV2 plugin and non-allow-listed plugins', async (t) => {
  const umi = await createUmi();
  const result = createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
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
    name: 'BlockedByBubblegumV2',
  });
});

test('it can add allow-listed plugins to collection with BubblegumV2 plugin', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    bubblegumV2: {
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

test('it cannot add non-allow-listed plugins to collection with BubblegumV2 plugin', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    bubblegumV2: {
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

test('it cannot create collection with BubblegumV2 plugin and external plugin', async (t) => {
  const umi = await createUmi();
  const result = createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
      },
      {
        type: 'AppData',
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  await t.throwsAsync(result, {
    name: 'BlockedByBubblegumV2',
  });
});

test('it cannot add external plugin to collection with BubblegumV2 plugin', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    currentSize: 0,
    numMinted: 0,
    bubblegumV2: {
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

test('it cannot create collection with BubblegumV2 plugin using wrong authority', async (t) => {
  const umi = await createUmi();
  const updateAuthorityResult = createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
        authority: {
          type: 'UpdateAuthority',
        },
      },
    ],
  });

  await t.throwsAsync(updateAuthorityResult, {
    name: 'InvalidAuthority',
  });

  const noneResult = createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
        authority: {
          type: 'None',
        },
      },
    ],
  });

  await t.throwsAsync(noneResult, {
    name: 'InvalidAuthority',
  });

  const ownerResult = createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
        authority: {
          type: 'Owner',
        },
      },
    ],
  });

  await t.throwsAsync(ownerResult, {
    name: 'InvalidAuthority',
  });

  const addressResult = createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
        authority: {
          type: 'Address',
          address: generateSigner(umi).publicKey,
        },
      },
    ],
  });

  await t.throwsAsync(addressResult, {
    name: 'InvalidAuthority',
  });
});

test('it can add non-allow-listed plugin to asset in BubblegumV2 collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
      },
    ],
  });

  const asset = await createAsset(umi, {
    collection,
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'FreezeDelegate',
      frozen: false,
    },
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

test('it can add external plugin adapter to asset in BubblegumV2 collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
      },
    ],
  });

  const asset = await createAsset(umi, {
    collection,
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'AppData',
      dataAuthority: { type: 'UpdateAuthority' },
      schema: ExternalPluginAdapterSchema.Json,
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });
});

test('it can create asset with non-allow-listed plugins in BubblegumV2 collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
      },
    ],
  });

  const asset = await createAsset(umi, {
    collection,
    plugins: [
      {
        type: 'FreezeDelegate',
        frozen: false,
      },
      {
        type: 'TransferDelegate',
      },
    ],
  });

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
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can create asset with external plugin adapter in BubblegumV2 collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
      },
    ],
  });

  const asset = await createAsset(umi, {
    collection,
    plugins: [
      {
        type: 'AppData',
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });
});
