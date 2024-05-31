import test from 'ava';

import {
  pluginAuthorityPair,
  updatePluginAuthority,
  createPlugin,
  addPluginV1,
  addCollectionPluginV1,
} from '../../../src';
import {
  DEFAULT_COLLECTION,
  assertCollection,
  createAsset,
  createCollection,
  createUmi,
} from '../../_setupRaw';

test('it can add masterEdition to collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);

  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'MasterEdition',
      data: {
        maxSupply: 100,
        name: 'name',
        uri: 'uri',
      },
    }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    masterEdition: {
      authority: {
        type: 'UpdateAuthority',
      },
      maxSupply: 100,
      name: 'name',
      uri: 'uri',
    },
  });
});

test('it can create collection with masterEdition', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'MasterEdition',
        data: {
          maxSupply: 100,
          name: 'name',
          uri: 'uri',
        },
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    masterEdition: {
      authority: {
        type: 'UpdateAuthority',
      },
      maxSupply: 100,
      name: 'name',
      uri: 'uri',
    },
  });
});

test('it can create master edition with default values', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'MasterEdition',
        data: {
          maxSupply: null,
          name: null,
          uri: null,
        },
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    masterEdition: {
      authority: {
        type: 'UpdateAuthority',
      },
      maxSupply: undefined,
      name: undefined,
      uri: undefined,
    },
  });
});

test('it cannot add masterEdition to asset', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi);

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'MasterEdition',
      data: {
        maxSupply: 100,
        name: 'name',
        uri: 'uri',
      },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidPlugin',
  });
});

test('it cannot create asset with masterEdition', async (t) => {
  const umi = await createUmi();

  const result = createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'MasterEdition',
        data: {
          maxSupply: 100,
          name: 'name',
          uri: 'uri',
        },
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await t.throwsAsync(result, {
    name: 'InvalidPlugin',
  });
});
