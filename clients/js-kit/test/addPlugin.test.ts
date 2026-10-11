import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
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

test('it can add a plugin to an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
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
      frozen: false,
    },
  });
});

test('it can add an authority managed plugin to an asset via update auth', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateAuth = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    updateAuthority: updateAuth,
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'UpdateDelegate',
    }),
    authority: updateAuth,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    updateAuth,
    payer,
  ]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: updateAuth.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });
});

test('it can add a plugin to an asset with a different authority than the default', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
    initAuthority: { __kind: 'Address', address: delegateAddress.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
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
});

test('it can add plugin to asset with a plugin', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
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
        type: 'Owner',
      },
      frozen: false,
    },
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'TransferDelegate' }),
    initAuthority: { __kind: 'Address', address: delegate.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
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
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
    },
  });
});

test('it can add a plugin to a collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {});

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
  });

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [{ address: payer.address, percentage: 100 }],
        ruleSet: { __kind: 'None' },
      },
    }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    payer,
  ]);

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: payer.address, percentage: 100 }],
      ruleSet: { __kind: 'None' },
    },
  });
});

test('it cannot add an owner-managed plugin to a collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {});

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
  });

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can add an authority-managed plugin to an asset via delegate authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          authority: { __kind: 'Address', address: delegate.address },
        }),
      ],
    }
  );

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          {
            address: payer.address,
            percentage: 100,
          },
        ],
        ruleSet: { __kind: 'None' },
      },
    }),
    authority: delegate,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    delegate,
    payer,
  ]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [
        {
          address: payer.address,
          percentage: 100,
        },
      ],
      ruleSet: { __kind: 'None' },
    },
  });
});

test('it can add an authority-managed plugin to an asset with the collection update authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collectionAuth = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    { authority: collectionAuth },
    { updateAuthority: collectionAuth }
  );

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          {
            address: payer.address,
            percentage: 100,
          },
        ],
        ruleSet: { __kind: 'None' },
      },
    }),
    authority: collectionAuth,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    collectionAuth,
    payer,
  ]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [
        {
          address: payer.address,
          percentage: 100,
        },
      ],
      ruleSet: { __kind: 'None' },
    },
  });
});

test('it cannot add a owner-managed plugin to an asset via delegate authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();
  const collectionAuth = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      authority: collectionAuth,
    },
    {
      updateAuthority: collectionAuth,
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          authority: { __kind: 'Address', address: delegate.address },
        }),
      ],
    }
  );

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: createPlugin({
      type: 'FreezeDelegate',
      data: {
        frozen: false,
      },
    }),
    authority: delegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [delegate, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot add authority-managed plugin to an asset by owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(rpc, payer, {
    owner: owner.address,
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    authority: owner,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          {
            address: payer.address,
            percentage: 100,
          },
        ],
        ruleSet: { __kind: 'None' },
      },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [owner, payer]
  );

  await t.throwsAsync(result);
});

test('it can add a plugin to a collection with a plugin', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
          creators: [
            {
              address: payer.address,
              percentage: 100,
            },
          ],
          ruleSet: { __kind: 'None' },
        },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [
        {
          address: payer.address,
          percentage: 100,
        },
      ],
      ruleSet: { __kind: 'None' },
    },
  });

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: createPlugin({
      type: 'UpdateDelegate',
    }),
    initAuthority: { __kind: 'Address', address: delegate.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    payer,
  ]);

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [
        {
          address: payer.address,
          percentage: 100,
        },
      ],
      ruleSet: { __kind: 'None' },
    },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
      additionalDelegates: [],
    },
  });
});

test('it can add a plugin to an asset that is part of a collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it cannot add a plugin to an asset if the collection is wrong', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot add a plugin to an asset if the collection is missing', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const wrongCollection = await createCollection(rpc, payer);
  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    collection: wrongCollection.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid system program for assets', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);
  const fakeSystemProgram = await generateKeyPairSigner();

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
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

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
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

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
  });

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
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

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
  });

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: createPlugin({ type: 'UpdateDelegate' }),
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
