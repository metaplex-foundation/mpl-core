import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { getBurnCollectionV1Instruction } from '../src';
import {
  createAssetWithCollection,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertCollection,
  assertBurned,
  DEFAULT_COLLECTION,
  sendAndConfirmInstructions,
} from './_setup';

test('it can burn a collection as the authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer);

  await assertCollection(t, rpc, {
    collection: collection.address,
    updateAuthority: payer.address,
    name: DEFAULT_COLLECTION.name,
    uri: DEFAULT_COLLECTION.uri,
  });

  const instruction = getBurnCollectionV1Instruction({
    collection: collection.address,
    payer,
    compressionProof: null,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertBurned(t, rpc, collection.address);
});

test('it cannot burn a collection if not the authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const attacker = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer);

  await assertCollection(t, rpc, {
    collection: collection.address,
    updateAuthority: payer.address,
    name: DEFAULT_COLLECTION.name,
    uri: DEFAULT_COLLECTION.uri,
  });

  const instruction = getBurnCollectionV1Instruction({
    collection: collection.address,
    payer,
    authority: attacker,
    compressionProof: null,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [attacker, payer]
  );

  await t.throwsAsync(result);

  await assertCollection(t, rpc, {
    collection: collection.address,
    updateAuthority: payer.address,
    name: DEFAULT_COLLECTION.name,
    uri: DEFAULT_COLLECTION.uri,
  });
});

test('it cannot burn a collection if it has Assets in it', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const { collection } = await createAssetWithCollection(rpc, payer, {});

  await assertCollection(t, rpc, {
    collection: collection.address,
    updateAuthority: payer.address,
    name: DEFAULT_COLLECTION.name,
    uri: DEFAULT_COLLECTION.uri,
  });

  const instruction = getBurnCollectionV1Instruction({
    collection: collection.address,
    payer,
    compressionProof: null,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertCollection(t, rpc, {
    collection: collection.address,
    updateAuthority: payer.address,
    name: DEFAULT_COLLECTION.name,
    uri: DEFAULT_COLLECTION.uri,
  });
});

test('it can burn asset with different payer', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const authority = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    updateAuthority: authority.address,
  });

  await assertCollection(t, rpc, {
    collection: collection.address,
    updateAuthority: authority.address,
    name: DEFAULT_COLLECTION.name,
    uri: DEFAULT_COLLECTION.uri,
  });

  const lamportsBefore = await rpc.getBalance(payer.address).send();

  const instruction = getBurnCollectionV1Instruction({
    collection: collection.address,
    payer,
    authority,
    compressionProof: null,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [authority, payer]
  );

  await assertBurned(t, rpc, collection.address);

  const lamportsAfter = await rpc.getBalance(payer.address).send();

  t.true(lamportsAfter.value > lamportsBefore.value);
});

test('it cannot use an invalid noop program for collections', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer);
  const fakeLogWrapper = await generateKeyPairSigner();

  await assertCollection(t, rpc, {
    collection: collection.address,
    updateAuthority: payer.address,
    name: DEFAULT_COLLECTION.name,
    uri: DEFAULT_COLLECTION.uri,
  });

  const instruction = getBurnCollectionV1Instruction({
    collection: collection.address,
    payer,
    logWrapper: fakeLogWrapper.address,
    compressionProof: null,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});
