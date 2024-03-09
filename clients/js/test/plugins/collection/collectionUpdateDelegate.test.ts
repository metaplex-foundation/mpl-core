import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  PluginType,
  updateAuthority,
  approveCollectionPluginAuthority,
  plugin,
  authority,
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
    plugins: [plugin('UpdateDelegate', [{}])],
  });

  await approveCollectionPluginAuthority(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: authority('Pubkey', { address: updateDelegate.publicKey }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    plugins: {
      updateDelegate: {
        authority: {
          pubkey: [updateDelegate.publicKey],
        },
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
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  });
});
