import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { address } from '@solana/addresses';
import {
  getCompressV1Instruction,
  getCreateV1Instruction,
  DataState,
  fetchAssetV1,
  fetchHashedAssetV1,
  getAssetV1Encoder,
  getHashedAssetSchemaEncoder,
  hash,
  type HashedAssetSchema,
} from '../src';
import {
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  sendAndConfirmInstructions,
} from './_setup';

test.skip('it can compress an asset without any plugins as the owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction = getCompressV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    logWrapper: address('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  const afterAsset = await fetchHashedAssetV1(rpc, asset.address);

  const hashedAssetSchema: HashedAssetSchema = {
    assetHash: hash(new Uint8Array(getAssetV1Encoder().encode(asset.data))),
    pluginHashes: [],
  };

  const hashedAsset = hash(
    new Uint8Array(getHashedAssetSchemaEncoder().encode(hashedAssetSchema))
  );
  t.deepEqual(afterAsset.data.hash, hashedAsset);
});

test.skip('it cannot compress an asset if not the owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const attacker = await generateSignerWithSol(rpc);

  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  const beforeAsset = await fetchAssetV1(rpc, assetAddress.address);
  t.is(beforeAsset.data.owner, payer.address);
  t.is(beforeAsset.data.name, 'Test Bread');
  t.is(beforeAsset.data.uri, 'https://example.com/bread');

  const compressInstruction = getCompressV1Instruction({
    asset: assetAddress.address,
    payer: attacker,
    authority: attacker,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [compressInstruction],
    [attacker]
  );

  await t.throwsAsync(result);

  const afterAsset = await fetchAssetV1(rpc, assetAddress.address);
  t.is(afterAsset.data.owner, payer.address);
  t.is(afterAsset.data.name, 'Test Bread');
  t.is(afterAsset.data.uri, 'https://example.com/bread');
});

test('it cannot compress an asset because it is not available', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  const beforeAsset = await fetchAssetV1(rpc, assetAddress.address);
  t.is(beforeAsset.data.owner, payer.address);
  t.is(beforeAsset.data.name, 'Test Bread');
  t.is(beforeAsset.data.uri, 'https://example.com/bread');

  const compressInstruction = getCompressV1Instruction({
    asset: assetAddress.address,
    payer,
    authority: payer,
    logWrapper: address('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [compressInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid system program', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const fakeSystemProgram = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  const beforeAsset = await fetchAssetV1(rpc, assetAddress.address);
  t.is(beforeAsset.data.owner, payer.address);
  t.is(beforeAsset.data.name, 'Test Bread');
  t.is(beforeAsset.data.uri, 'https://example.com/bread');

  const compressInstruction = getCompressV1Instruction({
    asset: assetAddress.address,
    payer,
    authority: payer,
    systemProgram: fakeSystemProgram.address,
    logWrapper: address('noopb9bkMVfRPU8AQkHtKwMYZiFUjNRtMmV'),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [compressInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid log wrapper program', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const fakeLogWrapper = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  const beforeAsset = await fetchAssetV1(rpc, assetAddress.address);
  t.is(beforeAsset.data.owner, payer.address);
  t.is(beforeAsset.data.name, 'Test Bread');
  t.is(beforeAsset.data.uri, 'https://example.com/bread');

  const compressInstruction = getCompressV1Instruction({
    asset: assetAddress.address,
    payer,
    authority: payer,
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [compressInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});
