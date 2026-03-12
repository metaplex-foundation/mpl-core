import test from 'ava';
import { getBurnV1Instruction, fetchCollectionV1 } from '../src';
import {
  createAsset,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  sendAndConfirmInstructions,
} from './_setup';

test('it can burn an asset which is the part of a collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {});

  const asset1 = await createAsset(rpc, payer, {
    collection: collection.address,
  });

  const asset2 = await createAsset(rpc, payer, {
    collection: collection.address,
  });

  const collectionAfterMinting = await fetchCollectionV1(
    rpc,
    collection.address
  );
  t.is(collectionAfterMinting.data.currentSize, 2);
  t.is(collectionAfterMinting.data.numMinted, 2);

  const burnInstruction1 = getBurnV1Instruction({
    asset: asset1.address,
    collection: collection.address,
    payer,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [burnInstruction1],
    [payer]
  );

  const burnInstruction2 = getBurnV1Instruction({
    asset: asset2.address,
    collection: collection.address,
    payer,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [burnInstruction2],
    [payer]
  );

  const collectionAfterBurning = await fetchCollectionV1(
    rpc,
    collection.address
  );
  t.is(collectionAfterBurning.data.currentSize, 0);
  t.is(collectionAfterBurning.data.numMinted, 2);
});
