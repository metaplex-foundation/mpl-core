import { generateSigner, sol } from '@metaplex-foundation/umi';
import test from 'ava';

import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import { burnCollection } from '../src';
import {
  DEFAULT_COLLECTION,
  assertBurned,
  assertCollection,
  createAssetWithCollection,
  createCollection,
  createUmi,
} from './_setupRaw';

test('it can burn a collection as the authority', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  await burnCollection(umi, {
    collection: collection.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  // And the asset address still exists but was resized to 1.
  const afterCollection = await assertBurned(t, umi, collection.publicKey);
  t.deepEqual(afterCollection.lamports, sol(0.00089784));
});

test('it cannot burn a collection if not the authority', async (t) => {
  const umi = await createUmi();
  const attacker = generateSigner(umi);

  const collection = await createCollection(umi);
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const result = burnCollection(umi, {
    collection: collection.publicKey,
    authority: attacker,
    compressionProof: null,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});

test('it cannot burn a collection if it has Assets in it', async (t) => {
  const umi = await createUmi();

  const { collection } = await createAssetWithCollection(umi, {});

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const result = burnCollection(umi, {
    collection: collection.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'CollectionMustBeEmpty' });
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});

test('it can burn asset with different payer', async (t) => {
  const umi = await createUmi();
  const authority = await generateSignerWithSol(umi);
  const collection = await createCollection(umi, {
    updateAuthority: authority.publicKey,
  });
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: authority.publicKey,
  });

  const lamportsBefore = await umi.rpc.getBalance(umi.identity.publicKey);

  await burnCollection(umi, {
    collection: collection.publicKey,
    payer: umi.identity,
    authority,
    compressionProof: null,
  }).sendAndConfirm(umi);

  // And the asset address still exists but was resized to 1.
  const afterCollection = await assertBurned(t, umi, collection.publicKey);
  t.deepEqual(afterCollection.lamports, sol(0.00089784));

  const lamportsAfter = await umi.rpc.getBalance(umi.identity.publicKey);

  t.true(lamportsAfter.basisPoints > lamportsBefore.basisPoints);
});

test('it cannot use an invalid noop program for collections', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);
  const fakeLogWrapper = generateSigner(umi);
  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const result = burnCollection(umi, {
    collection: collection.publicKey,
    logWrapper: fakeLogWrapper.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});
