import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  addCollectionPluginV1,
  createPlugin,
  pluginAuthorityPair,
  updatePluginAuthority,
  removeCollectionPluginV1,
  PluginType,
} from '../../../src';
import {
  assertCollection,
  createCollection,
  createUmi,
  DEFAULT_COLLECTION,
} from '../../_setupRaw';

test('it can add permanentTransferDelegate to collection', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it cannot add PermanentTransferDelegate to collection after creation', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const collection = await createCollection(umi);

  const result = addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({ type: 'PermanentTransferDelegate' }),
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentTransferDelegate: undefined,
  });
});

test('it can remove PermanentTransferDelegate from collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  await removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.PermanentTransferDelegate,
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentTransferDelegate: undefined,
  });
});
