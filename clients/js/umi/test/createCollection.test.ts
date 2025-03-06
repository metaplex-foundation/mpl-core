import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  PluginType,
  approveCollectionPluginAuthorityV1,
  pluginAuthorityPair,
  addressPluginAuthority,
  createCollectionV1,
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
} from './_setupRaw';

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
  const umi = await createUmi();

  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: false },
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
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
        }),
      ],
    }
  );

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('it can create a new asset with a collection with collection delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
      }),
    ],
  });

  await approveCollectionPluginAuthorityV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: addressPluginAuthority(delegate.publicKey),
  }).sendAndConfirm(umi);

  const umi2 = await createUmi(); // guarantee a new signer
  const asset = await createAsset(umi2, {
    collection: collection.publicKey,
    authority: delegate,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi2.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
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
  });

  await t.throwsAsync(result, { name: 'NoApprovals' });
});

test('it cannot create a collection with an owner managed plugin', async (t) => {
  const umi = await createUmi();

  const result = createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: {
          frozen: false,
        },
      }),
    ],
  });

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it cannot use an invalid system program', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAddress = generateSigner(umi);
  const fakeSystemProgram = generateSigner(umi);

  const result = createCollectionV1(umi, {
    collection: collectionAddress,
    name: 'Test',
    uri: 'Test',
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});
