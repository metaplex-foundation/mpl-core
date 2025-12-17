import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { address } from '@solana/addresses';
import {
  getAddPluginV1Instruction,
  getApprovePluginAuthorityV1Instruction,
  getRemovePluginV1Instruction,
  getRevokePluginAuthorityV1Instruction,
  getUpdateV1Instruction,
  getUpdatePluginV1Instruction,
  PluginType,
} from '../../../src';
import { pluginAuthorityPair, createPlugin } from '../../../src/plugins';
import {
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can create an asset with updateDelegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });
});

test('it can create an asset with updateDelegate with additional delegates', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.address],
    },
  });
});

test('it can add updateDelegate to asset with additional delegates', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);
  const updateDelegate = await generateKeyPairSigner();

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate.address] },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.address],
    },
  });
});

test('it can update updateDelegate on asset with additional delegates', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);
  const updateDelegate = await generateKeyPairSigner();

  const addInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [] },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addInstruction], [payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate.address] },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [updateInstruction], [payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.address],
    },
  });
});

test('an updateDelegate can update an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer, {
    name: 'short',
    uri: 'https://short.com',
  });
  const updateDelegate = await generateKeyPairSigner();

  const addInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [] },
    }),
    initAuthority: { __kind: 'Address', address: updateDelegate.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addInstruction], [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  const updateInstruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [updateInstruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('an updateDelegate additionalDelegate can update an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer, {
    name: 'short',
    uri: 'https://short.com',
  });
  const updateDelegate = await generateKeyPairSigner();

  const addInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate.address] },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addInstruction], [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.address],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  const updateInstruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [updateInstruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.address],
    },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('an updateDelegate cannot update an asset after delegate authority revoked', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer, {
    name: 'short',
    uri: 'https://short.com',
  });
  const updateDelegate = await generateKeyPairSigner();

  const addInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [] },
    }),
    initAuthority: { __kind: 'Address', address: updateDelegate.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addInstruction], [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [revokeInstruction], [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  const updateInstruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  });

  const result = sendAndConfirmInstructions(rpc, rpcSubscriptions, [updateInstruction], [updateDelegate, payer]);

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });
});

test('an updateDelegate additionalDelegate cannot update an asset after delegate authority revoked', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer, {
    name: 'short',
    uri: 'https://short.com',
  });
  const updateDelegate = await generateKeyPairSigner();

  const addInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate.address] },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addInstruction], [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.address],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  const updatePluginInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [] },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [updatePluginInstruction], [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  const updateInstruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  });

  const result = sendAndConfirmInstructions(rpc, rpcSubscriptions, [updateInstruction], [updateDelegate, payer]);

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });
});

test('it cannot add additional delegate as additional delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();
  const updateDelegate2 = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: {
        additionalDelegates: [updateDelegate.address, updateDelegate2.address],
      },
    }),
  });

  const result = sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await t.throwsAsync(result);
});

test('it can remove additional delegate as additional delegate if self', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();
  const updateDelegate2 = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: {
          additionalDelegates: [updateDelegate.address, updateDelegate2.address],
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate2.address] },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate2.address],
    },
  });
});

test('it cannot remove another additional delegate as additional delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();
  const updateDelegate2 = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: {
          additionalDelegates: [updateDelegate.address, updateDelegate2.address],
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate.address] },
    }),
  });

  const result = sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await t.throwsAsync(result);
});

test('it cannot approve the update delegate plugin authority as additional delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();
  const updateDelegate2 = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
    ],
  });

  const instruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: { __kind: 'Address', address: updateDelegate2.address },
  });

  const result = sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await t.throwsAsync(result);
});

test('it cannot revoke the update delegate plugin authority as additional delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();
  const updateDelegate2 = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
        authority: { type: 'Address', address: updateDelegate2.address },
      }),
    ],
  });

  const instruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    pluginType: PluginType.UpdateDelegate,
  });

  const result = sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await t.throwsAsync(result);
});

