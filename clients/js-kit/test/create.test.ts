import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { address } from '@solana/addresses';
import {
  getCreateV1Instruction,
  DataState,
  Key,
  fetchAssetV1,
} from '../src';
import {
  createAsset,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from './_setup';
import { pluginAuthorityPair } from '../src/plugins';

test('it can create a new asset in account state', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();

  await createAsset(rpc, payer, {
    asset: assetAddress,
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: assetAddress.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it can create a new asset with a different payer', async (t) => {
  const rpc = createRpc();
  const identity = await generateSignerWithSol(rpc);
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    asset: assetAddress,
    owner: identity,
    payer,
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: identity.address,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot create a new asset in ledger state because it is not available', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    dataState: DataState.LedgerState,
    asset: assetAddress,
    payer,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    logWrapper: address('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  await t.throwsAsync(result);
});

test('it can create a new asset in account state with plugins', async (t) => {
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
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  const asset = await fetchAssetV1(rpc, assetAddress.address);

  t.is(asset.data.key, Key.AssetV1);
  t.is(asset.data.owner, payer.address);
  t.is(asset.data.name, 'Test Bread');
  t.is(asset.data.uri, 'https://example.com/bread');
});

test('it can create a new asset in account state with a different update authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const updateAuth = await generateKeyPairSigner();

  await createAsset(rpc, payer, {
    asset: assetAddress,
    updateAuthority: updateAuth.address,
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: assetAddress.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: updateAuth.address },
  });
});

test('it can create a new asset in account state with plugins with a different update authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const updateAuth = await generateKeyPairSigner();

  await createAsset(rpc, payer, {
    asset: assetAddress,
    updateAuthority: updateAuth.address,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: assetAddress.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: updateAuth.address },
  });
});

test('it cannot create a new asset if the address is already in use', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();

  await createAsset(rpc, payer, {
    asset: assetAddress,
  });

  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: DEFAULT_ASSET.name,
    uri: DEFAULT_ASSET.uri,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot create a new asset with an update authority that is not the collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer);
  const otherAuth = await generateKeyPairSigner();

  const result = createAsset(rpc, payer, {
    collection: collection.address,
    updateAuthority: otherAuth,
  });

  await t.throwsAsync(result);
});

test('it cannot use an invalid system program for assets', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const fakeSystemProgram = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset: assetAddress,
    payer,
    name: 'Test',
    uri: 'Test',
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid noop program for assets', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const fakeLogWrapper = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset: assetAddress,
    payer,
    name: 'Test',
    uri: 'Test',
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  await t.throwsAsync(result);
});
