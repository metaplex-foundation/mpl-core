import test from 'ava';
import { getUpdatePluginV1Instruction } from '../../../src';
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

test('it can add attributes to an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'Attributes', data: { attributeList: [] } }),
    ],
  });

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
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Attributes',
      data: {
        attributeList: [
          { key: 'key0', value: 'value0' },
          { key: 'key1', value: 'value1' },
        ],
      },
    }),
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
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        { key: 'key0', value: 'value0' },
        { key: 'key1', value: 'value1' },
      ],
    },
  });
});

test('it can remove attributes to an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: {
          attributeList: [
            { key: 'key0', value: 'value0' },
            { key: 'key1', value: 'value1' },
          ],
        },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [
        { key: 'key0', value: 'value0' },
        { key: 'key1', value: 'value1' },
      ],
    },
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Attributes',
      data: { attributeList: [{ key: 'key0', value: 'value0' }] },
    }),
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
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key0', value: 'value0' }],
    },
  });
});

test('it can add then remove attributes to an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'Attributes', data: { attributeList: [] } }),
    ],
  });

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
  });

  const instruction1 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Attributes',
      data: {
        attributeList: [
          { key: 'key0', value: 'value0' },
          { key: 'key1', value: 'value1' },
        ],
      },
    }),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction1],
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
      attributeList: [
        { key: 'key0', value: 'value0' },
        { key: 'key1', value: 'value1' },
      ],
    },
  });

  const instruction2 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({ type: 'Attributes', data: { attributeList: [] } }),
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
  });
});
