import { transferSol } from '@metaplex-foundation/mpl-toolbox';
import {
  createNoopSigner,
  generateSigner,
  publicKey,
  sol,
} from '@metaplex-foundation/umi';
import test from 'ava';

import {
  burnV1,
  create,
  execute,
  fetchAssetV1,
  findAssetSignerPda,
} from '../src';
import { assertAsset, assertBurned, createUmi } from './_setupRaw';

test('it covers the freeze execute backed NFT flow', async (t) => {
  // ----------------------------------
  // 0. Test setup.
  // ----------------------------------
  const umi = await createUmi();

  // ----------------------------------
  // 1. Mint an asset with FreezeExecute { frozen: true }.
  // ----------------------------------
  const assetSigner = generateSigner(umi);

  await create(umi, {
    asset: assetSigner,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [{ type: 'FreezeExecute', frozen: true }],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, publicKey(assetSigner));

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  // ----------------------------------
  // 2. Deposit backing SOL into the asset account (simulate 0.5 SOL backing).
  // ----------------------------------
  // The Execute instruction pulls funds from the asset signer PDA, not the
  // asset account itself, so credit that PDA with some lamports.
  const [assetSignerPda] = findAssetSignerPda(umi, {
    asset: asset.publicKey,
  });

  await transferSol(umi, {
    source: umi.identity,
    destination: publicKey(assetSignerPda),
    amount: sol(0.5),
  }).sendAndConfirm(umi);

  // ----------------------------------
  // 3. Attempt Execute → should fail because plugin is frozen.
  // ----------------------------------
  const recipient = generateSigner(umi);

  const execResult = execute(umi, {
    asset,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSignerPda)),
      destination: recipient.publicKey,
      amount: sol(0.1),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(execResult, { name: 'InvalidAuthority' });

  // ----------------------------------
  // 4. Burn the asset → user should receive lamports back, asset account closed.
  // ----------------------------------
  const balanceBefore = await umi.rpc.getBalance(umi.identity.publicKey);

  await burnV1(umi, {
    asset: asset.publicKey,
  }).sendAndConfirm(umi);

  // Assert the asset account is burned (resized to 1 byte).
  await assertBurned(t, umi, asset.publicKey);

  const balanceAfter = await umi.rpc.getBalance(umi.identity.publicKey);
  t.true(
    balanceAfter.basisPoints > balanceBefore.basisPoints,
    'Payer balance did not increase after burn refund'
  );
});
