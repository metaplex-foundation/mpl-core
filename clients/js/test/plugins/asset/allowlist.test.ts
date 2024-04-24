import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  PluginType,
  addPluginV1,
  addressPluginAuthority,
  approvePluginAuthorityV1,
  createPlugin,
  fetchAssetV1,
  pluginAuthority,
  pluginAuthorityPair,
  removePluginV1,
  updatePluginV1,
} from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createUmi,
} from '../../_setup';

test('it can add a plugin if its allowlisted', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Allowlist',
        data: { plugins: [PluginType.FreezeDelegate], mustBeEmpty: false },
      }),
    ],
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  })
    .append(
      approvePluginAuthorityV1(umi, {
        asset: asset.publicKey,
        pluginType: PluginType.FreezeDelegate,
        newAuthority: addressPluginAuthority(delegateAddress.publicKey),
      })
    )
    .sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });
});

test('it can revoke a plugin if its allowlisted', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Allowlist',
        data: { plugins: [PluginType.FreezeDelegate], mustBeEmpty: false },
      }),
    ],
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  })
    .append(
      approvePluginAuthorityV1(umi, {
        asset: asset.publicKey,
        pluginType: PluginType.FreezeDelegate,
        newAuthority: addressPluginAuthority(delegateAddress.publicKey),
      })
    )
    .sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });

  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    authority: umi.identity,
    payer: umi.identity,
  }).sendAndConfirm(umi);

  const asset2 = await fetchAssetV1(umi, asset.publicKey);

  t.is(asset2.freezeDelegate, undefined);
});

test('it cannot add a plugin if its not allowlisted and not owner-managed', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Allowlist',
        data: { plugins: [], mustBeEmpty: true },
      }),
    ],
  });

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'PermanentBurnDelegate' }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it shows that owner-manager plugins cannot be added to allowlist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Allowlist',
        data: { plugins: [], mustBeEmpty: false },
      }),
    ],
  });

  const result = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'TransferDelegate' }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can add owner-managed plugins notwithstanding the allowlist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  let asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Allowlist',
        data: { plugins: [], mustBeEmpty: true },
      }),
    ],
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'TransferDelegate' }),
  })
    .append(
      approvePluginAuthorityV1(umi, {
        asset: asset.publicKey,
        pluginType: PluginType.TransferDelegate,
        newAuthority: addressPluginAuthority(delegateAddress.publicKey),
      })
    )
    .sendAndConfirm(umi);

  asset = await fetchAssetV1(umi, asset.publicKey);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    transferDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
    },
  });
});

test('it can revoke owner-managed plugins notwithstanding the allowlist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Allowlist',
        data: { plugins: [], mustBeEmpty: true },
      }),
    ],
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'TransferDelegate' }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.TransferDelegate,
    authority: umi.identity,
    payer: umi.identity,
  }).sendAndConfirm(umi);

  const asset2 = await fetchAssetV1(umi, asset.publicKey);

  t.is(asset2.transferDelegate, undefined);
});

test('plugin is NOT updatable if its authority is None', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  let asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Allowlist',
        data: { plugins: [], mustBeEmpty: false },
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    allowlist: {
      authority: {
        type: 'None',
      },
      plugins: [],
      mustBeEmpty: false,
    },
  });

  const result = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Allowlist',
      data: { plugins: [PluginType.PermanentBurnDelegate], mustBeEmpty: false },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('plugin IS updatable if its authority IS NOT None', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  let asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Allowlist',
        data: { plugins: [], mustBeEmpty: false },
        authority: pluginAuthority('UpdateAuthority'),
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    allowlist: {
      authority: {
        type: 'UpdateAuthority',
      },
      plugins: [],
      mustBeEmpty: false,
    },
  });

  // authority is UA
  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Allowlist',
      data: { plugins: [PluginType.PermanentBurnDelegate], mustBeEmpty: false },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    allowlist: {
      authority: {
        type: 'UpdateAuthority',
      },
      plugins: [PluginType.PermanentBurnDelegate],
      mustBeEmpty: false,
    },
  });

  asset = await fetchAssetV1(umi, asset.publicKey);
  t.is(asset.allowlist?.plugins[0], PluginType.PermanentBurnDelegate);
});
