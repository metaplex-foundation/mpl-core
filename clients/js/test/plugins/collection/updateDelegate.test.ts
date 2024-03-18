import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  PluginType,
  addCollectionPluginV1,
  approveCollectionPluginAuthorityV1,
  createPlugin,
  pluginAuthorityPair,
  pubkeyPluginAuthority,
} from '../../../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createCollection,
  createUmi,
} from '../../_setup';

test('it can create a new asset with a collection if it is the collection update delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateDelegate = await generateSignerWithSol(umi);

  // When we create a new account.
  const collection = await createCollection(umi, {
    plugins: [pluginAuthorityPair({ type: 'UpdateDelegate' })],
  });

  await approveCollectionPluginAuthorityV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: pubkeyPluginAuthority(updateDelegate.publicKey),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'Pubkey',
        address: updateDelegate.publicKey,
      },
    },
  });

  umi.identity = updateDelegate;
  umi.payer = updateDelegate;
  const owner = generateSigner(umi);
  // When we create a new account.
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    owner,
    authority: updateDelegate,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('it can add updateDelegate to collection and then approve', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateDelegate = generateSigner(umi);

  const collection = await createCollection(umi);
  await addCollectionPluginV1(umi, {
    collection: collection.publicKey,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
  }).sendAndConfirm(umi);

  await approveCollectionPluginAuthorityV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: pubkeyPluginAuthority(updateDelegate.publicKey),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'Pubkey',
        address: updateDelegate.publicKey,
      },
    },
  });
});
