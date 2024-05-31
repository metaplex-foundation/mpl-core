import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  createPlugin,
  pluginAuthorityPair,
  addPluginV1,
  updatePluginV1,
  updateV1,
  addressPluginAuthority,
  revokePluginAuthorityV1,
  PluginType,
} from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createUmi,
} from '../../_setupRaw';

test('it can create an asset with updateDelegate', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });
});

test('it can create an asset with updateDelegate with additional delegates', async (t) => {
  const umi = await createUmi();
  const updateDelegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.publicKey] },
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });
});

test('it can add updateDelegate to asset with additional delegates', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const updateDelegate = generateSigner(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate.publicKey] },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });
});

test('it can update updateDelegate on asset with additional delegates', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const updateDelegate = generateSigner(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate.publicKey] },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
  });
});

test('an updateDelegate can update an asset', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    name: 'short',
    uri: 'https://short.com',
  });
  const updateDelegate = generateSigner(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
    }),
    initAuthority: addressPluginAuthority(updateDelegate.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  await updateV1(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('an updateDelegate additionalDelegate can update an asset', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    name: 'short',
    uri: 'https://short.com',
  });
  const updateDelegate = generateSigner(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate.publicKey] },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  await updateV1(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('an updateDelegate cannot update an asset after delegate authority revoked', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    name: 'short',
    uri: 'https://short.com',
  });
  const updateDelegate = generateSigner(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
    }),
    initAuthority: addressPluginAuthority(updateDelegate.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.publicKey,
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  await revokePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.UpdateDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  const result = updateV1(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });
});

test('an updateDelegate additionalDelegate cannot update an asset after delegate authority revoked', async (t) => {
  const umi = await createUmi();
  const { identity } = umi;
  const asset = await createAsset(umi, {
    name: 'short',
    uri: 'https://short.com',
  });
  const updateDelegate = generateSigner(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate.publicKey] },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.publicKey],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    authority: identity,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [] },
    })
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  const result = updateV1(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    authority: updateDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });
});