test('it can approve/revoke the plugin authority of non-updateDelegate plugins as additional delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();
  const updateDelegate2 = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    pluginType: PluginType.Edition,
    newAuthority: { __kind: 'Address', address: updateDelegate2.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [approveInstruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.address],
    },
    edition: {
      authority: {
        type: 'Address',
        address: updateDelegate2.address,
      },
      number: 1,
    },
  });

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    pluginType: PluginType.Edition,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [revokeInstruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.address],
    },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });
});

test('it can update a non-updateDelegate plugin as additional delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    plugin: createPlugin({
      type: 'Edition',
      data: { number: 2 },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
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

test('it cannot add updateDelegate plugin with additional delegate as additional delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer);

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    plugin: createPlugin({
      type: 'UpdateDelegate',
      data: { additionalDelegates: [updateDelegate.address] },
    }),
  });

  const result = sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await t.throwsAsync(result);
});

test('it can update the update authority of the asset as an updateDelegate additional delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();
  const updateDelegate2 = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
    ],
  });

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    newUpdateAuthority: { __kind: 'Address', fields: [updateDelegate2.address] },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: updateDelegate2.address },
  });
});

test('it can update the update authority of the asset as an updateDelegate root authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();
  const updateDelegate2 = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        authority: { type: 'Address', address: updateDelegate.address },
        data: { additionalDelegates: [] },
      }),
    ],
  });

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    newUpdateAuthority: { __kind: 'Address', fields: [updateDelegate2.address] },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: updateDelegate2.address },
  });
});

test('an updateDelegate can add a plugin to an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
        authority: { type: 'Address', address: updateDelegate.address },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    plugin: createPlugin({
      type: 'Attributes',
      data: {
        attributeList: [
          {
            key: '123',
            value: '456',
          },
        ],
      },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        {
          key: '123',
          value: '456',
        },
      ],
    },
  });
});

test('an updateDelegate can add a plugin to an asset using delegated owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const updateAuthorityAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    updateAuthority: updateAuthorityAddress.address,
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
        authority: { type: 'Owner' },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthorityAddress.address },
    updateDelegate: {
      authority: {
        type: 'Owner',
      },
      additionalDelegates: [],
    },
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    plugin: createPlugin({
      type: 'Attributes',
      data: {
        attributeList: [
          {
            key: '123',
            value: '456',
          },
        ],
      },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [owner, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthorityAddress.address },
    updateDelegate: {
      authority: {
        type: 'Owner',
      },
      additionalDelegates: [],
    },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        {
          key: '123',
          value: '456',
        },
      ],
    },
  });
});

test('an updateDelegate can remove a plugin from an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
        authority: { type: 'Address', address: updateDelegate.address },
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: {
          attributeList: [
            {
              key: '123',
              value: '456',
            },
          ],
        },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        {
          key: '123',
          value: '456',
        },
      ],
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    pluginType: PluginType.Attributes,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
  });
});

test('an updateDelegate can remove a plugin from an asset using delegated owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const updateAuthorityAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    updateAuthority: updateAuthorityAddress.address,
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
        authority: { type: 'Owner' },
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: {
          attributeList: [
            {
              key: '123',
              value: '456',
            },
          ],
        },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthorityAddress.address },
    updateDelegate: {
      authority: {
        type: 'Owner',
      },
      additionalDelegates: [],
    },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        {
          key: '123',
          value: '456',
        },
      ],
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    pluginType: PluginType.Attributes,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [owner, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthorityAddress.address },
    updateDelegate: {
      authority: {
        type: 'Owner',
      },
      additionalDelegates: [],
    },
  });
});

test('it can approve/revoke the plugin authority of other plugins', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
        authority: { type: 'Address', address: updateDelegate.address },
      }),
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
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });

  const editionAuthority = await generateKeyPairSigner();
  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    pluginType: PluginType.Edition,
    newAuthority: { __kind: 'Address', address: editionAuthority.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [approveInstruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    edition: {
      authority: {
        type: 'Address',
        address: editionAuthority.address,
      },
      number: 1,
    },
  });

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    pluginType: PluginType.Edition,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [revokeInstruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });
});

