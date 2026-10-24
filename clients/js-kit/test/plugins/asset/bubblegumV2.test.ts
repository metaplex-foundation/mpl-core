import test from 'ava';
import { getAddPluginV1Instruction } from '../../../src';
import { pluginAuthorityPair } from '../../../src/plugins';
import {
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it cannot create asset with BubblegumV2 plugin', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const result = createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'BubblegumV2',
      }),
    ],
  });

  await t.throwsAsync(result);
});

test('it cannot add BubblegumV2 to asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'BubblegumV2',
      fields: [{}],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});
