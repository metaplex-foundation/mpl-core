import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  PluginType,
  fetchAssetV1,
  getRemovePluginV1Instruction,
  getRemoveCollectionPluginV1Instruction,
  fetchCollectionV1,
  getTransferV1Instruction,
} from '../src';
import { pluginAuthorityPair, pluginAuthority } from '../src/plugins';
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
} from './_setup';

test('it can remove a plugin from an asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer.address,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  const asset2 = await fetchAssetV1(rpc, asset.address);

  t.is((asset2.data as Record<string, unknown>).freezeDelegate, undefined);
});

test('it cannot remove an owner plugin from an asset if not the owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const attacker = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    authority: attacker,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [attacker, payer]
  );

  await t.throwsAsync(result);
});

test('it can remove authority managed plugin from collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
      }),
    ],
  });

  const instruction = getRemoveCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  const collection2 = await fetchCollectionV1(rpc, collection.address);

  t.is((collection2.data as Record<string, unknown>).updateDelegate, undefined);
});

test('it can remove authority managed plugin from asset in collection using update auth', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateAuth = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            creators: [{ address: payer.address, percentage: 100 }],
            basisPoints: 5,
            ruleSet: { __kind: 'None' },
          },
        }),
      ],
      updateAuthority: updateAuth,
    },
    {
      updateAuthority: updateAuth,
    }
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: updateAuth.address,
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.Royalties,
    authority: updateAuth,
    collection: collection.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [updateAuth, payer]
  );

  const asset2 = await fetchAssetV1(rpc, asset.address);

  t.is((asset2.data as Record<string, unknown>).royalties, undefined);
});

test('it can remove authority managed plugin from asset not in collection using update auth', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const assetAuth = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: assetAuth.address },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.Attributes,
    authority: assetAuth,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [assetAuth, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: assetAuth.address },
    attributes: undefined,
  });
});

test('it can remove authority managed plugin from collection using delegate auth', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            creators: [{ address: payer.address, percentage: 100 }],
            basisPoints: 5,
            ruleSet: { __kind: 'None' },
          },
        }),
      ],
    },
    {
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          authority: pluginAuthority('Address', { address: delegate.address }),
        }),
      ],
    }
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.Royalties,
    authority: delegate,
    collection: collection.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [delegate, payer]
  );

  const asset2 = await fetchAssetV1(rpc, asset.address);

  t.is((asset2.data as Record<string, unknown>).royalties, undefined);
});

test('it cannot remove owner managed plugin if the delegate authority is not owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: pluginAuthority('Address', { address: delegate.address }),
      }),
    ],
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    authority: delegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [delegate, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer.address,
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
      frozen: false,
    },
  });
});

test('it cannot remove authority managed plugin if the delegate authority is not update authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        authority: pluginAuthority('Address', { address: delegate.address }),
        data: {
          creators: [{ address: payer.address, percentage: 100 }],
          basisPoints: 5,
          ruleSet: { __kind: 'None' },
        },
      }),
    ],
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.Royalties,
    authority: delegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [delegate, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer.address,
    royalties: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
      creators: [{ address: payer.address, percentage: 100 }],
      basisPoints: 5,
      ruleSet: { type: 'None' },
    },
  });
});

test('it cannot remove authority managed collection plugin if the delegate authority is not update authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        authority: pluginAuthority('Address', { address: delegate.address }),
        data: {
          creators: [{ address: payer.address, percentage: 100 }],
          basisPoints: 5,
          ruleSet: { __kind: 'None' },
        },
      }),
    ],
  });

  const instruction = getRemoveCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.Royalties,
    authority: delegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [delegate, payer]
  );

  await t.throwsAsync(result);

  await assertCollection(t, rpc, {
    collection: collection.address,
    royalties: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
      creators: [{ address: payer.address, percentage: 100 }],
      basisPoints: 5,
      ruleSet: { type: 'None' },
    },
  });
});

test('it can remove a plugin from asset with existing plugins', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
      pluginAuthorityPair({
        type: 'TransferDelegate',
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  const instruction1 = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction1],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  const instruction2 = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.BurnDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction2],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it cannot remove a plugin from a frozen asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
      pluginAuthorityPair({
        type: 'TransferDelegate',
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.BurnDelegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it cannot remove a plugin from an asset with a frozen collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      plugins: [
        pluginAuthorityPair({
          type: 'TransferDelegate',
        }),
        pluginAuthorityPair({
          type: 'BurnDelegate',
        }),
      ],
    },
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: {
            frozen: true,
          },
        }),
      ],
    }
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Collection', address: collection.address },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.TransferDelegate,
    collection: collection.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid system program for assets', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const fakeSystemProgram = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer.address,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid noop program for assets', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const fakeLogWrapper = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer.address,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid system program for collections', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const fakeSystemProgram = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    plugins: [pluginAuthorityPair({ type: 'UpdateDelegate' })],
  });

  await assertCollection(t, rpc, {
    collection: collection.address,
  });

  const instruction = getRemoveCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid noop program for collections', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const fakeLogWrapper = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    plugins: [pluginAuthorityPair({ type: 'UpdateDelegate' })],
  });

  await assertCollection(t, rpc, {
    collection: collection.address,
  });

  const instruction = getRemoveCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot remove an authority managed plugin from an asset if not the authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const assetAuth = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: assetAuth.address },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.Attributes,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: assetAuth.address },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });
});

test('it cannot use an invalid collection to remove a plugin on an asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const assetAuth = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
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
    owner: payer.address,
    updateAuthority: { type: 'Address', address: assetAuth.address },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    collection: wrongCollection.address,
    payer,
    pluginType: PluginType.Attributes,
    authority: collectionAuth,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [collectionAuth, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: assetAuth.address },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });
});

test('it cannot remove an authority managed plugin when the authority is None', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const assetAuth = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
        authority: pluginAuthority('None'),
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: assetAuth.address },
    attributes: {
      authority: {
        type: 'None',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: assetAuth,
    pluginType: PluginType.Attributes,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [assetAuth, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot remove an owner managed plugin when the authority is None', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const assetAuth = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: pluginAuthority('None'),
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: assetAuth.address },
    transferDelegate: {
      authority: {
        type: 'None',
      },
    },
  });

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.TransferDelegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can remove an owner managed plugin from an asset when the authority is None, after transferring', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const assetAuth = await generateKeyPairSigner();
  const newOwner = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: pluginAuthority('None'),
      }),
    ],
  });

  const transferInstruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [transferInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: newOwner.address,
    updateAuthority: { type: 'Address', address: assetAuth.address },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  const removeInstruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: newOwner,
    pluginType: PluginType.TransferDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [removeInstruction],
    [newOwner, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: newOwner.address,
    updateAuthority: { type: 'Address', address: assetAuth.address },
  });
});

test('it cannot remove an authority managed plugin from an asset when the authority is None, after transferring', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const assetAuth = await generateKeyPairSigner();
  const newOwner = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: pluginAuthority('None'),
      }),
    ],
  });

  const transferInstruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [transferInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: newOwner.address,
    updateAuthority: { type: 'Address', address: assetAuth.address },
    permanentTransferDelegate: {
      authority: {
        type: 'None',
      },
    },
  });

  const removeInstruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: newOwner,
    pluginType: PluginType.PermanentTransferDelegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [removeInstruction],
    [newOwner, payer]
  );

  await t.throwsAsync(result);
});
