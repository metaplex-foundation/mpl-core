import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getUpdatePluginV1Instruction,
  getAddPluginV1Instruction,
  getRemovePluginV1Instruction,
  getTransferV1Instruction,
  PluginType,
} from '../../../src';
import { pluginAuthorityPair } from '../../../src/plugins';
import {
  createAsset,
  createAssetWithCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can freeze and unfreeze an asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'PermanentFreezeDelegate',
      fields: [{ frozen: false }],
    },
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
    owner,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('it cannot be transferred while frozen', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const newOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  const transferInstruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
    authority: owner,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [transferInstruction],
    [owner, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner,
    updateAuthority: { type: 'Address', address: payer.address },
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('it cannot add permanentFreeze after creation', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, { owner: owner.address });

  const addInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'PermanentFreezeDelegate',
      fields: [{ frozen: true }],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot move asset in a permanently frozen collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: true },
        }),
      ],
    }
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  const transferInstruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
    collection: collection.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [transferInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });
});

test('it can move asset with permanent freeze override in a frozen collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: false },
        }),
      ],
    },
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: true },
        }),
      ],
    }
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  const transferInstruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
    authority: payer,
    collection: collection.address,
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
    owner: newOwner,
    updateAuthority: { type: 'Collection', address: collection.address },
  });
});

test('it can unfreeze a permanent freeze plugin from an asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'PermanentFreezeDelegate',
      fields: [{ frozen: false }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('it cannot remove permanent freeze plugin if update authority and frozen', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  const removeInstruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.PermanentFreezeDelegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [removeInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('it can remove permanent freeze plugin if update authority and unfrozen', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  const removeInstruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.PermanentFreezeDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [removeInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    permanentFreezeDelegate: undefined,
  });
});

test('it can add another plugin on asset with permanent freeze plugin', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: false },
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
    asset: asset.address,
    owner: payer,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});
