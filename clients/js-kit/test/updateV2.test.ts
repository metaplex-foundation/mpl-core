import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getUpdateV2Instruction,
  getAddCollectionPluginV1Instruction,
  getApproveCollectionPluginAuthorityV1Instruction,
  getAddPluginV1Instruction,
  getApprovePluginAuthorityV1Instruction,
} from '../src';
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
import { pluginAuthorityPair, updateAuthority, updateAuthorityToBase } from '../src/plugins';

test('it can update an asset to be larger', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('it cannot update an asset using asset as authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const myAsset = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    asset: myAsset,
    name: 'short',
    uri: 'https://short.com',
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    authority: myAsset,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [myAsset, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    name: 'short',
    uri: 'https://short.com',
  });
});

test('it can update an asset to be smaller', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: '',
    uri: '',
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    name: '',
    uri: '',
  });
});

test('it can update an asset with plugins to be larger', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    name: 'short',
    uri: 'https://short.com',
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it can update an asset with plugins to be smaller', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: '',
    uri: '',
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    name: '',
    uri: '',
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it can update an asset update authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);
  const newUpdateAuthority = await generateKeyPairSigner();

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Address', newUpdateAuthority.address)),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: newUpdateAuthority.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });
});

test('it cannot update an asset using wrong authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateAuth = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    updateAuthority: updateAuth.address,
  });

  const newUpdateAuthority = await generateKeyPairSigner();
  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Address', newUpdateAuthority.address)),
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
    owner: payer,
    updateAuthority: { type: 'Address', address: updateAuth.address },
  });
});

test('it cannot use an invalid system program for assets', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);
  const fakeSystemProgram = await generateKeyPairSigner();

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
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
  const asset = await createAsset(rpc, payer);
  const fakeLogWrapper = await generateKeyPairSigner();

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
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
  const collection = await createCollection(rpc, payer);
  const fakeSystemProgram = await generateKeyPairSigner();

  const instruction = getUpdateV2Instruction({
    asset: collection.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
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
  const collection = await createCollection(rpc, payer);
  const fakeLogWrapper = await generateKeyPairSigner();

  const instruction = getUpdateV2Instruction({
    asset: collection.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
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

test('it can remove an asset from a collection using update', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 1,
    numMinted: 1,
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Address', payer.address)),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 0,
    numMinted: 1,
  });
});

test('it cannot remove an asset from a collection if not collection update auth', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collectionAuthority = await generateSignerWithSol(rpc);
  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    { authority: collectionAuthority },
    { updateAuthority: collectionAuthority }
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: collectionAuthority.address,
    currentSize: 1,
    numMinted: 1,
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Address', payer.address)),
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
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: collectionAuthority.address,
    currentSize: 1,
    numMinted: 1,
  });
});

test('it cannot remove an asset from a collection when missing collection account', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 1,
    numMinted: 1,
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Address', payer.address)),
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
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 1,
    numMinted: 1,
  });
});

test('it cannot remove an asset from a collection when using incorrect collection account', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  const randomCollection = await createCollection(rpc, payer);

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    collection: randomCollection.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Address', payer.address)),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can add asset to collection using update', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);
  const collection = await createCollection(rpc, payer);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 0,
    numMinted: 0,
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', collection.address)),
    newCollection: collection.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 1,
    numMinted: 0,
  });
});

test('it cannot add asset to collection when missing collection account', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);
  const collection = await createCollection(rpc, payer);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 0,
    numMinted: 0,
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', collection.address)),
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
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 0,
    numMinted: 0,
  });
});

test('it cannot add asset to collection when using incorrect collection account', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);
  const collection = await createCollection(rpc, payer);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const wrongCollection = await generateKeyPairSigner();
  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', collection.address)),
    newCollection: wrongCollection.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot add asset to collection using only new collection authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);
  const newCollectionAuthority = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer, {
    updateAuthority: newCollectionAuthority,
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', collection.address)),
    newCollection: collection.address,
    authority: newCollectionAuthority,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [newCollectionAuthority, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot add asset to collection if not both asset and collection auth', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const assetAuthority = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer, { updateAuthority: assetAuthority.address });

  const collectionAuthority = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer, {
    updateAuthority: collectionAuthority,
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: assetAuthority.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: collectionAuthority.address,
    currentSize: 0,
    numMinted: 0,
  });

  // Attempt to update using asset authority.
  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', collection.address)),
    newCollection: collection.address,
    authority: assetAuthority,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [assetAuthority, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: assetAuthority.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: collectionAuthority.address,
    currentSize: 0,
    numMinted: 0,
  });

  // Attempt to update using collection authority.
  const instruction2 = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', collection.address)),
    newCollection: collection.address,
    authority: collectionAuthority,
  });

  const result2 = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction2],
    [collectionAuthority, payer]
  );

  await t.throwsAsync(result2);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: assetAuthority.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: collectionAuthority.address,
    currentSize: 0,
    numMinted: 0,
  });
});

