import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  PluginType,
  addPluginV1,
  addressPluginAuthority,
  approvePluginAuthorityV1,
  createPlugin,
  pluginAuthorityPair,
  updateV1,
} from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createUmi,
} from '../../_setup';

test('it can prevent the asset from metadata updating', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Immutable',
        data: { whitelist: [], metadata: true },
      }),
    ],
  });

  const result = updateV1(umi, {
    asset: asset.publicKey,
    newName: 'bread',
    newUri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can update metadata if flag wasnt set', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Immutable',
        data: { whitelist: [], metadata: false },
      }),
    ],
  });

  await updateV1(umi, {
    asset: asset.publicKey,
    newName: 'bread',
    newUri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    name: 'bread',
    uri: 'https://example.com/bread',
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    immutable: {
      authority: {
        type: 'None',
      },
      whitelist: [],
      metadata: false,
    },
  });
});

test('it can prevent a plugin from being added/revoked', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Immutable',
        data: { whitelist: [], metadata: true },
      }),
    ],
  });

  const result = addPluginV1(umi, {
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

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can add/revoke a plugin if its whitelisted', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Immutable',
        data: { whitelist: [PluginType.FreezeDelegate], metadata: true },
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

test('it can add/revoke a plugin if its whitelisted and cannot if it isnt', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Immutable',
        data: { whitelist: [PluginType.FreezeDelegate], metadata: true },
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

  const result = addPluginV1(umi, {
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

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can add owner-managed plugins notwithstanding the whitelist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Immutable',
        data: { whitelist: [PluginType.TransferDelegate], metadata: true },
      }),
    ],
  });

  const result = addPluginV1(umi, {
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

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
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
