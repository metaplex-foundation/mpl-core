import {
  generateSigner,
  sol,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import { createUmi } from '@metaplex-foundation/umi-bundle-tests';
import test from 'ava';
import { DEFAULT_ASSET, assertAsset } from '../_setupRaw';
import { createV1 } from '../../src';

test('it can create a new asset', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));
  const assetAddress = generateSigner(umi);

  const instruction = createV1(umi, {
    asset: assetAddress,
    payer,
    ...DEFAULT_ASSET,
  }).getInstructions()[0];

  await transactionBuilder([
    {
      instruction,
      signers: [assetAddress, payer],
      bytesCreatedOnChain: 0,
    },
  ]).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: assetAddress.publicKey,
    owner: payer.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });
});

test('it cannot create a new asset if the asset does not sign', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));
  const assetAddress = generateSigner(umi);

  const instruction = createV1(umi, {
    asset: assetAddress,
    payer,
    ...DEFAULT_ASSET,
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

test('it cannot create a new asset if the payer does not sign', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));
  const assetAddress = generateSigner(umi);

  const instruction = createV1(umi, {
    asset: assetAddress,
    payer,
    ...DEFAULT_ASSET,
  }).getInstructions()[0];

  instruction.keys[3].isSigner = false;

  const result = transactionBuilder([
    {
      instruction,
      signers: [assetAddress],
      bytesCreatedOnChain: 0,
    },
  ]).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    message: /missing required signature for instruction/,
  });
});

test('it fails if an authority is provided and it does not sign', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  const authority = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));
  const assetAddress = generateSigner(umi);

  const instruction = createV1(umi, {
    asset: assetAddress,
    authority,
    payer,
    ...DEFAULT_ASSET,
  }).getInstructions()[0];

  instruction.keys[2].isSigner = false;

  const result = transactionBuilder([
    {
      instruction,
      signers: [assetAddress, payer],
      bytesCreatedOnChain: 0,
    },
  ]).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    message: /missing required signature for instruction/,
  });
});

test('it fails even if an authority signs but the payer does not sign', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  const authority = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));
  const assetAddress = generateSigner(umi);

  const instruction = createV1(umi, {
    asset: assetAddress,
    authority,
    payer,
    ...DEFAULT_ASSET,
  }).getInstructions()[0];

  instruction.keys[3].isSigner = false;

  const result = transactionBuilder([
    {
      instruction,
      signers: [assetAddress, authority],
      bytesCreatedOnChain: 0,
    },
  ]).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    message: /missing required signature for instruction/,
  });
});
