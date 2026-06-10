import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { getTransferV1Instruction } from '../src';
import { pluginAuthorityPair } from '../src/plugins';
import {
  createAsset,
  createAssetWithCollection,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  sendAndConfirmInstructions,
} from './_setup';

test('it can transfer an asset as the owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot transfer an asset if not the owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const attacker = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer: attacker,
    newOwner: newOwner.address,
    authority: attacker,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [attacker]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot transfer asset in collection if no collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });
});

test('it can transfer asset in collection as the owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
    collection: collection.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Collection', address: collection.address },
  });
});

test('it cannot transfer asset in collection with the wrong collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});
  const wrongCollection = await createCollection(rpc, payer);

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
    collection: wrongCollection.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });
});

test('authorities on owner-managed plugins are reset on transfer', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const freezeDelegate = await generateKeyPairSigner();
  const newOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: { __kind: 'Address', address: freezeDelegate.address },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('authorities on permanent plugins should not be reset on transfer', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const freezeDelegate = await generateKeyPairSigner();
  const newOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: false },
        authority: { __kind: 'Address', address: freezeDelegate.address },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot use an invalid system program', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const fakeSystemProgram = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid noop program', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const fakeLogWrapper = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
    logWrapper: fakeLogWrapper.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});
