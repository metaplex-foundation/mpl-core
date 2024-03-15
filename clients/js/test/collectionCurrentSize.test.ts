import test from 'ava';
import {
  burn,
  fetchCollection,
} from '../src';
import { createUmi, createCollection, createAsset } from './_setup';

test('it can burn an asset which is the part of a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const collection = await createCollection(umi, {});

  const asset1 = await createAsset(umi, {
    collection: collection.publicKey,
  });

  const asset2 = await createAsset(umi, {
    collection: collection.publicKey,
  });

  const collectionAfterMinting = await fetchCollection(umi, collection.publicKey);
  t.is(collectionAfterMinting.currentSize, 2);

  await burn(umi, {
    asset: asset1.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await burn(umi, {
    asset: asset2.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  const collectionAfterBurning = await fetchCollection(umi, collection.publicKey);
  t.is(collectionAfterBurning.currentSize, 0);
  
});
