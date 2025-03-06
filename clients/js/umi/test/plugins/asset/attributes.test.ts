import test from 'ava';
import {
  createPlugin,
  pluginAuthorityPair,
  updatePluginV1,
} from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createUmi,
} from '../../_setupRaw';

test('it can add attributes to an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'Attributes', data: { attributeList: [] } }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [],
    },
  });

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Attributes',
      data: {
        attributeList: [
          { key: 'key0', value: 'value0' },
          { key: 'key1', value: 'value1' },
        ],
      },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        { key: 'key0', value: 'value0' },
        { key: 'key1', value: 'value1' },
      ],
    },
  });
});

test('it can remove attributes to an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: {
          attributeList: [
            { key: 'key0', value: 'value0' },
            { key: 'key1', value: 'value1' },
          ],
        },
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        { key: 'key0', value: 'value0' },
        { key: 'key1', value: 'value1' },
      ],
    },
  });

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Attributes',
      data: { attributeList: [{ key: 'key0', value: 'value0' }] },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key0', value: 'value0' }],
    },
  });
});

test('it can add then remove attributes to an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'Attributes', data: { attributeList: [] } }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [],
    },
  });

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Attributes',
      data: {
        attributeList: [
          { key: 'key0', value: 'value0' },
          { key: 'key1', value: 'value1' },
        ],
      },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        { key: 'key0', value: 'value0' },
        { key: 'key1', value: 'value1' },
      ],
    },
  });

  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'Attributes', data: { attributeList: [] } }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [],
    },
  });
});
