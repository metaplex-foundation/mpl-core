import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getBurnV1Instruction,
  getTransferV1Instruction,
  getRemovePluginV1Instruction,
  getAddPluginV1Instruction,
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
  assertBurned,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can burn an assets as an owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
      }),
    ],
  });

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertBurned(t, rpc, asset.address);
});

test('it can burn an assets as a delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    payer: owner,
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
      }),
    ],
  });

  const transferInstruction = getTransferV1Instruction({
    asset: asset.address,
    payer: owner,
    authority: owner,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [transferInstruction],
    [owner]
  );

  const burnInstruction = getBurnV1Instruction({
    payer: owner,
    asset: asset.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [burnInstruction],
    [owner]
  );

  await assertBurned(t, rpc, asset.address);
});

test('it can burn an assets as a delegate for a collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const firstAssetOwner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const brandNewOwner = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    payer: firstAssetOwner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
      }),
    ],
  });

  const asset1 = await createAsset(rpc, payer, {
    payer: firstAssetOwner,
    authority: firstAssetOwner,
    owner: firstAssetOwner.address,
    collection: collection.address,
  });

  const asset2 = await createAsset(rpc, payer, {
    payer: firstAssetOwner,
    authority: firstAssetOwner,
    owner: firstAssetOwner.address,
    collection: collection.address,
  });

  // move asset #1 twice as a delegate for collection
  const transfer1Instruction = getTransferV1Instruction({
    asset: asset1.address,
    payer: firstAssetOwner,
    authority: firstAssetOwner,
    collection: collection.address,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [transfer1Instruction],
    [firstAssetOwner]
  );

  const transfer2Instruction = getTransferV1Instruction({
    asset: asset1.address,
    payer: firstAssetOwner,
    authority: newOwner,
    collection: collection.address,
    newOwner: brandNewOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [transfer2Instruction],
    [newOwner, firstAssetOwner]
  );

  // move asset #2 twice as a delegate for collection
  const transfer3Instruction = getTransferV1Instruction({
    asset: asset2.address,
    payer: firstAssetOwner,
    authority: firstAssetOwner,
    collection: collection.address,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [transfer3Instruction],
    [firstAssetOwner]
  );

  const transfer4Instruction = getTransferV1Instruction({
    asset: asset2.address,
    payer: firstAssetOwner,
    authority: newOwner,
    collection: collection.address,
    newOwner: brandNewOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [transfer4Instruction],
    [newOwner, firstAssetOwner]
  );

  const burn1Instruction = getBurnV1Instruction({
    payer: firstAssetOwner,
    asset: asset1.address,
    collection: collection.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [burn1Instruction],
    [firstAssetOwner]
  );

  const burn2Instruction = getBurnV1Instruction({
    payer: firstAssetOwner,
    asset: asset2.address,
    collection: collection.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [burn2Instruction],
    [firstAssetOwner]
  );

  await assertBurned(t, rpc, asset1.address);
  await assertBurned(t, rpc, asset2.address);
});

test('it can burn an asset which is the part of a collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const firstAssetOwner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const brandNewOwner = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    payer: firstAssetOwner,
  });

  const asset = await createAsset(rpc, payer, {
    payer: firstAssetOwner,
    authority: firstAssetOwner,
    owner: firstAssetOwner.address,
    collection: collection.address,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
      }),
    ],
  });

  // move asset #1 twice as a delegate for collection
  const transfer1Instruction = getTransferV1Instruction({
    asset: asset.address,
    payer: firstAssetOwner,
    authority: firstAssetOwner,
    collection: collection.address,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [transfer1Instruction],
    [firstAssetOwner]
  );

  const transfer2Instruction = getTransferV1Instruction({
    asset: asset.address,
    payer: firstAssetOwner,
    authority: newOwner,
    collection: collection.address,
    newOwner: brandNewOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [transfer2Instruction],
    [newOwner, firstAssetOwner]
  );

  const burnInstruction = getBurnV1Instruction({
    payer: firstAssetOwner,
    asset: asset.address,
    collection: collection.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [burnInstruction],
    [firstAssetOwner]
  );

  await assertBurned(t, rpc, asset.address);
});

test('it can remove permanent burn plugin if update authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
      }),
    ],
  });

  const removeInstruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.PermanentBurnDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [removeInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentBurnDelegate: undefined,
  });
});

test('it can add another plugin on asset with permanent burn plugin', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
      }),
    ],
  });

  const addInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'TransferDelegate',
      fields: [{}],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
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
