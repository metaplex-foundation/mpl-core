import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  PluginType,
  getApprovePluginAuthorityV1Instruction,
  getAddPluginV1Instruction,
  getAddCollectionPluginV1Instruction,
  getApproveCollectionPluginAuthorityV1Instruction,
} from '../src';
import { pluginAuthorityPair, createPlugin } from '../src/plugins';
import {
  DEFAULT_ASSET,
  assertAsset,
  assertCollection,
  createAsset,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  sendAndConfirmInstructions,
} from './_setup';

test('it can add an authority to a plugin', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const addPluginInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [addPluginInstruction, approveInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.address,
      },
      frozen: false,
    },
  });
});

// TODO: This should fail
test('it cannot reassign authority of a plugin while already delegated', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();
  const newDelegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [approveInstruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.address,
      },
      frozen: false,
    },
  });

  const secondApproveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'Address', address: newDelegateAddress.address },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [secondApproveInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.address,
      },
      frozen: false,
    },
  });
});

test('it cannot reassign authority of a plugin as delegate while already delegated', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateSignerWithSol(rpc);
  const newDelegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [approveInstruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.address,
      },
      frozen: false,
    },
  });

  const secondApproveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    authority: delegateAddress,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'Address', address: newDelegateAddress.address },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [secondApproveInstruction],
    [delegateAddress, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.address,
      },
      frozen: false,
    },
  });
});

test('it cannot approve to reassign authority back to owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [approveInstruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.address,
      },
      frozen: false,
    },
  });

  const secondApproveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'Owner' },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [secondApproveInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.address,
      },
      frozen: false,
    },
  });
});

test('it cannot use an invalid system program for assets', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);
  const fakeSystemProgram = await generateKeyPairSigner();

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const addPluginInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addPluginInstruction], [
    payer,
  ]);

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid noop program for assets', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);
  const fakeLogWrapper = await generateKeyPairSigner();

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const addPluginInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addPluginInstruction], [
    payer,
  ]);

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid system program for collections', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer);
  const fakeSystemProgram = await generateKeyPairSigner();

  await assertCollection(t, rpc, {
    collection: collection.address,
    updateAuthority: payer.address,
  });

  const addPluginInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addPluginInstruction], [
    payer,
  ]);

  const approveInstruction = getApproveCollectionPluginAuthorityV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid noop program for collections', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer);
  const fakeLogWrapper = await generateKeyPairSigner();

  await assertCollection(t, rpc, {
    collection: collection.address,
    updateAuthority: payer.address,
  });

  const addPluginInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addPluginInstruction], [
    payer,
  ]);

  const approveInstruction = getApproveCollectionPluginAuthorityV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});
