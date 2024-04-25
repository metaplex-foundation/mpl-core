import {
  generateSigner,
  sol,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import { createUmi } from '@metaplex-foundation/umi-bundle-tests';
import test from 'ava';
import { DEFAULT_COLLECTION, assertCollection } from '../_setupRaw';
import { createCollectionV1 } from '../../src';

test('it can create a new asset', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));
  const collectionAddress = generateSigner(umi);

  const instruction = createCollectionV1(umi, {
    collection: collectionAddress,
    payer,
    ...DEFAULT_COLLECTION,
  }).getInstructions()[0];

  await transactionBuilder([
    {
      instruction,
      signers: [collectionAddress, payer],
      bytesCreatedOnChain: 0,
    },
  ]).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collectionAddress.publicKey,
    updateAuthority: payer.publicKey,
  });
});

test('it cannot create a new collection if the collection does not sign', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));
  const collectionAddress = generateSigner(umi);

  const instruction = createCollectionV1(umi, {
    collection: collectionAddress,
    payer,
    ...DEFAULT_COLLECTION,
  }).getInstructions()[0];

  instruction.keys[0].isSigner = false;

  const result = transactionBuilder([
    {
      instruction,
      signers: [payer],
      bytesCreatedOnChain: 0,
    },
  ]).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    message: /missing required signature for instruction/,
  });
});

test('it cannot create a new collection if the payer does not sign', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));
  const collectionAddress = generateSigner(umi);

  const instruction = createCollectionV1(umi, {
    collection: collectionAddress,
    payer,
    ...DEFAULT_COLLECTION,
  }).getInstructions()[0];

  instruction.keys[2].isSigner = false;

  const result = transactionBuilder([
    {
      instruction,
      signers: [collectionAddress],
      bytesCreatedOnChain: 0,
    },
  ]).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    message: /missing required signature for instruction/,
  });
});
