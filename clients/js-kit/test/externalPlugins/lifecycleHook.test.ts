import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getAddExternalPluginAdapterV1Instruction,
  getUpdateExternalPluginAdapterV1Instruction,
  getCreateV2Instruction,
} from '../../src';
import {
  createExternalPluginAdapterInitInfo,
  createExternalPluginAdapterUpdateInfo,
  createExternalPluginAdapterKey,
  CheckResult,
} from '../../src/plugins';
import {
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../_setup';

test.skip('it cannot create asset with lifecycle hook that has no lifecycle checks', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const account = await generateKeyPairSigner();
  const owner = await generateKeyPairSigner();
  const assetAddress = await generateKeyPairSigner();

  const instruction = getCreateV2Instruction({
    asset: assetAddress,
    payer,
    owner: owner.address,
    name: DEFAULT_ASSET.name,
    uri: DEFAULT_ASSET.uri,
    externalPluginAdapters: [
      createExternalPluginAdapterInitInfo({
        type: 'LifecycleHook',
        hookedProgram: account.address,
        lifecycleChecks: {},
      }),
    ],
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  await t.throwsAsync(result);
});

test.skip('it cannot add lifecycle hook with no lifecycle checks to asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const account = await generateKeyPairSigner();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const assetAddress = await generateKeyPairSigner();

  const createInstruction = getCreateV2Instruction({
    asset: assetAddress,
    payer,
    owner: owner.address,
    name: DEFAULT_ASSET.name,
    uri: DEFAULT_ASSET.uri,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [createInstruction],
    [assetAddress, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: assetAddress.address,
    owner: owner.address,
  });

  const instruction = getAddExternalPluginAdapterV1Instruction({
    asset: assetAddress.address,
    payer,
    initInfo: createExternalPluginAdapterInitInfo({
      type: 'LifecycleHook',
      hookedProgram: account.address,
      lifecycleChecks: {},
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: assetAddress.address,
    owner: owner.address,
  });
});

test.skip('it cannot update lifecycle hook to have no lifecycle checks', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const account = await generateKeyPairSigner();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    externalPluginAdapters: [
      createExternalPluginAdapterInitInfo({
        type: 'LifecycleHook',
        hookedProgram: account.address,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_LISTEN],
        },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
  });

  const instruction = getUpdateExternalPluginAdapterV1Instruction({
    asset: asset.address,
    payer,
    key: createExternalPluginAdapterKey({
      type: 'LifecycleHook',
      hookedProgram: account.address,
    }),
    updateInfo: createExternalPluginAdapterUpdateInfo({
      type: 'LifecycleHook',
      lifecycleChecks: {},
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
  });
});