test('it can change an asset collection using same update authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collectionAuthority = await generateSignerWithSol(rpc);
  const { asset, collection: originalCollection } =
    await createAssetWithCollection(
      rpc,
      payer,
      { authority: collectionAuthority },
      { updateAuthority: collectionAuthority }
    );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: {
      type: 'Collection',
      address: originalCollection.address,
    },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: originalCollection.address,
    updateAuthority: collectionAuthority.address,
    currentSize: 1,
    numMinted: 1,
  });

  const newCollection = await createCollection(rpc, payer, {
    updateAuthority: collectionAuthority,
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.address,
    updateAuthority: collectionAuthority.address,
    currentSize: 0,
    numMinted: 0,
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    collection: originalCollection.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', newCollection.address)),
    newCollection: newCollection.address,
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [collectionAuthority, payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: newCollection.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: originalCollection.address,
    updateAuthority: collectionAuthority.address,
    currentSize: 0,
    numMinted: 1,
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.address,
    updateAuthority: collectionAuthority.address,
    currentSize: 1,
    numMinted: 0,
  });
});

test('it cannot change an asset collection if not both asset and collection auth', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const originalCollectionAuthority = await generateSignerWithSol(rpc);
  const { asset, collection: originalCollection } =
    await createAssetWithCollection(
      rpc,
      payer,
      { authority: originalCollectionAuthority },
      { updateAuthority: originalCollectionAuthority }
    );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: {
      type: 'Collection',
      address: originalCollection.address,
    },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: originalCollection.address,
    updateAuthority: originalCollectionAuthority.address,
    currentSize: 1,
    numMinted: 1,
  });

  const newCollectionAuthority = await generateSignerWithSol(rpc);
  const newCollection = await createCollection(rpc, payer, {
    updateAuthority: newCollectionAuthority,
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.address,
    updateAuthority: newCollectionAuthority.address,
    currentSize: 0,
    numMinted: 0,
  });

  // Attempt to update using original collection authority.
  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    collection: originalCollection.address,
    payer,
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', newCollection.address)),
    newCollection: newCollection.address,
    authority: originalCollectionAuthority,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [originalCollectionAuthority, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: {
      type: 'Collection',
      address: originalCollection.address,
    },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: originalCollection.address,
    updateAuthority: originalCollectionAuthority.address,
    currentSize: 1,
    numMinted: 1,
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.address,
    updateAuthority: newCollectionAuthority.address,
    currentSize: 0,
    numMinted: 0,
  });

  // Attempt to update using new collection authority.
  const instruction2 = getUpdateV2Instruction({
    asset: asset.address,
    collection: originalCollection.address,
    payer,
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', newCollection.address)),
    newCollection: newCollection.address,
    authority: newCollectionAuthority,
  });

  const result2 = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction2],
    [newCollectionAuthority, payer]
  );

  await t.throwsAsync(result2);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: {
      type: 'Collection',
      address: originalCollection.address,
    },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: originalCollection.address,
    updateAuthority: originalCollectionAuthority.address,
    currentSize: 1,
    numMinted: 1,
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.address,
    updateAuthority: newCollectionAuthority.address,
    currentSize: 0,
    numMinted: 0,
  });
});

