import {
  assertAccountExists,
  generateSigner,
  sol,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import { createUmi } from '@metaplex-foundation/umi-bundle-tests';
import test from 'ava';
import { assertAsset, createAsset } from '../_setupRaw';
import { Key, burnV1 } from '../../src';

test('it can burn an asset as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));

  const asset = await createAsset(umi, {
    payer,
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: payer.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });

  const instruction = burnV1(umi, {
    asset: asset.publicKey,
    payer,
  }).getInstructions()[0];

  await transactionBuilder([
    {
      instruction,
      signers: [payer],
      bytesCreatedOnChain: 0,
    },
  ]).sendAndConfirm(umi);

  // And the asset address still exists but was resized to 1.
  const afterAsset = await umi.rpc.getAccount(asset.publicKey);
  t.true(afterAsset.exists);
  assertAccountExists(afterAsset);
  t.deepEqual(afterAsset.lamports, sol(0.00089784 + 0.0015));
  t.is(afterAsset.data.length, 1);
  t.is(afterAsset.data[0], Key.Uninitialized);
});

test('it cannot burn an asset if the owner does not sign', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));

  const asset = await createAsset(umi, {
    payer,
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: payer.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });

  const instruction = burnV1(umi, {
    asset: asset.publicKey,
    payer,
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

test('it cannot burn an asset if an authority is provided but does not sign', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  const authority = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));

  const asset = await createAsset(umi, {
    payer,
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: payer.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });

  const instruction = burnV1(umi, {
    asset: asset.publicKey,
    payer,
    authority,
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

test('it cannot burn an asset if an authority is provided and the payer does not sign', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = generateSigner(umi);
  const authority = generateSigner(umi);
  await umi.rpc.airdrop(payer.publicKey, sol(10));

  const asset = await createAsset(umi, {
    payer,
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: payer.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });

  const instruction = burnV1(umi, {
    asset: asset.publicKey,
    payer,
    authority,
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
