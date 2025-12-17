import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getCreateV1Instruction,
  getAddPluginV1Instruction,
  getUpdatePluginV1Instruction,
  getRemovePluginV1Instruction,
  fetchAssetV1,
  PluginType,
  Key,
} from '../../../src';
import {
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can create asset with PermanentFreezeExecute plugin', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetSigner = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset: assetSigner,
    payer,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        plugin: {
          __kind: 'PermanentFreezeExecute',
          fields: [{ frozen: true }],
        },
        authority: { __option: 'None' },
      },
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [assetSigner, payer]
  );

  const asset = await fetchAssetV1(rpc, assetSigner.address);

  t.is(asset.data.key, Key.AssetV1);
  t.is(asset.data.owner, payer.address);
  t.is(asset.data.name, 'Test Asset');
});

test('it cannot add PermanentFreezeExecute after creation', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, { owner: owner.address });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'PermanentFreezeExecute',
      fields: [{ frozen: true }],
    },
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
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it can update PermanentFreezeExecute plugin to unfreeze', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetSigner = await generateKeyPairSigner();

  const createInstruction = getCreateV1Instruction({
    asset: assetSigner,
    payer,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        plugin: {
          __kind: 'PermanentFreezeExecute',
          fields: [{ frozen: true }],
        },
        authority: { __option: 'None' },
      },
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [createInstruction],
    [assetSigner, payer]
  );

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: assetSigner.address,
    payer,
    plugin: {
      __kind: 'PermanentFreezeExecute',
      fields: [{ frozen: false }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [payer]
  );

  const asset = await fetchAssetV1(rpc, assetSigner.address);
  t.is(asset.data.key, Key.AssetV1);
});

test('it cannot remove PermanentFreezeExecute plugin if frozen', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetSigner = await generateKeyPairSigner();

  const createInstruction = getCreateV1Instruction({
    asset: assetSigner,
    payer,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        plugin: {
          __kind: 'PermanentFreezeExecute',
          fields: [{ frozen: true }],
        },
        authority: { __option: 'None' },
      },
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [createInstruction],
    [assetSigner, payer]
  );

  const removeInstruction = getRemovePluginV1Instruction({
    asset: assetSigner.address,
    payer,
    pluginType: PluginType.PermanentFreezeExecute,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [removeInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can remove PermanentFreezeExecute plugin if unfrozen', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetSigner = await generateKeyPairSigner();

  const createInstruction = getCreateV1Instruction({
    asset: assetSigner,
    payer,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        plugin: {
          __kind: 'PermanentFreezeExecute',
          fields: [{ frozen: false }],
        },
        authority: { __option: 'None' },
      },
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [createInstruction],
    [assetSigner, payer]
  );

  const removeInstruction = getRemovePluginV1Instruction({
    asset: assetSigner.address,
    payer,
    pluginType: PluginType.PermanentFreezeExecute,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [removeInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: assetSigner.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});
