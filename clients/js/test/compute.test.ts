import test from 'ava';
import { addAssetsToGroup, removeAssetsFromGroup } from '../src';
import { createAsset, createGroup, createUmi } from './_setupRaw';

const MAX_COMPUTE_UNITS = 1_400_000;
const STRESS_ADD_ASSET_COUNT = 20;
const STRESS_REMOVE_ASSET_COUNT = 10;

/**
 * Utility to fetch compute units consumed by a confirmed transaction signature.
 */
async function getComputeUnits(
  umi: Awaited<ReturnType<typeof createUmi>>,
  signature: Uint8Array
) {
  const txInfo = await umi.rpc.getTransaction(signature);
  const computeUnitsConsumed = txInfo?.meta?.computeUnitsConsumed;
  if (computeUnitsConsumed == null) {
    throw new Error(
      'Unable to read compute units for the confirmed transaction'
    );
  }

  return Number(computeUnitsConsumed);
}

const toWritableRemainingAccounts = (
  assets: Awaited<ReturnType<typeof createAsset>>[]
) =>
  assets.map((asset) => ({
    isSigner: false,
    isWritable: true,
    pubkey: asset.publicKey,
  }));

test.serial(
  `compute units: adding ${STRESS_ADD_ASSET_COUNT} assets to a group stays below the 1.4M limit`,
  async (t) => {
    const umi = await createUmi();
    const group = await createGroup(umi);

    const assets = await Promise.all(
      Array.from({ length: STRESS_ADD_ASSET_COUNT }).map(() => createAsset(umi))
    );

    const addTx = await addAssetsToGroup(umi, {
      group: group.publicKey,
      authority: umi.identity,
    })
      .addRemainingAccounts(toWritableRemainingAccounts(assets))
      .sendAndConfirm(umi);

    const computeUnits = await getComputeUnits(umi, addTx.signature);

    t.true(
      computeUnits <= MAX_COMPUTE_UNITS,
      `Adding ${STRESS_ADD_ASSET_COUNT} assets used ${computeUnits} CUs which exceeds the 1.4M limit.`
    );
  }
);

test.serial(
  `compute units: removing ${STRESS_REMOVE_ASSET_COUNT} assets from a group stays below the 1.4M limit`,
  async (t) => {
    const umi = await createUmi();
    const group = await createGroup(umi);

    const assets = await Promise.all(
      Array.from({ length: STRESS_REMOVE_ASSET_COUNT }).map(() =>
        createAsset(umi)
      )
    );

    await addAssetsToGroup(umi, {
      group: group.publicKey,
      authority: umi.identity,
    })
      .addRemainingAccounts(toWritableRemainingAccounts(assets))
      .sendAndConfirm(umi);

    const removeTx = await removeAssetsFromGroup(umi, {
      group: group.publicKey,
      assets: assets.map((a) => a.publicKey),
      authority: umi.identity,
    })
      .addRemainingAccounts(toWritableRemainingAccounts(assets))
      .sendAndConfirm(umi);

    const computeUnits = await getComputeUnits(umi, removeTx.signature);

    t.true(
      computeUnits <= MAX_COMPUTE_UNITS,
      `Removing ${STRESS_REMOVE_ASSET_COUNT} assets used ${computeUnits} CUs which exceeds the 1.4M limit.`
    );
  }
);
