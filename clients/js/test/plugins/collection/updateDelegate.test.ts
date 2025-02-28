import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import { SPL_SYSTEM_PROGRAM_ID } from '@metaplex-foundation/mpl-toolbox';
import {
  addCollectionPlugin,
  addPlugin,
  approveCollectionPluginAuthority,
  approvePluginAuthority,
  AssetV1,
  createV2,
  fetchAssetV1,
  revokeCollectionPluginAuthority,
  revokePluginAuthority,
  update,
  updateAuthority,
  updateCollection,
  updateCollectionPlugin,
  updatePlugin,
  updateV2,
} from '../../../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createUmi,
} from '../../_setupRaw';
import {
  createAsset,
  createAssetWithCollection,
  createCollection,
} from '../../_setupSdk';

test('it can create a new asset with a collection if it is the collection updateDelegate', async (t) => {
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [],
      },
    ],
  });

  await approveCollectionPluginAuthority(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
    },

    newAuthority: { type: 'Address', address: updateDelegate.publicKey },
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

test('it can create a new asset with a collection if it is a collection updateDelegate additionalDelegate', async (t) => {
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [updateDelegate.publicKey],
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });

  umi.identity = updateDelegate;
  umi.payer = updateDelegate;
  const owner = generateSigner(umi);

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
  const umi = await createUmi();
  const updateDelegate = generateSigner(umi);

  const collection = await createCollection(umi);
  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [],
    },
  }).sendAndConfirm(umi);

  await approveCollectionPluginAuthority(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
    },
    newAuthority: { type: 'Address', address: updateDelegate.publicKey },
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

test('it can create a collection with updateDelegate with additional delegates', async (t) => {
  const umi = await createUmi();
  const updateDelegate = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [updateDelegate.publicKey],
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: { type: 'UpdateAuthority' },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });
});

test('it can add updateDelegate to collection with additional delegates', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);
  const updateDelegate = generateSigner(umi);

  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [updateDelegate.publicKey],
    },
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: { type: 'UpdateAuthority' },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });
});

test('it can update updateDelegate on collection with additional delegates', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);
  const updateDelegate = generateSigner(umi);

  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [],
    },
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

  await updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [updateDelegate.publicKey],
    },
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });
});

test('an updateDelegate on collection can update an asset', async (t) => {
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [],
      },
    ],
  });

  await approveCollectionPluginAuthority(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
    },
    newAuthority: { type: 'Address', address: updateDelegate.publicKey },
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
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    owner,
    authority: updateDelegate,
    name: 'short',
    uri: 'https://short.com',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });

  await update(umi, {
    asset,
    collection,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('an updateDelegate additionalDelegate on collection can update an asset', async (t) => {
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [updateDelegate.publicKey],
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });

  umi.identity = updateDelegate;
  umi.payer = updateDelegate;
  const owner = generateSigner(umi);
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    owner,
    authority: updateDelegate,
    name: 'short',
    uri: 'https://short.com',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });

  await update(umi, {
    asset,
    collection,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('an updateDelegate on collection cannot update an asset after delegate authority revoked', async (t) => {
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [],
      },
    ],
  });

  await approveCollectionPluginAuthority(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
    },
    newAuthority: { type: 'Address', address: updateDelegate.publicKey },
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
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    owner,
    authority: updateDelegate,
    name: 'short',
    uri: 'https://short.com',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });

  await revokeCollectionPluginAuthority(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'UpdateDelegate',
    },
  }).sendAndConfirm(umi);

  const result = update(umi, {
    asset,
    collection,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });
});

test('an updateDelegate additionalDelegate on collection cannot update an asset after delegate authority revoked', async (t) => {
  const umi = await createUmi();
  const { identity } = umi;
  const updateDelegate = await generateSignerWithSol(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [updateDelegate.publicKey],
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });

  umi.identity = updateDelegate;
  umi.payer = updateDelegate;
  const owner = generateSigner(umi);
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    owner,
    authority: updateDelegate,
    name: 'short',
    uri: 'https://short.com',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });

  await updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    authority: identity,
    plugin: {
      type: 'UpdateDelegate',
      additionalDelegates: [],
    },
  }).sendAndConfirm(umi);

  const result = update(umi, {
    asset,
    collection,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    name: 'short',
    uri: 'https://short.com',
  });
});

test('it can update a non-updateDelegate plugin on an asset as collection update additional delegate', async (t) => {
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      plugins: [
        {
          type: 'Edition',
          number: 1,
        },
      ],
    },
    {
      plugins: [
        {
          type: 'UpdateDelegate',
          additionalDelegates: [updateDelegate.publicKey],
        },
      ],
    }
  );

  await updatePlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'Edition',
      number: 2,
    },
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    edition: {
      authority: { type: 'UpdateAuthority' },
      number: 2,
    },
  });
});