test('it can change an asset collection using delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const originalCollectionAuthority = await generateSignerWithSol(rpc);
  const { asset, collection: originalCollection } =
    await createAssetWithCollection(
      rpc,
      payer,
      { authority: originalCollectionAuthority },
      { updateAuthority: originalCollectionAuthority }
    );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: {
      type: 'Collection',
      address: originalCollection.address,
    },
  });

  const newCollectionAuthority = await generateSignerWithSol(rpc);
  const newCollection = await createCollection(rpc, payer, {
    updateAuthority: newCollectionAuthority,
  });

  const addPluginInstruction = getAddCollectionPluginV1Instruction({
    collection: newCollection.address,
    payer,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{ additionalDelegates: [] }],
    },
    authority: newCollectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addPluginInstruction],
    [newCollectionAuthority, payer]
  );

  const approveInstruction = getApproveCollectionPluginAuthorityV1Instruction({
    collection: newCollection.address,
    payer,
    plugin: { __kind: 'UpdateDelegate' },
    newAuthority: {
      __kind: 'Address',
      fields: [originalCollectionAuthority.address],
    },
    authority: newCollectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [approveInstruction],
    [newCollectionAuthority, payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.address,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: originalCollectionAuthority.address,
      },
      additionalDelegates: [],
    },
    updateAuthority: newCollectionAuthority.address,
    currentSize: 0,
    numMinted: 0,
  });

  const updateInstruction = getUpdateV2Instruction({
    asset: asset.address,
    collection: originalCollection.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', newCollection.address)),
    newCollection: newCollection.address,
    authority: originalCollectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [originalCollectionAuthority, payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: newCollection.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.address,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: originalCollectionAuthority.address,
      },
      additionalDelegates: [],
    },
    updateAuthority: newCollectionAuthority.address,
    currentSize: 1,
    numMinted: 0,
  });
});

test('it can change an asset collection using same update authority (delegate exists but not used)', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collectionAuthority = await generateSignerWithSol(rpc);
  const { asset, collection: originalCollection } =
    await createAssetWithCollection(
      rpc,
      payer,
      { authority: collectionAuthority },
      { updateAuthority: collectionAuthority }
    );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: {
      type: 'Collection',
      address: originalCollection.address,
    },
  });

  const newCollection = await createCollection(rpc, payer, {
    updateAuthority: collectionAuthority,
  });

  const addPluginInstruction = getAddCollectionPluginV1Instruction({
    collection: newCollection.address,
    payer,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{ additionalDelegates: [] }],
    },
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addPluginInstruction],
    [collectionAuthority, payer]
  );

  const updateDelegate = await generateKeyPairSigner();
  const approveInstruction = getApproveCollectionPluginAuthorityV1Instruction({
    collection: newCollection.address,
    payer,
    plugin: { __kind: 'UpdateDelegate' },
    newAuthority: {
      __kind: 'Address',
      fields: [updateDelegate.address],
    },
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [approveInstruction],
    [collectionAuthority, payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.address,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    updateAuthority: collectionAuthority.address,
    currentSize: 0,
    numMinted: 0,
  });

  const updateInstruction = getUpdateV2Instruction({
    asset: asset.address,
    collection: originalCollection.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', newCollection.address)),
    newCollection: newCollection.address,
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [collectionAuthority, payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: newCollection.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: newCollection.address,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    updateAuthority: collectionAuthority.address,
    currentSize: 1,
    numMinted: 0,
  });
});

test('it can remove an asset from collection as Collection authority with update delegate on asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collectionAuthority = await generateSignerWithSol(rpc);
  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    { authority: collectionAuthority },
    { updateAuthority: collectionAuthority }
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: collectionAuthority.address,
    currentSize: 1,
    numMinted: 1,
  });

  const addPluginInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{ additionalDelegates: [] }],
    },
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addPluginInstruction],
    [collectionAuthority, payer]
  );

  const updateDelegate = await generateKeyPairSigner();
  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: { __kind: 'UpdateDelegate' },
    newAuthority: {
      __kind: 'Address',
      fields: [updateDelegate.address],
    },
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [approveInstruction],
    [collectionAuthority, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
  });

  const updateInstruction = getUpdateV2Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Address', payer.address)),
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [collectionAuthority, payer]
  );

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

