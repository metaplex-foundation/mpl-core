import test from 'ava';
import {
  pluginAuthorityPair,
  updatePluginAuthority,
  addCollectionPluginV1,
  createPlugin,
  removeCollectionPluginV1,
  PluginType,
} from '../../../src';
import {
  DEFAULT_COLLECTION,
  assertCollection,
  createCollection,
  createUmi,
} from '../../_setupRaw';

test('it can add permanentBurnDelegate to collection', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentBurnDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it cannot add permanentBurnDelegate to collection after creation', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi);

  const result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'PermanentBurnDelegate',
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentBurnDelegate: undefined,
  });
});

test('it can remove permanentBurnDelegate from collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentBurnDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  await removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.PermanentBurnDelegate,
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentBurnDelegate: undefined,
  });
});
