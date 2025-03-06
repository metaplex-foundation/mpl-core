import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { Key, getAssetV1GpaBuilder, updateAuthority } from '../src';
import { createAsset, createCollection, createUmi } from './_setupRaw';

test('it can gpa fetch assets by owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);
  await createAsset(umi, {
    name: 'asset1',
    owner: owner.publicKey,
  });
  await createAsset(umi, {
    name: 'asset2',
    owner: owner.publicKey,
  });
  await createAsset(umi, {});

  const assets = await getAssetV1GpaBuilder(umi)
    .whereField('owner', owner.publicKey)
    .whereField('key', Key.AssetV1)
    .getDeserialized();
  const names = ['asset1', 'asset2'];

  t.is(assets.length, 2);
  t.assert(assets.every((asset) => names.includes(asset.name)));
  t.assert(assets.every((asset) => asset.owner === owner.publicKey));
});

test('it can gpa fetch assets by collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collection = await createCollection(umi);
  await createAsset(umi, {
    name: 'asset1',
    collection: collection.publicKey,
  });
  await createAsset(umi, {
    name: 'asset2',
    collection: collection.publicKey,
  });
  await createAsset(umi, {});

  const assets = await getAssetV1GpaBuilder(umi)
    .whereField(
      'updateAuthority',
      updateAuthority('Collection', [collection.publicKey])
    )
    .whereField('key', Key.AssetV1)
    .getDeserialized();
  const names = ['asset1', 'asset2'];

  t.is(assets.length, 2);
  t.assert(assets.every((asset) => names.includes(asset.name)));
  assets.forEach((asset) => {
    t.deepEqual(asset.updateAuthority, {
      type: 'Collection',
      address: collection.publicKey,
    });
  });
});

test('it can gpa fetch assets by updateAuthority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const authority = generateSigner(umi);
  await createAsset(umi, {
    name: 'asset1',
    updateAuthority: authority,
  });
  await createAsset(umi, {
    name: 'asset2',
    updateAuthority: authority,
  });
  await createAsset(umi, {});

  const assets = await getAssetV1GpaBuilder(umi)
    .whereField(
      'updateAuthority',
      updateAuthority('Address', [authority.publicKey])
    )
    .whereField('key', Key.AssetV1)
    .getDeserialized();
  const names = ['asset1', 'asset2'];

  t.is(assets.length, 2);
  t.assert(assets.every((asset) => names.includes(asset.name)));
  assets.forEach((asset) => {
    t.deepEqual(asset.updateAuthority, {
      type: 'Address',
      address: authority.publicKey,
    });
  });
});
