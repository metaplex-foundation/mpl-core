import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  PluginType,
  approvePluginAuthorityV1,
  addPluginV1,
  addressPluginAuthority,
  pluginAuthorityPair,
  createPlugin,
  ownerPluginAuthority,
} from '../src';
import { DEFAULT_ASSET, assertAsset, createAsset, createUmi } from './_setup';

test('it can add an authority to a plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
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
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });
});

// TODO: This should fail
test('it cannot reassign authority of a plugin while already delegated', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const newDelegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

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

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: addressPluginAuthority(delegateAddress.publicKey),
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
      frozen: false,
    },
  });

  const result = approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: addressPluginAuthority(newDelegateAddress.publicKey),
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
      frozen: false,
    },
  });
});

test('it cannot reassign authority of a plugin as delegate while already delegated', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const newDelegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

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

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: addressPluginAuthority(delegateAddress.publicKey),
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
      frozen: false,
    },
  });

  const result = approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    authority: delegateAddress,
    newAuthority: addressPluginAuthority(newDelegateAddress.publicKey),
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
      frozen: false,
    },
  });
});

test('it cannot approve to reassign authority back to owner', async (t) => {
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
      frozen: false,
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
      frozen: false,
    },
  });
});
