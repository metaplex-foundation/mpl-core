import { transferSol } from '@metaplex-foundation/mpl-toolbox';
import {
  createNoopSigner,
  generateSigner,
  publicKey,
  sol,
} from '@metaplex-foundation/umi';
import test from 'ava';

import {
  addCollectionPluginV1,
  create,
  createPlugin,
  execute,
  fetchAssetV1,
  findAssetSignerPda,
  PluginType,
  removeCollectionPluginV1,
  transferV1,
  updateCollectionPluginV1,
} from '../../../src';
import {
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createUmi,
  DEFAULT_ASSET,
} from '../../_setupRaw';
import { createCollection } from '../../_setupSdk';

test('it can add PermanentFreezeExecute to collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: true,
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('it can remove PermanentFreezeExecute from collection when unfrozen', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: false,
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  await removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.PermanentFreezeExecute,
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeExecute: undefined,
  });
});

test('it cannot remove PermanentFreezeExecute from collection when frozen', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: true,
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  const result = removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.PermanentFreezeExecute,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('it cannot add PermanentFreezeExecute to collection after creation', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi);

  const result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeExecute',
      data: { frozen: true },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeExecute: undefined,
  });
});

test('it can freeze and unfreeze a collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: false,
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  await updateCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeExecute',
      data: { frozen: true },
    }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  await updateCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeExecute',
      data: { frozen: false },
    }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('assets inherit PermanentFreezeExecute plugin from collection and execute is blocked when frozen', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(owner.publicKey, sol(1));

  // Create collection with PermanentFreezeExecute plugin frozen
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: true,
      },
    ],
  });

  // Create asset in the collection
  const assetSigner = generateSigner(umi);
  await create(umi, {
    asset: assetSigner,
    owner: owner.publicKey,
    collection,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, assetSigner.publicKey);

  // Asset should not have its own permanentFreezeExecute plugin (it inherits from collection)
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    permanentFreezeExecute: undefined, // No asset-level plugin
  });

  // Fund the asset signer PDA
  const [assetSignerPda] = findAssetSignerPda(umi, {
    asset: asset.publicKey,
  });

  await transferSol(umi, {
    source: umi.identity,
    destination: publicKey(assetSignerPda),
    amount: sol(0.5),
  }).sendAndConfirm(umi);

  // Execute should be blocked due to collection plugin
  const recipient = generateSigner(umi);
  const execResult = execute(umi, {
    asset,
    payer: owner,
    collection,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSignerPda)),
      destination: recipient.publicKey,
      amount: sol(0.1),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(execResult, { name: 'InvalidAuthority' });

  // Unfreeze at collection level should allow execute
  await updateCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeExecute',
      data: { frozen: false },
    }),
  }).sendAndConfirm(umi);

  // Now execute should succeed
  await execute(umi, {
    asset,
    payer: owner,
    collection,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSignerPda)),
      destination: recipient.publicKey,
      amount: sol(0.1),
    }),
  }).sendAndConfirm(umi);

  // Verify recipient received the SOL
  const recipientBalance = await umi.rpc.getBalance(recipient.publicKey);
  t.true(recipientBalance.basisPoints >= sol(0.1).basisPoints);
});

test('asset-level PermanentFreezeExecute overrides collection-level plugin when unfrozen', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(owner.publicKey, sol(1));

  // Create collection with PermanentFreezeExecute plugin frozen
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: true,
      },
    ],
  });

  // Create asset in the collection with its own unfrozen PermanentFreezeExecute plugin
  const assetSigner = generateSigner(umi);
  await create(umi, {
    asset: assetSigner,
    owner: owner.publicKey,
    collection,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: false,
      },
    ],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, assetSigner.publicKey);

  // Asset should have its own permanentFreezeExecute plugin
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  // Fund the asset signer PDA
  const [assetSignerPda] = findAssetSignerPda(umi, {
    asset: asset.publicKey,
  });

  await transferSol(umi, {
    source: umi.identity,
    destination: publicKey(assetSignerPda),
    amount: sol(0.5),
  }).sendAndConfirm(umi);

  // Execute should succeed because asset-level plugin overrides collection (asset plugin is unfrozen)
  const recipient = generateSigner(umi);
  await execute(umi, {
    asset,
    payer: owner,
    collection,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSignerPda)),
      destination: recipient.publicKey,
      amount: sol(0.1),
    }),
  }).sendAndConfirm(umi);

  // Verify recipient received the SOL
  const recipientBalance = await umi.rpc.getBalance(recipient.publicKey);
  t.true(recipientBalance.basisPoints >= sol(0.1).basisPoints);
});

