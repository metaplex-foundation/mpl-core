import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getAddCollectionPluginV1Instruction,
  getApproveCollectionPluginAuthorityV1Instruction,
  getUpdateCollectionPluginV1Instruction,
  getUpdateV1Instruction,
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
  assertCollection,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can create a new asset with a collection if it is the collection updateDelegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
      }),
    ],
  });

  const approveInstruction = getApproveCollectionPluginAuthorityV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: { __kind: 'Address', address: updateDelegate.address },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [approveInstruction],
    [payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
  });

  const owner = await generateKeyPairSigner();
  const asset = await createAsset(rpc, updateDelegate, {
    collection: collection.address,
    owner: owner.address,
    authority: updateDelegate,
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Collection', address: collection.address },
  });
});

test('it can create a new asset with a collection if it is a collection updateDelegate additionalDelegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.address],
    },
  });

  const owner = await generateKeyPairSigner();
  const asset = await createAsset(rpc, updateDelegate, {
    collection: collection.address,
    owner: owner.address,
    authority: updateDelegate,
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Collection', address: collection.address },
  });
});

test('it can add updateDelegate to collection and then approve', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer);

  const addPluginInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{ additionalDelegates: [] }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addPluginInstruction],
    [payer]
  );

  const approveInstruction = getApproveCollectionPluginAuthorityV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: { __kind: 'Address', address: updateDelegate.address },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [approveInstruction],
    [payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
  });
});

test('it can create a collection with updateDelegate with additional delegates', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    updateDelegate: {
      authority: { type: 'UpdateAuthority' },
      additionalDelegates: [updateDelegate.address],
    },
  });
});

test('it can add updateDelegate to collection with additional delegates', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer);
  const updateDelegate = await generateKeyPairSigner();

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{ additionalDelegates: [updateDelegate.address] }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    updateDelegate: {
      authority: { type: 'UpdateAuthority' },
      additionalDelegates: [updateDelegate.address],
    },
  });
});

test('it can update updateDelegate on collection with additional delegates', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer);
  const updateDelegate = await generateKeyPairSigner();

  const addInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{ additionalDelegates: [] }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addInstruction],
    [payer]
  );

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
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{ additionalDelegates: [updateDelegate.address] }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.address],
    },
  });
});

test('an updateDelegate on collection can update an asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
      }),
    ],
  });

  const approveInstruction = getApproveCollectionPluginAuthorityV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: { __kind: 'Address', address: updateDelegate.address },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [approveInstruction],
    [payer]
  );

  const owner = await generateKeyPairSigner();
  const asset = await createAsset(rpc, updateDelegate, {
    collection: collection.address,
    owner: owner.address,
    authority: updateDelegate,
    name: 'short',
    uri: 'https://short.com',
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Collection', address: collection.address },
    name: 'short',
    uri: 'https://short.com',
  });

  const updateInstruction = getUpdateV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer: updateDelegate,
    authority: updateDelegate,
    newName: { __option: 'Some', value: 'Test Bread 2' },
    newUri: { __option: 'Some', value: 'https://example.com/bread2' },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [updateDelegate]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Collection', address: collection.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('an updateDelegate additionalDelegate on collection can update an asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateDelegate = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [updateDelegate.address] },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [updateDelegate.address],
    },
  });

  const owner = await generateKeyPairSigner();
  const asset = await createAsset(rpc, updateDelegate, {
    collection: collection.address,
    owner: owner.address,
    authority: updateDelegate,
    name: 'short',
    uri: 'https://short.com',
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Collection', address: collection.address },
    name: 'short',
    uri: 'https://short.com',
  });

  const updateInstruction = getUpdateV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer: updateDelegate,
    authority: updateDelegate,
    newName: { __option: 'Some', value: 'Test Bread 2' },
    newUri: { __option: 'Some', value: 'https://example.com/bread2' },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [updateDelegate]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Collection', address: collection.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});
