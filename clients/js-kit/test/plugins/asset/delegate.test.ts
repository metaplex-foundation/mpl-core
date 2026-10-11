import { generateKeyPairSigner } from '@solana/signers';
import test from 'ava';
import {
  PluginType,
  getApprovePluginAuthorityV1Instruction,
  getUpdatePluginV1Instruction,
} from '../../../src';
import { pluginAuthorityPair } from '../../../src/plugins';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can delegate a new authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const instruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
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
        type: 'Address',
        address: delegateAddress.address,
      },
      frozen: false,
    },
  });
});

test('a delegate can freeze the token', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer: delegateAddress,
    authority: delegateAddress,
    plugin: { __kind: 'FreezeDelegate', fields: [{ frozen: true }] },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
    [delegateAddress]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.address,
      },
      frozen: true,
    },
  });
});
