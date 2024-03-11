import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  PluginType,
  approveCollectionPluginAuthority,
  authority,
  plugin,
  updateAuthority,
} from '../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createAssetWithCollection,
  createCollection,
  createUmi,
} from './_setup';

test('it can create a new collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);

  await createCollection(umi, {
    collection: collectionAddress,
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collectionAddress,
    updateAuthority: umi.identity.publicKey,
  });
});

test('it can create a new collection with plugins', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const collection = await createCollection(umi, {
    plugins: [{ plugin: plugin('Freeze', [{ frozen: false }]), authority: null }],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    freeze: {
      authority: {
        owner: true,
      },
      frozen: false,
    },
  });
});

test('it can create a new asset with a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [{ plugin: plugin('UpdateDelegate', [{}]), authority: null }],
    }
  );

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        update: true,
      },
    },
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  });
});

test('it can create a new asset with a collection with collection delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [{ plugin: plugin('UpdateDelegate', [{}]), authority: null }],
  });

  await approveCollectionPluginAuthority(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: authority('Pubkey', { address: delegate.publicKey }),
  }).sendAndConfirm(umi);

  const umi2 = await createUmi(); // guarantee a new signer
  const asset = await createAsset(umi2, {
    collection: collection.publicKey,
    authority: delegate,
  })

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi2.identity.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  });
});

// TODO: Add test
test('it cannot create a new asset with an update authority that is not the collection', async (t) => {
  t.pass();
});

test('it cannot create a new asset with a collection if it is not the collection auth', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAuth = generateSigner(umi);

  const collection = await createCollection(umi, {
    updateAuthority: collectionAuth.publicKey,
  });

  const result = createAsset(umi, {
    collection: collection.publicKey,
  })

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});
