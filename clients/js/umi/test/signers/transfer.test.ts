import {
  generateSigner,
  sol,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import { createUmi } from '@metaplex-foundation/umi-bundle-tests';
import test from 'ava';
import { assertAsset, createAsset } from '../_setupRaw';
import { transferV1 } from '../../src';

test('it can transfer an asset as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    payer,
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: payer.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });

  const instruction = transferV1(umi, {
    asset: asset.publicKey,
    payer,
    newOwner: newOwner.publicKey,
  }).getInstructions()[0];

  await transactionBuilder([
    {
      instruction,
      signers: [payer],
      bytesCreatedOnChain: 0,
    },
  ]).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });
});

test('it cannot transfer an asset if the owner does not sign', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    payer,
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: payer.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });

  const instruction = transferV1(umi, {
    asset: asset.publicKey,
    payer,
    newOwner: newOwner.publicKey,
  }).getInstructions()[0];

  instruction.keys[2].isSigner = false;

  const result = transactionBuilder([
    {
      instruction,
      signers: [],
      bytesCreatedOnChain: 0,
    },
  ]).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    message: /missing required signature for instruction/,
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: payer.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });
});

test('it cannot transfer an asset if the authority is provided but does not sign', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  const authority = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    authority,
    payer,
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: payer.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });

  const instruction = transferV1(umi, {
    asset: asset.publicKey,
    authority,
    payer,
    newOwner: newOwner.publicKey,
  }).getInstructions()[0];

  instruction.keys[3].isSigner = false;

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

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: payer.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });
});

test('it cannot transfer an asset if the authority is provided and the payer does not sign', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  const authority = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    authority,
    payer,
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: payer.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });

  const instruction = transferV1(umi, {
    asset: asset.publicKey,
    authority,
    payer,
    newOwner: newOwner.publicKey,
  }).getInstructions()[0];

  instruction.keys[2].isSigner = false;

  const result = transactionBuilder([
    {
      instruction,
      signers: [authority],
      bytesCreatedOnChain: 0,
    },
  ]).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    message: /missing required signature for instruction/,
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: payer.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });
});
