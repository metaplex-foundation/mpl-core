import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  addPluginV1,
  approvePluginAuthorityV1,
  createPlugin,
  nonePluginAuthority,
  pluginAuthorityPair,
  PluginType,
  removePluginV1,
  revokePluginAuthorityV1,
  updatePluginV1,
} from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createCollection,
  createUmi,
} from '../../_setupRaw';

test('it can create asset with edition plugin', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });
});

test('it cannot add edition plugin after mint', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {});

  const res = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Edition',
      data: { number: 1 },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it cannot add edition plugin to collection', async (t) => {
  const umi = await createUmi();

  const res = createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
  });

  await t.throwsAsync(res, { name: 'InvalidPlugin' });
});
test('it cannot remove edition plugin', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
  });

  const res = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Edition,
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });
});

test('it can update edition plugin', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
  });

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Edition',
      data: { number: 2 },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 2,
    },
  });
});

test('it cannot update edition plugin as owner', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
    owner: owner.publicKey,
  });

  const res = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Edition',
      data: { number: 2 },
    }),
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });
});

test('it can create immutable edition plugin', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
        authority: nonePluginAuthority(),
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    edition: {
      authority: {
        type: 'None',
      },
      number: 1,
    },
  });

  const res = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Edition',
      data: { number: 2 },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'NoApprovals' });
});

test('it can make edition plugin immutable', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Edition,
    newAuthority: nonePluginAuthority(),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    edition: {
      authority: {
        type: 'None',
      },
      number: 1,
    },
  });

  const revokeRes = revokePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Edition,
  }).sendAndConfirm(umi);

  await t.throwsAsync(revokeRes, { name: 'InvalidAuthority' });

  const res = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Edition',
      data: { number: 2 },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'NoApprovals' });
});
