import test from 'ava';

import { addPluginV1, createPlugin, pluginAuthorityPair } from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createUmi,
} from '../../_setup';

test('it cannot add UA-managed plugin if addBlocker had been added on creation', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'AddBlocker',
      }),
    ],
  });

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeDelegate',
      data: { frozen: true },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can add plugins unless AddBlocker is added', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Attributes',
      data: {
        attributeList: [],
      },
    }),
  }).sendAndConfirm(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'AddBlocker',
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [],
    },
    addBlocker: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeDelegate',
      data: {
        frozen: false,
      },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can add owner-managed plugins even if AddBlocker had been added', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  //   const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'AddBlocker',
      }),
    ],
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
    addBlocker: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});
