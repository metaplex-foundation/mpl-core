import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getUpdatePluginV1Instruction,
  getUpdateCollectionPluginV1Instruction,
  getAddPluginV1Instruction,
  getAddCollectionPluginV1Instruction,
} from '../src';
import { pluginAuthorityPair, createPlugin } from '../src/plugins';
import {
  createAsset,
  createAssetWithCollection,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  assertCollection,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  sendAndConfirmInstructions,
} from './_setup';

test('it cannot use an invalid system program for assets', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);
  const fakeSystemProgram = await generateKeyPairSigner();

  const addInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addInstruction], [
    payer,
  ]);

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

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
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

  const addInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addInstruction], [
    payer,
  ]);

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

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
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

  const addInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addInstruction], [
    payer,
  ]);

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });

  const updateInstruction = getUpdateCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
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

  const addInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addInstruction], [
    payer,
  ]);

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });

  const updateInstruction = getUpdateCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid collection to update a plugin on an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const assetAuth = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    updateAuthority: assetAuth.address,
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'initial' }] },
      }),
    ],
  });

  const collectionAuth = await generateKeyPairSigner();
  const wrongCollection = await createCollection(rpc, payer, {
    updateAuthority: collectionAuth.address,
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: assetAuth.address },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'initial' }],
    },
  });

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    collection: wrongCollection.address,
    plugin: createPlugin({
      type: 'Attributes',
      data: {
        attributeList: [{ key: 'key', value: 'updated' }],
      },
    }),
    authority: collectionAuth,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
    [collectionAuth, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: assetAuth.address },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'initial' }],
    },
  });
});

test('it cannot update a plugin with Owner authority on an Asset if a plugin with UpdateAuthority authority is present on a Collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    name: 'test',
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  const randomAuthority = await generateKeyPairSigner();

  const { asset } = await createAssetWithCollection(
    rpc,
    payer,
    {
      name: 'test',
      collection: collection.address,
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: false },
          authority: { __kind: 'Address', address: randomAuthority.address },
        }),
      ],
    },
    {}
  );

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    collection: collection.address,
    plugin: createPlugin({
      type: 'FreezeDelegate',
      data: {
        frozen: false,
      },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can update a plugin with Owner authority on an Asset if a plugin with UpdateAuthority authority is present on a Collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    ...DEFAULT_COLLECTION,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  const { asset } = await createAssetWithCollection(
    rpc,
    payer,
    {
      ...DEFAULT_ASSET,
      collection: collection.address,
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: false },
          authority: { __kind: 'UpdateAuthority' },
        }),
      ],
    },
    {}
  );

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    collection: collection.address,
    plugin: createPlugin({
      type: 'FreezeDelegate',
      data: {
        frozen: false,
      },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [updateInstruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
    freezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('it cannot update a plugin with Owner authority on an Asset if a plugin with UpdateAuthority authority is present', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const randomAuthority = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: { __kind: 'Address', address: randomAuthority.address },
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
  });

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'FreezeDelegate',
      data: {
        frozen: false,
      },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can update a plugin with Owner authority on an Asset if a plugin with UpdateAuthority authority is present', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    ...DEFAULT_ASSET,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: { __kind: 'UpdateAuthority' },
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
  });

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'FreezeDelegate',
      data: {
        frozen: false,
      },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [updateInstruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });
});

test('it cannot update a plugin with UpdateAuthority authority on an Asset if a plugin with Owner authority is present', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: { __kind: 'Owner' },
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
  });

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    plugin: createPlugin({
      type: 'Attributes',
      data: { attributeList: [{ key: 'key2', value: 'value2' }] },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
    [owner, payer]
  );

  await t.throwsAsync(result);
});

test('it can update a plugin with UpdateAuthority authority on an Asset if a plugin with Owner authority is present', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    ...DEFAULT_ASSET,
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: { __kind: 'Owner' },
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
        authority: { __kind: 'Owner' },
      }),
    ],
  });

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    plugin: createPlugin({
      type: 'Attributes',
      data: { attributeList: [{ key: 'key2', value: 'value2' }] },
    }),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
    [owner, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
    attributes: {
      authority: {
        type: 'Owner',
      },
      attributeList: [{ key: 'key2', value: 'value2' }],
    },
  });
});
