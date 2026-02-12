import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getAddPluginV1Instruction,
  getUpdateV1Instruction,
} from '../../../src';
import { pluginAuthorityPair, createPlugin } from '../../../src/plugins';
import {
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can prevent the asset from metadata updating', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'ImmutableMetadata',
      }),
    ],
  });

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'bread',
    newUri: 'https://example.com/bread',
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can mutate its metadata unless ImmutableMetadata plugin is added', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const updateInstruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [updateInstruction], [
    payer,
  ]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  const addPluginInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [addPluginInstruction],
    [payer]
  );

  const secondUpdateInstruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Bread 3',
    newUri: 'https://example.com/bread3',
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [secondUpdateInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it states that UA is the only one who can add the ImmutableMetadata', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateAuthority = await generateKeyPairSigner();
  const randomUser = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    updateAuthority: updateAuthority,
  });

  // random keypair can't add ImmutableMetadata
  const randomUserInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    authority: randomUser,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  });

  let result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [randomUserInstruction],
    [randomUser, payer]
  );

  await t.throwsAsync(result);

  // Owner can't add ImmutableMetadata
  const ownerInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  });

  result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [ownerInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  // UA CAN add ImmutableMetadata
  const uaInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    authority: updateAuthority,
    plugin: createPlugin({ type: 'ImmutableMetadata' }),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [uaInstruction],
    [updateAuthority, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    immutableMetadata: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});
