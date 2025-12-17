import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getAddPluginV1Instruction,
  getUpdatePluginV1Instruction,
  getApprovePluginAuthorityV1Instruction,
  getRevokePluginAuthorityV1Instruction,
  getRemovePluginV1Instruction,
  PluginType,
} from '../../../src';
import { pluginAuthorityPair } from '../../../src/plugins';
import {
  createAsset,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can freeze and unfreeze an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const addPluginInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'FreezeDelegate',
      fields: [{ frozen: true }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [addPluginInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });

  const updatePluginInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'FreezeDelegate',
      fields: [{ frozen: false }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updatePluginInstruction],
    [payer]
  );

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
});

test('it can delegate then freeze an asset', async (t) => {
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
    newAuthority: {
      __kind: 'Address',
      address: delegateAddress.address,
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  const updatePluginInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: delegateAddress,
    plugin: {
      __kind: 'FreezeDelegate',
      fields: [{ frozen: true }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updatePluginInstruction],
    [delegateAddress, payer]
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
      frozen: true,
    },
  });
});

test('owner cannot undelegate a freeze plugin with a delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
        authority: {
          __kind: 'Address',
          address: delegateAddress.address,
        },
      }),
    ],
  });

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
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

test('owner cannot approve to reassign authority back to owner if frozen', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
        authority: {
          __kind: 'Address',
          address: delegateAddress.address,
        },
      }),
    ],
  });

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
      frozen: true,
    },
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: {
      __kind: 'Owner',
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
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
      frozen: true,
    },
  });
});

test('it cannot add multiple freeze plugins to an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const addPluginInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'FreezeDelegate',
      fields: [{ frozen: true }],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [addPluginInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot remove freeze plugin if update authority and frozen', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  const removePluginInstruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [removePluginInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it delegate cannot freeze after delegate has been revoked', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: {
          __kind: 'Address',
          address: delegateAddress.address,
        },
      }),
    ],
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

  const updatePluginInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: delegateAddress,
    plugin: {
      __kind: 'FreezeDelegate',
      fields: [{ frozen: true }],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updatePluginInstruction],
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
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it owner cannot unfreeze frozen asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
        authority: {
          __kind: 'Address',
          address: delegateAddress.address,
        },
      }),
    ],
  });

  const updatePluginInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    plugin: {
      __kind: 'FreezeDelegate',
      fields: [{ frozen: false }],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updatePluginInstruction],
    [owner, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.address,
      },
      frozen: true,
    },
  });
});

test('it update authority cannot unfreeze frozen asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const updateAuthority = await generateKeyPairSigner();
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    updateAuthority: updateAuthority.address,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
        authority: {
          __kind: 'Address',
          address: delegateAddress.address,
        },
      }),
    ],
  });

  const updatePluginInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateAuthority,
    plugin: {
      __kind: 'FreezeDelegate',
      fields: [{ frozen: false }],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updatePluginInstruction],
    [updateAuthority, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthority.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.address,
      },
      frozen: true,
    },
  });
});

test('a freezeDelegate can freeze using delegated update authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const updateAuthority = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    updateAuthority: updateAuthority.address,
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: {
          __kind: 'UpdateAuthority',
        },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthority.address },
    freezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  const updatePluginInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateAuthority,
    plugin: {
      __kind: 'FreezeDelegate',
      fields: [{ frozen: true }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updatePluginInstruction],
    [updateAuthority, payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthority.address },
    freezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('a freezeDelegate can freeze using delegated update authority from collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const updateAuthority = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    updateAuthority: updateAuthority.address,
  });

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    collection: collection.address,
    authority: updateAuthority,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: {
          __kind: 'UpdateAuthority',
        },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Collection', address: collection.address },
    freezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  const updatePluginInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    authority: updateAuthority,
    plugin: {
      __kind: 'FreezeDelegate',
      fields: [{ frozen: true }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updatePluginInstruction],
    [updateAuthority, payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Collection', address: collection.address },
    freezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});
