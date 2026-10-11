import { generateKeyPairSigner } from '@solana/signers';
import test from 'ava';
import {
  getApprovePluginAuthorityV1Instruction,
  getTransferV1Instruction,
  getRevokePluginAuthorityV1Instruction,
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
  sendAndConfirmInstructions,
  DEFAULT_ASSET,
} from '../../_setup';

test('a delegate can transfer the asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();
  const newOwnerAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [pluginAuthorityPair({ type: 'TransferDelegate' })],
  });

  const approveInstruction = getApprovePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.TransferDelegate,
    newAuthority: { __kind: 'Address', address: delegateAddress.address },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  const transferInstruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwnerAddress.address,
    authority: delegateAddress,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [transferInstruction],
    [delegateAddress, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: newOwnerAddress,
    updateAuthority: { type: 'Address', address: payer.address },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('owner can transfer asset with delegate transfer', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();
  const newOwnerAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: { __kind: 'Address', address: delegateAddress.address },
      }),
    ],
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwnerAddress.address,
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
    owner: newOwnerAddress,
    updateAuthority: { type: 'Address', address: payer.address },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can revoke a delegate transfer plugin', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: { __kind: 'Address', address: delegateAddress.address },
      }),
    ],
  });

  const instruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.TransferDelegate,
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
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it cannot transfer after delegate authority has been revoked', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegateAddress = await generateKeyPairSigner();
  const newOwnerAddress = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: { __kind: 'Address', address: delegateAddress.address },
      }),
    ],
  });

  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.TransferDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [revokeInstruction],
    [payer]
  );

  const transferInstruction = getTransferV1Instruction({
    asset: asset.address,
    payer: delegateAddress,
    newOwner: newOwnerAddress.address,
    authority: delegateAddress,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [transferInstruction],
    [delegateAddress]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can transfer using delegated update authority', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const updateAuthority = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    updateAuthority: updateAuthority.address,
    plugins: [
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
    authority: updateAuthority,
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: updateAuthority.address },
    transferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer: updateAuthority,
    newOwner: newOwner.address,
    authority: updateAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [updateAuthority]
  );

  // Resets to `Owner` after the transfer.
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Address', address: updateAuthority.address },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can transfer using delegated update authority from collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const updateAuthority = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      owner: owner.address,
      updateAuthority: updateAuthority.address,
      plugins: [
        pluginAuthorityPair({
          type: 'TransferDelegate',
          authority: { __kind: 'UpdateAuthority' },
        }),
      ],
      authority: updateAuthority,
    },
    {
      updateAuthority: updateAuthority.address,
    }
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Collection', address: collection.address },
    transferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer: updateAuthority,
    collection: collection.address,
    newOwner: newOwner.address,
    authority: updateAuthority,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [updateAuthority]
  );

  // Resets to `Owner` after the transfer.
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Collection', address: collection.address },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});
