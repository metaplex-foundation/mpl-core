import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  PluginType,
  addPluginV1,
  approvePluginAuthorityV1,
  addressPluginAuthority,
  pluginAuthorityPair,
  revokePluginAuthorityV1,
  updatePluginV1,
  createPlugin,
  ownerPluginAuthority,
  removePluginV1,
  updatePluginAuthority,
} from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createCollection,
  createUmi,
} from '../../_setupRaw';

test('it can freeze and unfreeze an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it can delegate then freeze an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: addressPluginAuthority(delegateAddress.publicKey),
  }).sendAndConfirm(umi);

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
    authority: delegateAddress,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: true,
    },
  });
});

test('owner cannot undelegate a freeze plugin with a delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
        authority: addressPluginAuthority(delegateAddress.publicKey),
      }),
    ],
  });

  const result = revokePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('owner cannot approve to reassign authority back to owner if frozen', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
        authority: addressPluginAuthority(delegateAddress.publicKey),
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: true,
    },
  });

  const result = approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: ownerPluginAuthority(),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'CannotRedelegate' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: true,
    },
  });
});

test('it cannot add multiple freeze plugins to an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'PluginAlreadyExists' });
});

test('it cannot remove freeze plugin if update authority and frozen', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it delegate cannot freeze after delegate has been revoked', async (t) => {
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: addressPluginAuthority(delegateAddress.publicKey),
      }),
    ],
  });

  await revokePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
  }).sendAndConfirm(umi);

  const result = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
    authority: delegateAddress,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it owner cannot unfreeze frozen asset', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
        authority: addressPluginAuthority(delegateAddress.publicKey),
      }),
    ],
  });

  const result = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: true,
    },
  });
});

test('it update authority cannot unfreeze frozen asset', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const updateAuthority = generateSigner(umi);
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    updateAuthority,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
        authority: addressPluginAuthority(delegateAddress.publicKey),
      }),
    ],
  });

  const result = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
    authority: updateAuthority,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuthority.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: true,
    },
  });
});

test('a freezeDelegate can freeze using delegated update authority', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const updateAuthority = generateSigner(umi);

  const asset = await createAsset(umi, {
    updateAuthority: updateAuthority.publicKey,
    owner: owner.publicKey,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuthority.publicKey },
    freezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
    authority: updateAuthority,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuthority.publicKey },
    freezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('a freezeDelegate can freeze using delegated update authority from collection', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const updateAuthority = generateSigner(umi);

  const collection = await createCollection(umi, {
    updateAuthority: updateAuthority.publicKey,
  });

  const asset = await createAsset(umi, {
    owner: owner.publicKey,
    collection: collection.publicKey,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: updatePluginAuthority(),
      }),
    ],
    authority: updateAuthority,
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    freezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
    authority: updateAuthority,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    freezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});
