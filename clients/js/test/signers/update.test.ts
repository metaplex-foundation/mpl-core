import {
  generateSigner,
  none,
  sol,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import { createUmi } from '@metaplex-foundation/umi-bundle-tests';
import test from 'ava';
import { DEFAULT_ASSET, assertAsset, createAsset } from '../_setupRaw';
import { updateV1 } from '../../src';

test('it can update an asset as the update authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));

  const asset = await createAsset(umi, {
    owner: owner.publicKey,
    payer,
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });

  const instruction = updateV1(umi, {
    asset: asset.publicKey,
    payer,
    newName: 'Test Update',
    newUri: none(),
  }).getInstructions()[0];

  await transactionBuilder([
    {
      instruction,
      signers: [payer],
      bytesCreatedOnChain: 0,
    },
  ]).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    name: 'Test Update',
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });
});

test('it cannot update an asset if the payer does not sign', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));

  const asset = await createAsset(umi, {
    owner: owner.publicKey,
    payer,
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });

  const instruction = updateV1(umi, {
    asset: asset.publicKey,
    payer,
    newName: 'Test Update',
    newUri: none(),
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
    name: DEFAULT_ASSET.name,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });
});

test('it cannot update an asset if the authority is provided but does not sign', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  const authority = generateSigner(umi);
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));

  const asset = await createAsset(umi, {
    owner: owner.publicKey,
    payer,
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });

  const instruction = updateV1(umi, {
    asset: asset.publicKey,
    payer,
    authority,
    newName: 'Test Update',
    newUri: none(),
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
    name: DEFAULT_ASSET.name,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });
});

test('it cannot update an asset if the authority is provided and the payer does not sign', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  const authority = generateSigner(umi);
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));

  const asset = await createAsset(umi, {
    owner: owner.publicKey,
    payer,
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });

  const instruction = updateV1(umi, {
    asset: asset.publicKey,
    payer,
    authority,
    newName: 'Test Update',
    newUri: none(),
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
    name: DEFAULT_ASSET.name,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });
});
