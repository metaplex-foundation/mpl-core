import test from 'ava';

import { generateSigner } from '@metaplex-foundation/umi';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createUmi,
} from '../../_setupRaw';
import { createAsset, createCollection } from '../../_setupSdk';
import {
  addCollectionPlugin,
  addPlugin,
  updateCollectionPlugin,
  updatePlugin,
} from '../../../src';

test('it can create collection with verified creators plugin', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [],
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [],
    },
  });
});

test('it cannot create collection with verified creators plugin and unauthorized signature', async (t) => {
  const umi = await createUmi();
  const creator = generateSigner(umi);

  const res = createCollection(umi, {
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: true,
          },
        ],
      },
    ],
  });

  await t.throwsAsync(res, { name: 'MissingSigner' });
});

test('it can create collection with verified creators plugin with authorized signature', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: umi.identity.publicKey,
            verified: true,
          },
        ],
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: true,
        },
      ],
    },
  });
});

test('it cannot add verified creators plugin to collection with unauthorized signature', async (t) => {
  const umi = await createUmi();
  const creator = generateSigner(umi);

  const collection = await createCollection(umi);

  const res = addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'MissingSigner' });
});

test('it can verify a creator on a collection verified creators plugin', async (t) => {
  const umi = await createUmi();
  const creator = await generateSignerWithSol(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  await updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
    authority: creator,
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
  });
});

test('it cannot verify a creator on a collection verified creators plugin with unauthorized signature', async (t) => {
  const umi = await createUmi();
  const creator = generateSigner(umi);
  const unauthed = await generateSignerWithSol(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  const res = updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
    authority: unauthed,
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'MissingSigner' });
});

test('it cannot unverify a collection creator with update auth', async (t) => {
  const umi = await createUmi();
  const creator = await generateSignerWithSol(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  await updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
    authority: creator,
  }).sendAndConfirm(umi);

  const res = updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: creator.publicKey,
          verified: false,
        },
      ],
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'InvalidPluginOperation' });
});

test('it can create an asset in a collection with a verified creator that is not the signer', async (t) => {
  const umi = await createUmi();
  const creator = await generateSignerWithSol(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  // The creator verifies themselves on the collection, so the collection now carries a
  // verified signature that does not belong to the update authority.
  await updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
    authority: creator,
  }).sendAndConfirm(umi);

  // The update authority can still mint into the collection without the creator signing.
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    numMinted: 1,
    currentSize: 1,
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
  });
});

test('it can print an edition into a master edition collection with verified creators plugin as an update delegate', async (t) => {
  const umi = await createUmi();
  const creator = await generateSignerWithSol(umi);
  const printer = await generateSignerWithSol(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'MasterEdition',
        maxSupply: 100,
        name: 'name',
        uri: 'uri',
      },
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: umi.identity.publicKey,
            verified: true,
          },
          {
            address: creator.publicKey,
            verified: false,
          },
        ],
      },
      {
        type: 'UpdateDelegate',
        additionalDelegates: [printer.publicKey],
      },
    ],
  });

  await updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: true,
        },
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
    authority: creator,
  }).sendAndConfirm(umi);

  // A delegate (e.g. a printing program) that is not one of the verified creators can
  // still print editions into the collection.
  const asset = await createAsset(umi, {
    collection: collection.publicKey,
    payer: printer,
    authority: printer,
    plugins: [
      {
        type: 'Edition',
        number: 1,
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: printer.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    numMinted: 1,
    currentSize: 1,
    masterEdition: {
      authority: {
        type: 'UpdateAuthority',
      },
      maxSupply: 100,
      name: 'name',
      uri: 'uri',
    },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: true,
        },
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
  });
});

test('it can add and update verified creators plugin on an asset in a collection with verified creators plugin', async (t) => {
  const umi = await createUmi();
  const collectionCreator = await generateSignerWithSol(umi);
  const assetCreator = await generateSignerWithSol(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: collectionCreator.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  await updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: collectionCreator.publicKey,
          verified: true,
        },
      ],
    },
    authority: collectionCreator,
  }).sendAndConfirm(umi);

  const asset = await createAsset(umi, {
    collection: collection.publicKey,
  });

  // The asset gets its own creator list, unrelated to the collection's.
  await addPlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: true,
        },
        {
          address: assetCreator.publicKey,
          verified: false,
        },
      ],
    },
  }).sendAndConfirm(umi);

  await updatePlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: true,
        },
        {
          address: assetCreator.publicKey,
          verified: true,
        },
      ],
    },
    authority: assetCreator,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: true,
        },
        {
          address: assetCreator.publicKey,
          verified: true,
        },
      ],
    },
  });

  // The collection's own list is untouched.
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    numMinted: 1,
    currentSize: 1,
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: collectionCreator.publicKey,
          verified: true,
        },
      ],
    },
  });
});

test('it still rejects unauthorized verified signatures on an asset in a collection with verified creators plugin', async (t) => {
  const umi = await createUmi();
  const creator = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: umi.identity.publicKey,
            verified: true,
          },
        ],
      },
    ],
  });

  const res = createAsset(umi, {
    collection: collection.publicKey,
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: true,
          },
        ],
      },
    ],
  });

  await t.throwsAsync(res, { name: 'MissingSigner' });
});