test('it can approve/revoke the plugin authority of other plugins as delegated owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const updateAuthorityAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    updateAuthority: updateAuthorityAddress.address,
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
        authority: { type: 'Owner' },
      }),
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthorityAddress.address },
    updateDelegate: {
      authority: {
        type: 'Owner',
      },
      additionalDelegates: [],
    },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });

  const editionAuthority = await generateKeyPairSigner();
  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    pluginType: PluginType.Edition,
    newAuthority: { __kind: 'Address', address: editionAuthority.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [approveInstruction], [owner, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthorityAddress.address },
    updateDelegate: {
      authority: {
        type: 'Owner',
      },
      additionalDelegates: [],
    },
    edition: {
      authority: {
        type: 'Address',
        address: editionAuthority.address,
      },
      number: 1,
    },
  });

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    pluginType: PluginType.Edition,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [revokeInstruction], [owner, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthorityAddress.address },
    updateDelegate: {
      authority: {
        type: 'Owner',
      },
      additionalDelegates: [],
    },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    },
  });
});

test('it can update an asset as delegated owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const updateAuthorityAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    updateAuthority: updateAuthorityAddress.address,
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
        authority: { type: 'Owner' },
      }),
    ],
    name: 'short',
    uri: 'https://short.com',
  });

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthorityAddress.address },
    updateDelegate: {
      authority: {
        type: 'Owner',
      },
      additionalDelegates: [],
    },
    name: 'short',
    uri: 'https://short.com',
  });

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [owner, payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthorityAddress.address },
    updateDelegate: {
      authority: {
        type: 'Owner',
      },
      additionalDelegates: [],
    },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('an updateDelegate can update a plugin on an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
        authority: { type: 'Address', address: updateDelegate.address },
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: {
          attributeList: [
            {
              key: '123',
              value: '456',
            },
          ],
        },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        {
          key: '123',
          value: '456',
        },
      ],
    },
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    plugin: createPlugin({
      type: 'Attributes',
      data: {
        attributeList: [
          {
            key: '789',
            value: '012',
          },
        ],
      },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        {
          key: '789',
          value: '012',
        },
      ],
    },
  });
});

test('an updateDelegate can update a plugin on an asset using delegated owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const updateAuthorityAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    updateAuthority: updateAuthorityAddress.address,
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
        authority: { type: 'Owner' },
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: {
          attributeList: [
            {
              key: '123',
              value: '456',
            },
          ],
        },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthorityAddress.address },
    updateDelegate: {
      authority: {
        type: 'Owner',
      },
      additionalDelegates: [],
    },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        {
          key: '123',
          value: '456',
        },
      ],
    },
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    plugin: createPlugin({
      type: 'Attributes',
      data: {
        attributeList: [
          {
            key: '789',
            value: '012',
          },
        ],
      },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [owner, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthorityAddress.address },
    updateDelegate: {
      authority: {
        type: 'Owner',
      },
      additionalDelegates: [],
    },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        {
          key: '789',
          value: '012',
        },
      ],
    },
  });
});

test('it can update an authority-managed plugin on an asset as additional delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    plugin: createPlugin({
      type: 'PermanentFreezeDelegate',
      data: { frozen: false },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentFreezeDelegate: {
      authority: { type: 'UpdateAuthority' },
      frozen: false,
    },
  });
});

test('it cannot update an authority-managed plugin on an asset as additional delegate if the plugin authority is not UpdateAuthority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        authority: { type: 'Address', address: address('11111111111111111111111111111111') },
        data: { frozen: true },
      }),
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    plugin: createPlugin({
      type: 'PermanentFreezeDelegate',
      data: { frozen: false },
    }),
  });

  const result = sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentFreezeDelegate: {
      authority: { type: 'Address', address: '11111111111111111111111111111111' },
      frozen: true,
    },
  });
});

test('it cannot update an owner-managed plugin on an asset as collection update additional delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    plugin: createPlugin({
      type: 'FreezeDelegate',
      data: { frozen: false },
    }),
  });

  const result = sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: { type: 'Owner' },
      frozen: true,
    },
  });
});

test('it can update an owner-managed plugin on an asset as collection update additional delegate if the plugin authority is UpdateAuthority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        authority: { type: 'UpdateAuthority' },
        data: { frozen: true },
      }),
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateDelegate,
    plugin: createPlugin({
      type: 'FreezeDelegate',
      data: { frozen: false },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [updateDelegate, payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: { type: 'UpdateAuthority' },
      frozen: false,
    },
  });
});
