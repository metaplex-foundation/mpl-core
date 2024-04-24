import test from 'ava';
import { burnV1, fetchCollectionV1 } from '../src';
import { createUmi, createCollection, createAsset } from './_setupRaw';

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

  const collectionAfterMinting = await fetchCollectionV1(
    umi,
    collection.publicKey
  );
  t.is(collectionAfterMinting.currentSize, 2);
  t.is(collectionAfterMinting.numMinted, 2);

  await burnV1(umi, {
    asset: asset1.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await burnV1(umi, {
    asset: asset2.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  const collectionAfterBurning = await fetchCollectionV1(
    umi,
    collection.publicKey
  );
  t.is(collectionAfterBurning.currentSize, 0);
  t.is(collectionAfterMinting.numMinted, 2);
});
