import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';

import {
  addCollectionPlugin,
  addPlugin,
  authority,
  plugin,
  ruleSet,
  updateAuthority,
} from '../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createCollection,
  createUmi,
} from './_setup';

test('it can add a plugin to an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {});

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    freeze: {
      authority: {
        owner: true,
      },
      frozen: false,
    },
  });
});

test('it can add a plugin to an asset with a different authority than the default', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {});

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
    initAuthority: authority('Pubkey', { address: delegateAddress.publicKey }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    freeze: {
      authority: {
        pubkey: [delegateAddress.publicKey],
      },
      frozen: false,
    },
  });
});

test('it can add a plugin to a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const collection = await createCollection(umi, {});

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: plugin('Royalties', [
      {
        percentage: 5,
        creators: [],
        ruleSet: ruleSet('None'),
      },
    ]),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    royalties: {
      authority: {
        update: true,
      },
      percentage: 5,
      creators: [],
      ruleSet: ruleSet('None'),
    },
  });
});

test('it cannot add an owner-managed plugin to a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const collection = await createCollection(umi, {});

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const result = addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});
