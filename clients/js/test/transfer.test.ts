import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';

import { transferV1 } from '../src';
import {
  assertAsset,
  createAsset,
  createAssetWithCollection,
  createUmi,
} from './_setup';

test('it can transfer an asset as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi);
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it cannot transfer an asset if not the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);
  const attacker = generateSigner(umi);

  const asset = await createAsset(umi);

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it cannot transfer asset in collection if no collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {});

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'MissingCollection' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('it can transfer asset in collection as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {});

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('it cannot transfer asset in collection if collection not included', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {});

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'MissingCollection' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});
