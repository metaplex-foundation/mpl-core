import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { getAddPluginV1Instruction } from '../../../src';
import { pluginAuthorityPair, createPlugin } from '../../../src/plugins';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it cannot add UA-managed plugin if addBlocker had been added on creation', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'AddBlocker',
      }),
    ],
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Attributes',
      data: { attributeList: [] },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can add plugins unless AddBlocker is added', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction1 = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Attributes',
      data: { attributeList: [] },
    }),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction1],
    [payer]
  );

  const instruction2 = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'AddBlocker',
    }),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction2],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [],
    },
    addBlocker: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const instruction3 = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Attributes',
      data: { attributeList: [] },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction3],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can add owner-managed plugins even if AddBlocker had been added', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'AddBlocker',
      }),
    ],
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

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
    addBlocker: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it states that UA is the only one who can add the AddBlocker', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateAuthority = await generateKeyPairSigner();
  const randomUser = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    updateAuthority: updateAuthority.address,
  });

  // random keypair can't add AddBlocker
  const instruction1 = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    authority: randomUser,
    plugin: createPlugin({
      type: 'AddBlocker',
    }),
  });

  let result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction1],
    [randomUser, payer]
  );

  await t.throwsAsync(result);

  // Owner can't add AddBlocker
  const instruction2 = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    plugin: createPlugin({
      type: 'AddBlocker',
    }),
  });

  result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction2],
    [payer]
  );

  await t.throwsAsync(result);

  // UA CAN add AddBlocker
  const instruction3 = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateAuthority,
    plugin: createPlugin({ type: 'AddBlocker' }),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction3],
    [updateAuthority, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: updateAuthority.address },
    addBlocker: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});