test('it can approve/revoke non-updateDelegate plugin on an asset as collection update additional delegate', async (t) => {
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);
  const updateDelegate2 = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      plugins: [
        {
          type: 'Edition',
          number: 1,
        },
      ],
    },
    {
      plugins: [
        {
          type: 'UpdateDelegate',
          additionalDelegates: [updateDelegate.publicKey],
        },
      ],
    }
  );

  await approvePluginAuthority(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'Edition',
    },
    newAuthority: { type: 'Address', address: updateDelegate2.publicKey },
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    edition: {
      authority: { type: 'Address', address: updateDelegate2.publicKey },
      number: 1,
    },
  });

  await revokePluginAuthority(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'Edition',
    },
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    edition: {
      authority: { type: 'UpdateAuthority' },
      number: 1,
    },
  });
});

test('it can update the update authority of the collection as an updateDelegate additional delegate', async (t) => {
  const umi = await createUmi();
  const updateDelegate = generateSigner(umi);
  const updateDelegate2 = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [updateDelegate.publicKey],
      },
    ],
  });

  await updateCollection(umi, {
    collection: collection.publicKey,
    authority: updateDelegate,
    newUpdateAuthority: updateDelegate2.publicKey,
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: updateDelegate2.publicKey,
  });
});

test('it can update the update authority of the collection as an updateDelegate root authority', async (t) => {
  const umi = await createUmi();
  const updateDelegate = generateSigner(umi);
  const updateDelegate2 = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        authority: { type: 'Address', address: updateDelegate.publicKey },
        additionalDelegates: [],
      },
    ],
  });

  await updateCollection(umi, {
    collection: collection.publicKey,
    authority: updateDelegate,
    newUpdateAuthority: updateDelegate2.publicKey,
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: updateDelegate2.publicKey,
  });
});

test('it can update collection details as an updateDelegate additional delegate', async (t) => {
  const umi = await createUmi();
  const updateDelegate = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [updateDelegate.publicKey],
      },
    ],
  });

  await updateCollection(umi, {
    collection: collection.publicKey,
    authority: updateDelegate,
    name: 'new name',
    uri: 'new uri',
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    name: 'new name',
    uri: 'new uri',
  });
});

test('it can update an authority-managed plugin on an asset as collection update additional delegate', async (t) => {
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      plugins: [
        {
          type: 'PermanentFreezeDelegate',
          frozen: true,
        },
      ],
    },
    {
      plugins: [
        {
          type: 'UpdateDelegate',
          additionalDelegates: [updateDelegate.publicKey],
        },
      ],
    }
  );

  await updatePlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'PermanentFreezeDelegate',
      frozen: false,
    },
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    permanentFreezeDelegate: {
      authority: { type: 'UpdateAuthority' },
      frozen: false,
    },
  });
});

test('it cannot update an authority-managed plugin on an asset as collection update additional delegate if the plugin authority is not UpdateAuthority', async (t) => {
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      plugins: [
        {
          type: 'PermanentFreezeDelegate',
          authority: { type: 'Address', address: SPL_SYSTEM_PROGRAM_ID },
          frozen: true,
        },
      ],
    },
    {
      plugins: [
        {
          type: 'UpdateDelegate',
          additionalDelegates: [updateDelegate.publicKey],
        },
      ],
    }
  );

  const result = updatePlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'PermanentFreezeDelegate',
      frozen: false,
    },
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    permanentFreezeDelegate: {
      authority: { type: 'Address', address: SPL_SYSTEM_PROGRAM_ID },
      frozen: true,
    },
  });
});

test('it cannot update an owner-managed plugin on an asset as collection update additional delegate', async (t) => {
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      plugins: [
        {
          type: 'FreezeDelegate',
          frozen: true,
        },
      ],
    },
    {
      plugins: [
        {
          type: 'UpdateDelegate',
          additionalDelegates: [updateDelegate.publicKey],
        },
      ],
    }
  );

  const result = updatePlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'FreezeDelegate',
      frozen: false,
    },
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    freezeDelegate: {
      authority: { type: 'Owner' },
      frozen: true,
    },
  });
});

test('it can update an owner-managed plugin on an asset as collection update additional delegate if the plugin authority is UpdateAuthority', async (t) => {
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      plugins: [
        {
          type: 'FreezeDelegate',
          authority: { type: 'UpdateAuthority' },
          frozen: true,
        },
      ],
    },
    {
      plugins: [
        {
          type: 'UpdateDelegate',
          additionalDelegates: [updateDelegate.publicKey],
        },
      ],
    }
  );

  await updatePlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'FreezeDelegate',
      frozen: false,
    },
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    freezeDelegate: {
      authority: { type: 'UpdateAuthority' },
      frozen: false,
    },
  });
});

