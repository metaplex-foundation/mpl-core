import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { fetchCollectionsByUpdateAuthority } from '../src';
import {
  createCollection,
  createRpc,
  generateSignerWithSol,
} from './_setup';

test('it can gpa fetch collections by updateAuthority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateAuthority = await generateKeyPairSigner();

  await createCollection(rpc, payer, {
    name: 'collection1',
    updateAuthority: updateAuthority.address,
  });
  await createCollection(rpc, payer, {
    name: 'collection2',
    updateAuthority: updateAuthority.address,
  });
  await createCollection(rpc, payer, {});

  const collections = await fetchCollectionsByUpdateAuthority(
    rpc,
    updateAuthority.address
  );
  const names = ['collection1', 'collection2'];

  t.is(collections.length, 2);
  t.assert(
    collections.every((collection) => names.includes(collection.data.name))
  );
  t.assert(
    collections.every(
      (collection) =>
        collection.data.updateAuthority === updateAuthority.address
    )
  );
});
