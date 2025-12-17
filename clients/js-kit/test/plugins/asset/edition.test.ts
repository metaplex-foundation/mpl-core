import test from 'ava';
import {
  getAddPluginV1Instruction,
  getApprovePluginAuthorityV1Instruction,
  getRemovePluginV1Instruction,
  getRevokePluginAuthorityV1Instruction,
  getUpdatePluginV1Instruction,
  PluginType,
} from '../../../src';
import { pluginAuthorityPair, createPlugin } from '../../../src/plugins';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can create asset with edition plugin', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });
});

test('it cannot add edition plugin after mint', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {});

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Edition',
      data: { number: 1 },
    }),
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
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot add edition plugin to collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const result = createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
  });

  await t.throwsAsync(result);
});

test('it cannot remove edition plugin', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.Edition,
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
    updateAuthority: { type: 'Address', address: payer.address },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });
});

test('it can update edition plugin', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Edition',
      data: { number: 2 },
    }),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 2,
    },
  });
});

test('it cannot update edition plugin as owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
    owner: owner.address,
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Edition',
      data: { number: 2 },
    }),
    authority: owner,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [owner, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });
});

test('it can create immutable edition plugin', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
        authority: { __kind: 'None' },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    edition: {
      authority: {
        type: 'None',
      },
      number: 1,
    },
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Edition',
      data: { number: 2 },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can make edition plugin immutable', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.Edition,
    newAuthority: { __kind: 'None' },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    edition: {
      authority: {
        type: 'None',
      },
      number: 1,
    },
  });

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.Edition,
  });

  const revokeResult = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [payer]
  );

  await t.throwsAsync(revokeResult);

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Edition',
      data: { number: 2 },
    }),
  });

  const updateResult = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
    [payer]
  );

  await t.throwsAsync(updateResult);
});
