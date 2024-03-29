import test from 'ava';
import {
  addCollectionPluginV1,
  createPlugin,
  pluginAuthorityPair,
  updateCollectionPluginV1,
} from '../../../src';
import {
  DEFAULT_COLLECTION,
  assertCollection,
  createCollection,
  createUmi,
} from '../../_setup';

test('it can add permanent freeze to collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('it cannot add permanentFreeze to collection after creation', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi);

  const result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeDelegate',
      data: { frozen: true },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeDelegate: undefined,
  });
});

test('it can freeze and unfreeze a collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  await updateCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeDelegate',
      data: { frozen: false },
    }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});
