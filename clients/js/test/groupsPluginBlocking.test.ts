import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';

import {
  PluginType,
  addAssetsToGroup,
  addCollectionPluginV1,
  addCollectionsToGroup,
  addPluginV1,
  addressPluginAuthority,
  approveCollectionPluginAuthorityV1,
  approvePluginAuthorityV1,
  createPlugin,
  pluginAuthorityPair,
  removeCollectionPluginV1,
  removePluginV1,
  revokeCollectionPluginAuthorityV1,
  revokePluginAuthorityV1,
  updateCollectionPluginV1,
  updatePluginV1,
} from '../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createCollection,
  createGroup,
  createUmi,
} from './_setupRaw';

test('it cannot create an asset with a Groups plugin', async (t) => {
  const umi = await createUmi();

  await t.throwsAsync(
    createAsset(umi, {
      plugins: [
        pluginAuthorityPair({
          type: 'Groups',
          data: { groups: [umi.identity.publicKey] },
        }),
      ],
    }),
    { name: 'InvalidPlugin' }
  );
});

test('it cannot create a collection with a Groups plugin', async (t) => {
  const umi = await createUmi();

  await t.throwsAsync(
    createCollection(umi, {
      plugins: [
        pluginAuthorityPair({
          type: 'Groups',
          data: { groups: [umi.identity.publicKey] },
        }),
      ],
    }),
    { name: 'InvalidPlugin' }
  );
});

test('it blocks generic asset plugin operations for Groups', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const group = await createGroup(umi);
  const delegate = generateSigner(umi);

  await t.throwsAsync(
    addPluginV1(umi, {
      asset: asset.publicKey,
      plugin: createPlugin({
        type: 'Groups',
        data: { groups: [group.publicKey] },
      }),
    }).sendAndConfirm(umi),
    { name: 'InvalidPlugin' }
  );

  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: asset.publicKey },
    ])
    .sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    groups: {
      authority: { type: 'UpdateAuthority' },
      groups: [group.publicKey],
    },
  });

  await t.throwsAsync(
    updatePluginV1(umi, {
      asset: asset.publicKey,
      plugin: createPlugin({
        type: 'Groups',
        data: { groups: [] },
      }),
    }).sendAndConfirm(umi),
    { name: 'InvalidPlugin' }
  );

  await t.throwsAsync(
    removePluginV1(umi, {
      asset: asset.publicKey,
      pluginType: PluginType.Groups,
    }).sendAndConfirm(umi),
    { name: 'InvalidPlugin' }
  );

  await t.throwsAsync(
    approvePluginAuthorityV1(umi, {
      asset: asset.publicKey,
      pluginType: PluginType.Groups,
      newAuthority: addressPluginAuthority(delegate.publicKey),
    }).sendAndConfirm(umi),
    { name: 'InvalidPlugin' }
  );

  await t.throwsAsync(
    revokePluginAuthorityV1(umi, {
      asset: asset.publicKey,
      pluginType: PluginType.Groups,
    }).sendAndConfirm(umi),
    { name: 'InvalidPlugin' }
  );
});

test('it blocks generic collection plugin operations for Groups', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);
  const group = await createGroup(umi);
  const delegate = generateSigner(umi);

  await t.throwsAsync(
    addCollectionPluginV1(umi, {
      collection: collection.publicKey,
      plugin: createPlugin({
        type: 'Groups',
        data: { groups: [group.publicKey] },
      }),
    }).sendAndConfirm(umi),
    { name: 'InvalidPlugin' }
  );

  await addCollectionsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: collection.publicKey },
    ])
    .sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    groups: {
      authority: { type: 'UpdateAuthority' },
      groups: [group.publicKey],
    },
  });

  await t.throwsAsync(
    updateCollectionPluginV1(umi, {
      collection: collection.publicKey,
      plugin: createPlugin({
        type: 'Groups',
        data: { groups: [] },
      }),
    }).sendAndConfirm(umi),
    { name: 'InvalidPlugin' }
  );

  await t.throwsAsync(
    removeCollectionPluginV1(umi, {
      collection: collection.publicKey,
      pluginType: PluginType.Groups,
    }).sendAndConfirm(umi),
    { name: 'InvalidPlugin' }
  );

  await t.throwsAsync(
    approveCollectionPluginAuthorityV1(umi, {
      collection: collection.publicKey,
      pluginType: PluginType.Groups,
      newAuthority: addressPluginAuthority(delegate.publicKey),
    }).sendAndConfirm(umi),
    { name: 'InvalidPlugin' }
  );

  await t.throwsAsync(
    revokeCollectionPluginAuthorityV1(umi, {
      collection: collection.publicKey,
      pluginType: PluginType.Groups,
    }).sendAndConfirm(umi),
    { name: 'InvalidPlugin' }
  );
});
