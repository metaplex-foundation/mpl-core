import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { getCollectionGpaBuilder } from '../src';
import { createUmi, createCollection } from './_setup';

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

  const collections = await getCollectionGpaBuilder(umi)
    .whereField('updateAuthority', updateAuthority.publicKey)
    .getDeserialized();
  const names = ['collection1', 'collection2'];

  t.is(collections.length, 2);
  t.assert(collections.every((collection) => names.includes(collection.name)));
  t.assert(
    collections.every(
      (collection) => collection.updateAuthority === updateAuthority.publicKey
    )
  );
});
