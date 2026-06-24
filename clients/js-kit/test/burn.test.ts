import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { getBurnV1Instruction } from '../src';
import { pluginAuthorityPair } from '../src/plugins';
import {
  createAsset,
  createAssetWithCollection,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  assertBurned,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from './_setup';

test('it can burn an asset as the owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertBurned(t, rpc, asset.address);
});

test('it cannot burn an asset if not the owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const attacker = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
    authority: attacker,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [attacker, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot burn an asset as the authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const authority = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, { updateAuthority: authority });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: authority.address },
  });

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
    authority,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [authority, payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: authority.address },
  });
});

test('it cannot burn an asset if it is frozen', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot burn asset in collection if no collection specified', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
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
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });
});

test('it cannot burn an asset if collection permanently frozen', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

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

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
    collection: collection.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid system program for assets', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);
  const fakeSystemProgram = await generateKeyPairSigner();

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
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

test('it cannot use an invalid noop program for assets', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);
  const fakeLogWrapper = await generateKeyPairSigner();

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
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

test('it can burn using owner authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [owner, payer]
  );

  await assertBurned(t, rpc, asset.address);
});

test('it cannot burn an asset with the wrong collection specified', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);
  const wrongCollection = await createCollection(rpc, payer);

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
    collection: wrongCollection.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});
