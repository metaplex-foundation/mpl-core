import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  PluginType,
  getAddPluginV1Instruction,
  getRemovePluginV1Instruction,
  getTransferV1Instruction,
} from '../../../src';
import { pluginAuthorityPair } from '../../../src/plugins';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createAssetWithCollection,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it cannot add permanentTransfer after creation', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, { owner });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    plugin: {
      __kind: 'PermanentTransferDelegate',
      fields: [{}],
    },
    payer: owner,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [owner]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentTransferDelegate: undefined,
  });
});

test('it can transfer an asset as the owner and not the delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const newOwner = await generateSignerWithSol(rpc);
  const brandNewOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
  });

  const instruction1 = getTransferV1Instruction({
    asset: asset.address,
    payer: owner,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction1], [
    owner,
  ]);

  const instruction2 = getTransferV1Instruction({
    asset: asset.address,
    payer: newOwner,
    newOwner: brandNewOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction2], [
    newOwner,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: brandNewOwner,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer an asset as the delegate and the owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer: owner,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    owner,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer an asset as not the owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const brandNewOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
  });

  const instruction1 = getTransferV1Instruction({
    asset: asset.address,
    payer: owner,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction1], [
    owner,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const instruction2 = getTransferV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    newOwner: brandNewOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction2], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: brandNewOwner,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer asset as the owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer: owner,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    owner,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer a collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer asset that is a part of a collection forever as a delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const brandNewOwner = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer);

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
    collection: collection.address,
  });

  const instruction1 = getTransferV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    collection: collection.address,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction1], [
    payer,
  ]);

  const instruction2 = getTransferV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    collection: collection.address,
    newOwner: brandNewOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction2], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: brandNewOwner,
    updateAuthority: { type: 'Collection', address: collection.address },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer multiple assets that is a part of a collection forever as a delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const firstAssetOwner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const brandNewOwner = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
  });

  const asset1 = await createAsset(rpc, payer, {
    owner: firstAssetOwner,
    collection: collection.address,
  });

  const asset2 = await createAsset(rpc, payer, {
    owner: firstAssetOwner,
    collection: collection.address,
  });

  // move asset #1 twice as a delegate for collection
  const instruction1 = getTransferV1Instruction({
    asset: asset1.address,
    payer,
    authority: payer,
    collection: collection.address,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction1], [
    payer,
  ]);

  const instruction2 = getTransferV1Instruction({
    asset: asset1.address,
    payer,
    authority: payer,
    collection: collection.address,
    newOwner: brandNewOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction2], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset1.address,
    owner: brandNewOwner,
    updateAuthority: { type: 'Collection', address: collection.address },
    permanentTransferDelegate: undefined,
  });

  // move asset #2 twice as a delegate for collection
  const instruction3 = getTransferV1Instruction({
    asset: asset2.address,
    payer,
    authority: payer,
    collection: collection.address,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction3], [
    payer,
  ]);

  const instruction4 = getTransferV1Instruction({
    asset: asset2.address,
    payer,
    authority: payer,
    collection: collection.address,
    newOwner: brandNewOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction4], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset2.address,
    owner: brandNewOwner,
    updateAuthority: { type: 'Collection', address: collection.address },
    permanentTransferDelegate: undefined,
  });

  await assertCollection(t, rpc, {
    currentSize: 2,
    numMinted: 2,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can remove permanent transfer plugin if collection update authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collectionAuth = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      authority: collectionAuth,
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentTransferDelegate',
          authority: { __kind: 'UpdateAuthority' },
        }),
      ],
    },
    {
      updateAuthority: collectionAuth,
    }
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    pluginType: PluginType.PermanentTransferDelegate,
    payer: collectionAuth,
    authority: collectionAuth,
    collection: collection.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    collectionAuth,
  ]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
    permanentTransferDelegate: undefined,
  });
});

test('it can permanent transfer using collection delegate authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const delegate = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentTransferDelegate',
          authority: { __kind: 'Address', address: delegate.address },
        }),
      ],
    }
  );

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer: delegate,
    authority: delegate,
    collection: collection.address,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    delegate,
  ]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    permanentTransferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
    },
  });
});

test('it can permanent transfer asset that is frozen as a delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const delegate = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: { __kind: 'Address', address: delegate.address },
      }),
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer: delegate,
    authority: delegate,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    delegate,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentTransferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
    },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it cannot transfer asset that is frozen with permanent transfer by owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const delegate = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: { __kind: 'Address', address: delegate.address },
      }),
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer: owner,
    newOwner: newOwner.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [owner]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentTransferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
    },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it can collection permanent transfer asset that is frozen as a collection delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const delegate = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      owner,
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: true },
        }),
      ],
    },
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentTransferDelegate',
          authority: { __kind: 'Address', address: delegate.address },
        }),
      ],
    }
  );

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer: delegate,
    authority: delegate,
    collection: collection.address,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    delegate,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Collection', address: collection.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it can collection permanent transfer asset that is frozen as a collection update auth', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const colAuth = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      owner,
      authority: colAuth,
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: true },
        }),
      ],
    },
    {
      updateAuthority: colAuth,
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentTransferDelegate',
          authority: { __kind: 'UpdateAuthority' },
        }),
      ],
    }
  );

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer: colAuth,
    authority: colAuth,
    collection: collection.address,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    colAuth,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Collection', address: collection.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it can add another plugin on asset with permanent transfer plugin', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
      }),
    ],
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    plugin: {
      __kind: 'TransferDelegate',
      fields: [{}],
    },
    payer,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});
