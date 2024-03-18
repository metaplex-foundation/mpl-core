import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  PluginType,
  approvePluginAuthorityV1,
  addPluginV1,
  pubkeyPluginAuthority,
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
        newAuthority: pubkeyPluginAuthority(delegateAddress.publicKey),
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
        type: 'Pubkey',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });
});

test('it can reassign authority of a plugin to another pubkey', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const newDelegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } })],
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
    newAuthority: pubkeyPluginAuthority(delegateAddress.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Pubkey',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: pubkeyPluginAuthority(newDelegateAddress.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Pubkey',
        address: newDelegateAddress.publicKey,
      },
      frozen: false,
    },
  });
});

test('it can approve to reassign authority back to owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } })],
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: pubkeyPluginAuthority(delegateAddress.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Pubkey',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: ownerPluginAuthority(),
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
