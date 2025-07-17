import test from 'ava';
import { addAssetsToGroup, removeAssetsFromGroup } from '../src';
import { createAsset, createGroup, createUmi } from './_setupRaw';

const MAX_COMPUTE_UNITS = 1_400_000; // 1.4M Compute Units – Solana TX limit.

/**
 * Utility to fetch compute units consumed by a confirmed transaction signature.
 */
async function getComputeUnits(
  umi: Awaited<ReturnType<typeof createUmi>>,
  signature: Uint8Array
) {
  const txInfo = await umi.rpc.getTransaction(signature);
  return Number(txInfo?.meta.computeUnitsConsumed ?? 0);
}

// Ensure we stay within Solana's 1.4M CU per-TX hard limit.
// NOTE: This benchmark has been moved to the dedicated `/bench` folder to
// keep the deterministic unit‐test suite lightweight. See
// `clients/js/bench/compute.ts` for the performance check.

test('compute units: removing 1 asset from a group stays below the 1.4M limit', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);

  // Create and add 1 asset to ensure it is a member of the group first.
  const assets = await Promise.all(
    Array.from({ length: 1 }).map(() => createAsset(umi))
  );

  // Initial add so that we can remove afterwards.
  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts(
      assets.map((asset) => ({
        isSigner: false,
        isWritable: true,
        pubkey: asset.publicKey,
      }))
    )
    .sendAndConfirm(umi);

  // Now remove those same assets in a single instruction.
  const removeTx = await removeAssetsFromGroup(umi, {
    group: group.publicKey,
    assets: assets.map((a) => a.publicKey),
    authority: umi.identity,
  })
    .addRemainingAccounts(
      assets.map((asset) => ({
        isSigner: false,
        isWritable: true,
        pubkey: asset.publicKey,
      }))
    )
    .sendAndConfirm(umi);

  const computeUnits = await getComputeUnits(umi, removeTx.signature);

  t.true(
    computeUnits <= MAX_COMPUTE_UNITS,
    `Removing 1 asset used ${computeUnits} CUs which exceeds the 1.4M limit.`
  );
});