test('it can add asset to collection as collection update delegate', async (t) => {
  const umi = await createUmi();
  const assetOwner = umi.identity;
  const collectionUmi = await createUmi();

  const assetSigner = generateSigner(umi);
  const assetAddress = assetSigner.publicKey;
  await createV2(umi, {
    asset: assetSigner,
    name: 'My Asset',
    uri: 'https://example.com/my-asset.json',
  }).sendAndConfirm(umi);

  const collectionSigner = generateSigner(collectionUmi);
  const collectionAddress = collectionSigner.publicKey;
  await createCollection(collectionUmi, {
    collection: collectionSigner,
    name: 'My Collection',
    uri: 'https://example.com/my-collection.json',
  });

  await addCollectionPlugin(collectionUmi, {
    collection: collectionAddress,
    plugin: {
      type: 'UpdateDelegate',
      authority: { type: 'Address', address: assetOwner.publicKey },
      additionalDelegates: [],
    },
  }).sendAndConfirm(collectionUmi);

  await updateV2(umi, {
    asset: assetAddress,
    newCollection: collectionAddress,
    newUpdateAuthority: updateAuthority('Collection', [collectionAddress]),
    authority: assetOwner,
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, assetAddress);
  t.like(asset, <Partial<AssetV1>>{
    owner: assetOwner.publicKey,
    updateAuthority: {
      type: 'Collection',
      address: collectionAddress,
    },
  });
});

test('it can add asset to collection as collection additional delegate', async (t) => {
  const umi = await createUmi();
  const assetOwner = umi.identity;
  const collectionUmi = await createUmi();

  const assetSigner = generateSigner(umi);
  const assetAddress = assetSigner.publicKey;
  await createV2(umi, {
    asset: assetSigner,
    name: 'My Asset',
    uri: 'https://example.com/my-asset.json',
  }).sendAndConfirm(umi);

  const collectionSigner = generateSigner(collectionUmi);
  const collectionAddress = collectionSigner.publicKey;
  await createCollection(collectionUmi, {
    collection: collectionSigner,
    name: 'My Collection',
    uri: 'https://example.com/my-collection.json',
  });

  await addCollectionPlugin(collectionUmi, {
    collection: collectionAddress,
    plugin: {
      type: 'UpdateDelegate',
      // Some other address
      authority: { type: 'Address', address: generateSigner(umi).publicKey },
      // Who we want to be able to add collection assets
      additionalDelegates: [assetOwner.publicKey],
    },
  }).sendAndConfirm(collectionUmi);

  await updateV2(umi, {
    asset: assetAddress,
    newCollection: collectionAddress,
    newUpdateAuthority: updateAuthority('Collection', [collectionAddress]),
    authority: assetOwner,
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, assetAddress);
  t.like(asset, <Partial<AssetV1>>{
    owner: assetOwner.publicKey,
    updateAuthority: {
      type: 'Collection',
      address: collectionAddress,
    },
  });
});

test('it can add asset to collection with collection owner as update delegate', async (t) => {
  const umi = await createUmi();
  const assetOwner = umi.identity;
  const collectionUmi = await createUmi();
  const collectionOwner = collectionUmi.identity;

  const assetSigner = generateSigner(umi);
  await createV2(umi, {
    asset: assetSigner,
    name: 'My Asset',
    uri: 'https://example.com/my-asset.json',
  }).sendAndConfirm(umi);

  const collectionSigner = generateSigner(collectionUmi);
  const collectionAddress = collectionSigner.publicKey;
  await createCollection(collectionUmi, {
    collection: collectionSigner,
    name: 'My Collection',
    uri: 'https://example.com/my-collection.json',
  });

  await addPlugin(umi, {
    asset: assetSigner.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      authority: { type: 'Address', address: collectionOwner.publicKey },
      additionalDelegates: [],
    },
  }).sendAndConfirm(umi);

  await updateV2(collectionUmi, {
    asset: assetSigner.publicKey,
    authority: collectionOwner,
    newCollection: collectionAddress,
    newUpdateAuthority: updateAuthority('Collection', [collectionAddress]),
  }).sendAndConfirm(collectionUmi);

  const asset = await fetchAssetV1(umi, assetSigner.publicKey);
  t.like(asset, <Partial<AssetV1>>{
    owner: assetOwner.publicKey,
    updateAuthority: {
      type: 'Collection',
      address: collectionAddress,
    },
  });
});
