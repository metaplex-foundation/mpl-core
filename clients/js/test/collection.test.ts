import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { Key, getCollectionV1GpaBuilder } from '../src';
import { createUmi, createCollection } from './_setupRaw';

test('it can gpa fetch collections by updateAuthority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateAuthority = generateSigner(umi);
  await createCollection(umi, {
    name: 'collection1',
    updateAuthority: updateAuthority.publicKey,
  });
  await createCollection(umi, {
    name: 'collection2',
    updateAuthority: updateAuthority.publicKey,
  });
  await createCollection(umi, {});

  const collections = await getCollectionV1GpaBuilder(umi)
    .whereField('updateAuthority', updateAuthority.publicKey)
    .whereField('key', Key.CollectionV1)
    .getDeserialized();
  const names = ['collection1', 'collection2'];

  t.is(collections.length, 2);
  t.assert(
    collections.every((collectionV1) => names.includes(collectionV1.name))
  );
  t.assert(
    collections.every(
      (collectionV1) =>
        collectionV1.updateAuthority === updateAuthority.publicKey
    )
  );
});
