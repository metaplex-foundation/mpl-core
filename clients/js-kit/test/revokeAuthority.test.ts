import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  PluginType,
  getRevokePluginAuthorityV1Instruction,
  getRevokeCollectionPluginAuthorityV1Instruction,
  getApprovePluginAuthorityV1Instruction,
  getApproveCollectionPluginAuthorityV1Instruction,
  pluginAuthority,
} from '../src';
import { pluginAuthorityPair } from '../src/plugins';
import {
  assertAsset,
  assertCollection,
  createAsset,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  sendAndConfirmInstructions,
} from './_setup';

test('it can remove an authority from a plugin', async (t) => {
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

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
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

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
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
});

test('it can remove the default authority from a plugin to make it immutable', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'None' },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'None',
      },
      frozen: false,
    },
  });
});

test('it can remove a pubkey authority from an owner-managed plugin if that pubkey is the signer authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const pubkeyAuth = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'Address', address: pubkeyAuth.address },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: pubkeyAuth.address,
      },
      frozen: false,
    },
  });

  const payer2 = await generateSignerWithSol(rpc);

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer: payer2,
    authority: pubkeyAuth,
    pluginType: PluginType.FreezeDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [pubkeyAuth, payer2]
  );

  await assertAsset(t, rpc, {
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
});

test('it can remove an update authority from an owner-managed plugin if that pubkey is the signer authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'UpdateAuthority' },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [owner, payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: { type: 'UpdateAuthority' },
      frozen: false,
    },
  });

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it can remove a pubkey authority from an authority-managed plugin if that pubkey is the signer authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const pubkeyAuth = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'Attributes', data: { attributeList: [] } }),
    ],
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.Attributes,
    newAuthority: { __kind: 'Address', address: pubkeyAuth.address },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    attributes: {
      authority: {
        type: 'Address',
        address: pubkeyAuth.address,
      },
      attributeList: [],
    },
  });

  const payer2 = await generateSignerWithSol(rpc);

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer: payer2,
    authority: pubkeyAuth,
    pluginType: PluginType.Attributes,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [pubkeyAuth, payer2]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [],
    },
  });
});

test('it can remove an owner authority from an authority-managed plugin if that pubkey is the signer authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({ type: 'Attributes', data: { attributeList: [] } }),
    ],
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.Attributes,
    newAuthority: { __kind: 'Owner' },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
    attributes: {
      authority: {
        type: 'Owner',
      },
      attributeList: [],
    },
  });

  const payer2 = await generateSignerWithSol(rpc);

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer: payer2,
    authority: owner,
    pluginType: PluginType.Attributes,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [owner, payer2]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [],
    },
  });
});

test('it cannot remove a none authority from a plugin', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'None' },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    pluginType: PluginType.FreezeDelegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid system program for assets', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();
  const fakeSystemProgram = await generateKeyPairSigner();

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

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
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

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid noop program for assets', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();
  const fakeLogWrapper = await generateKeyPairSigner();

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

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
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

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid system program for collections', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();
  const fakeSystemProgram = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    plugins: [pluginAuthorityPair({ type: 'UpdateDelegate' })],
  });

  const approveInstruction = getApproveCollectionPluginAuthorityV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await assertCollection(t, rpc, {
    collection: collection.address,
    updateAuthority: payer.address,
  });

  const revokeInstruction = getRevokeCollectionPluginAuthorityV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid noop program for collections', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();
  const fakeLogWrapper = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    plugins: [pluginAuthorityPair({ type: 'UpdateDelegate' })],
  });

  const approveInstruction = getApproveCollectionPluginAuthorityV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await assertCollection(t, rpc, {
    collection: collection.address,
    updateAuthority: payer.address,
  });

  const revokeInstruction = getRevokeCollectionPluginAuthorityV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can revoke an authority from a plugin if another plugin is None', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
      pluginAuthorityPair({
        authority: pluginAuthority('None'),
        type: 'PermanentFreezeDelegate',
        data: { frozen: false },
      }),
    ],
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
    [approveInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
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

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
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
});
