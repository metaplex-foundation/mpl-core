import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getUpdateV1Instruction,
  getUpdateCollectionV1Instruction,
  type BaseUpdateAuthority,
} from '../src';
import { pluginAuthorityPair } from '../src/plugins';
import {
  createAsset,
  createAssetWithCollection,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from './_setup';

test('it can update an asset to be larger', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('it cannot update an asset using asset as authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const myAsset = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    asset: myAsset,
    name: 'short',
    uri: 'https://short.com',
  });

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    authority: myAsset,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [myAsset, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    name: 'short',
    uri: 'https://short.com',
  });
});

test('it can update an asset to be smaller', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: '',
    newUri: '',
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    name: '',
    uri: '',
  });
});

test('it can update an asset with plugins to be larger', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    name: 'short',
    uri: 'https://short.com',
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it can update an asset with plugins to be smaller', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: '',
    newUri: '',
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    name: '',
    uri: '',
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it can update an asset update authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);
  const newUpdateAuthority = await generateKeyPairSigner();

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    newUpdateAuthority: {
      __kind: 'Address',
      fields: [newUpdateAuthority.address],
    } as BaseUpdateAuthority,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: newUpdateAuthority.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('it can update an asset update authority to None', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    newUpdateAuthority: {
      __kind: 'None',
    } as BaseUpdateAuthority,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'None' },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('it can update an asset with plugins update authority to None', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    newUpdateAuthority: {
      __kind: 'None',
    } as BaseUpdateAuthority,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'None' },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('it cannot update an asset update authority to be part of a collection using updateV1', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);
  const newCollection = await generateKeyPairSigner();

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    newUpdateAuthority: {
      __kind: 'Collection',
      fields: [newCollection.address],
    } as BaseUpdateAuthority,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot remove an asset from a collection using updateV1', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    collection: collection.address,
    newUpdateAuthority: {
      __kind: 'Address',
      fields: [payer.address],
    } as BaseUpdateAuthority,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot update an asset using wrong authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateAuth = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    updateAuthority: updateAuth,
  });

  const newUpdateAuthority = await generateKeyPairSigner();

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    newUpdateAuthority: {
      __kind: 'Address',
      fields: [newUpdateAuthority.address],
    } as BaseUpdateAuthority,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: updateAuth.address },
  });
});

test('it cannot use an invalid system program for assets', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);
  const fakeSystemProgram = await generateKeyPairSigner();

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid noop program for assets', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);
  const fakeLogWrapper = await generateKeyPairSigner();

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid system program for collections', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer);
  const fakeSystemProgram = await generateKeyPairSigner();

  const instruction = getUpdateCollectionV1Instruction({
    collection: collection.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid noop program for collections', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer);
  const fakeLogWrapper = await generateKeyPairSigner();

  const instruction = getUpdateCollectionV1Instruction({
    collection: collection.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});
