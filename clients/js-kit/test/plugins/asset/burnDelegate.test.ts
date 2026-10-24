import { generateKeyPairSigner } from '@solana/signers';
import test from 'ava';
import {
  getAddPluginV1Instruction,
  getRevokePluginAuthorityV1Instruction,
  getBurnV1Instruction,
  PluginType,
} from '../../../src';
import { pluginAuthorityPair, createPlugin } from '../../../src/plugins';
import {
  DEFAULT_ASSET,
  assertAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertBurned,
  sendAndConfirmInstructions,
  createAsset,
  createCollection,
} from '../../_setup';

test('it can create an asset with burnDelegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'BurnDelegate',
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can add burnDelegate to an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {});

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    burnDelegate: undefined,
  });

  const burnDelegate = await generateKeyPairSigner();

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'BurnDelegate' }),
    initAuthority: { __kind: 'Address', address: burnDelegate.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    burnDelegate: {
      authority: {
        type: 'Address',
        address: burnDelegate.address,
      },
    },
  });
});

test('a burnDelegate can burn an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {});
  const burnDelegate = await generateKeyPairSigner();

  const addPluginInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'BurnDelegate' }),
    initAuthority: { __kind: 'Address', address: burnDelegate.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addPluginInstruction], [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    burnDelegate: {
      authority: {
        type: 'Address',
        address: burnDelegate.address,
      },
    },
  });

  const burnInstruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
    authority: burnDelegate,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [burnInstruction], [burnDelegate, payer]);

  await assertBurned(t, rpc, asset.address);
});

test('an burnDelegate cannot burn an asset after delegate authority revoked', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {});
  const burnDelegate = await generateKeyPairSigner();

  const addPluginInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'BurnDelegate' }),
    initAuthority: { __kind: 'Address', address: burnDelegate.address },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [addPluginInstruction], [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    burnDelegate: {
      authority: {
        type: 'Address',
        address: burnDelegate.address,
      },
    },
  });

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.BurnDelegate,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [revokeInstruction], [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  const burnInstruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
    authority: burnDelegate,
  });

  const result = sendAndConfirmInstructions(rpc, rpcSubscriptions, [burnInstruction], [burnDelegate, payer]);

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('a burnDelegate can burn using delegated update authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const updateAuthority = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    updateAuthority: updateAuthority.address,
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: {
          type: 'UpdateAuthority',
        },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthority.address },
    burnDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const burnInstruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
    authority: updateAuthority,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [burnInstruction], [updateAuthority, payer]);

  await assertBurned(t, rpc, asset.address);
});

test('a burnDelegate can burn using delegated update authority from collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const updateAuthority = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    updateAuthority: updateAuthority.address,
  });

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    collection: collection.address,
    plugins: [
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: {
          type: 'UpdateAuthority',
        },
      }),
    ],
    authority: updateAuthority,
  });

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Collection', address: collection.address },
    burnDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const burnInstruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
    collection: collection.address,
    authority: updateAuthority,
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [burnInstruction], [updateAuthority, payer]);

  await assertBurned(t, rpc, asset.address);
});