test('collection PermanentFreezeExecute persists through asset transfer and still blocks execute', async (t) => {
  const umi = await createUmi();
  const originalOwner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  await umi.rpc.airdrop(originalOwner.publicKey, sol(1));
  await umi.rpc.airdrop(newOwner.publicKey, sol(1));

  // Create collection with PermanentFreezeExecute plugin frozen
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: true,
      },
    ],
  });

  // Create asset in the collection
  const assetSigner = generateSigner(umi);
  await create(umi, {
    asset: assetSigner,
    owner: originalOwner.publicKey,
    collection,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, assetSigner.publicKey);

  // Verify initial state - execute should be blocked
  const [assetSignerPda] = findAssetSignerPda(umi, {
    asset: asset.publicKey,
  });

  await transferSol(umi, {
    source: umi.identity,
    destination: publicKey(assetSignerPda),
    amount: sol(0.5),
  }).sendAndConfirm(umi);

  const recipient1 = generateSigner(umi);
  const execResult1 = execute(umi, {
    asset,
    payer: originalOwner,
    collection,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSignerPda)),
      destination: recipient1.publicKey,
      amount: sol(0.1),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(execResult1, { name: 'InvalidAuthority' });

  // Transfer the asset to a new owner
  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    authority: originalOwner,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  // Verify the asset has been transferred
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    permanentFreezeExecute: undefined, // No asset-level plugin
  });

  // Execute should still be blocked for the new owner due to inherited collection plugin
  const recipient2 = generateSigner(umi);
  const execResult2 = execute(umi, {
    asset,
    payer: newOwner,
    collection,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSignerPda)),
      destination: recipient2.publicKey,
      amount: sol(0.1),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(execResult2, { name: 'InvalidAuthority' });

  // Verify that the collection plugin state persisted through the asset transfer
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  // Unfreeze collection plugin and verify execute now works for new owner
  await updateCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeExecute',
      data: { frozen: false },
    }),
  }).sendAndConfirm(umi);

  // Execute should now succeed for the new owner
  await execute(umi, {
    asset,
    payer: newOwner,
    collection,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSignerPda)),
      destination: recipient2.publicKey,
      amount: sol(0.1),
    }),
  }).sendAndConfirm(umi);

  // Verify recipient received the SOL
  const recipientBalance = await umi.rpc.getBalance(recipient2.publicKey);
  t.true(recipientBalance.basisPoints >= sol(0.1).basisPoints);
});

test('collection owner cannot remove or unfreeze PermanentFreezeExecute plugin', async (t) => {
  const umi = await createUmi();
  const collectionOwner = generateSigner(umi);

  // Create collection with PermanentFreezeExecute plugin (frozen) and specific owner
  const collection = await createCollection(umi, {
    updateAuthority: collectionOwner.publicKey,
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: true,
      },
    ],
  });

  // Verify initial state
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: collectionOwner.publicKey,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  // Create a different user who will try to modify the plugin (not the update authority)
  const unauthorizedUser = generateSigner(umi);

  // Unauthorized user should not be able to unfreeze the plugin
  const unfreezeResult = updateCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeExecute',
      data: { frozen: false },
    }),
    authority: unauthorizedUser,
  }).sendAndConfirm(umi);

  await t.throwsAsync(unfreezeResult, { name: 'InvalidAuthority' });

  // Unauthorized user should not be able to remove the plugin (even when frozen)
  const removeResult = removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.PermanentFreezeExecute,
    authority: unauthorizedUser,
  }).sendAndConfirm(umi);

  await t.throwsAsync(removeResult, { name: 'InvalidAuthority' });

  // Plugin should still be there and frozen
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: collectionOwner.publicKey,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  // Only the collection's update authority should be able to unfreeze
  await updateCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeExecute',
      data: { frozen: false },
    }),
    authority: collectionOwner, // Collection update authority
  }).sendAndConfirm(umi);

  // Unauthorized user still cannot remove it even when unfrozen
  const removeUnfrozenResult = removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.PermanentFreezeExecute,
    authority: unauthorizedUser,
  }).sendAndConfirm(umi);

  await t.throwsAsync(removeUnfrozenResult, { name: 'InvalidAuthority' });

  // But collection update authority can remove it when unfrozen
  await removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.PermanentFreezeExecute,
    authority: collectionOwner,
  }).sendAndConfirm(umi);

  // Verify plugin is removed
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: collectionOwner.publicKey,
    permanentFreezeExecute: undefined,
  });
});