test('it can remove an asset from collection using update delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collectionAuthority = await generateSignerWithSol(rpc);
  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    { authority: collectionAuthority },
    { updateAuthority: collectionAuthority }
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: collectionAuthority.address,
    currentSize: 1,
    numMinted: 1,
  });

  const addPluginInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{ additionalDelegates: [] }],
    },
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addPluginInstruction],
    [collectionAuthority, payer]
  );

  const updateDelegate = await generateSignerWithSol(rpc);
  const approveInstruction = getApproveCollectionPluginAuthorityV1Instruction({
    collection: collection.address,
    payer,
    plugin: { __kind: 'UpdateDelegate' },
    newAuthority: {
      __kind: 'Address',
      fields: [updateDelegate.address],
    },
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [approveInstruction],
    [collectionAuthority, payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
    updateAuthority: collectionAuthority.address,
    currentSize: 1,
    numMinted: 1,
  });

  const updateInstruction = getUpdateV2Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Address', payer.address)),
    authority: updateDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [updateDelegate, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it can remove an asset from collection using additional update delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collectionAuthority = await generateSignerWithSol(rpc);
  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    { authority: collectionAuthority },
    { updateAuthority: collectionAuthority }
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  const additionalDelegate = await generateSignerWithSol(rpc);
  const addPluginInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{ additionalDelegates: [additionalDelegate.address] }],
    },
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addPluginInstruction],
    [collectionAuthority, payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [additionalDelegate.address],
    },
    updateAuthority: collectionAuthority.address,
    currentSize: 1,
    numMinted: 1,
  });

  const updateInstruction = getUpdateV2Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Address', payer.address)),
    authority: additionalDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [additionalDelegate, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot remove an asset from collection using update delegate on the asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collectionAuthority = await generateSignerWithSol(rpc);
  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    { authority: collectionAuthority },
    { updateAuthority: collectionAuthority }
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: collectionAuthority.address,
    currentSize: 1,
    numMinted: 1,
  });

  const addPluginInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{ additionalDelegates: [] }],
    },
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addPluginInstruction],
    [collectionAuthority, payer]
  );

  const updateDelegate = await generateSignerWithSol(rpc);
  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: { __kind: 'UpdateDelegate' },
    newAuthority: {
      __kind: 'Address',
      fields: [updateDelegate.address],
    },
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [approveInstruction],
    [collectionAuthority, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegate.address,
      },
      additionalDelegates: [],
    },
  });

  const updateInstruction = getUpdateV2Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Address', payer.address)),
    authority: updateDelegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [updateDelegate, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot remove an asset from collection using additional update delegate on the asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collectionAuthority = await generateSignerWithSol(rpc);
  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    { authority: collectionAuthority },
    { updateAuthority: collectionAuthority }
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  const additionalDelegate = await generateSignerWithSol(rpc);
  const addPluginInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{ additionalDelegates: [additionalDelegate.address] }],
    },
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addPluginInstruction],
    [collectionAuthority, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [additionalDelegate.address],
    },
  });

  const updateInstruction = getUpdateV2Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Address', payer.address)),
    authority: additionalDelegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [additionalDelegate, payer]
  );

  await t.throwsAsync(result);
});

test('it can add asset to collection using additional update delegate on new collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const collectionAuthority = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer, {
    updateAuthority: collectionAuthority,
  });

  const addPluginInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'UpdateDelegate',
      fields: [{ additionalDelegates: [payer.address] }],
    },
    authority: collectionAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addPluginInstruction],
    [collectionAuthority, payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [payer.address],
    },
    updateAuthority: collectionAuthority.address,
    currentSize: 0,
    numMinted: 0,
  });

  const updateInstruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', collection.address)),
    newCollection: collection.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });
});

test('it cannot add asset to collection if new collection contains permanent freeze delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);
  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 0,
    numMinted: 0,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', collection.address)),
    newCollection: collection.address,
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
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 0,
    numMinted: 0,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('it cannot add asset to collection if new collection contains permanent transfer delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);
  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        data: {},
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 0,
    numMinted: 0,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', collection.address)),
    newCollection: collection.address,
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
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 0,
    numMinted: 0,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it cannot add asset to collection if new collection contains permanent burn delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);
  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
        data: {},
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 0,
    numMinted: 0,
    permanentBurnDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const instruction = getUpdateV2Instruction({
    asset: asset.address,
    payer,
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
    newUpdateAuthority: updateAuthorityToBase(updateAuthority('Collection', collection.address)),
    newCollection: collection.address,
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
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 0,
    numMinted: 0,
    permanentBurnDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});
