import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getAddCollectionPluginV1Instruction,
  getUpdateV1Instruction,
  getUpdateCollectionV1Instruction,
} from '../../../src';
import { pluginAuthorityPair } from '../../../src/plugins';
import {
  createAsset,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertCollection,
  DEFAULT_COLLECTION,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can add immutableMetadata to collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer);

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'ImmutableMetadata',
      fields: [{}],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    immutableMetadata: {
      authority: {
        __kind: 'UpdateAuthority',
      },
    },
  });
});

test('it can prevent collection assets metadata from being updated', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer);

  const addInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'ImmutableMetadata',
      fields: [{}],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [addInstruction],
    [payer]
  );

  const asset = await createAsset(rpc, payer, {
    collection: collection.address,
  });

  const updateInstruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    collection: collection.address,
    newName: 'Test Bread 3',
    newUri: 'https://example.com/bread3',
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it states that UA is the only one who can add the ImmutableMetadata', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateAuthority = await generateKeyPairSigner();
  const randomUser = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer, {
    updateAuthority: updateAuthority.address,
  });

  // random keypair can't add ImmutableMetadata
  let addInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer: randomUser,
    authority: randomUser,
    plugin: {
      __kind: 'ImmutableMetadata',
      fields: [{}],
    },
  });

  let result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [addInstruction],
    [randomUser]
  );

  await t.throwsAsync(result);

  // Payer for the the collection can't add ImmutableMetadata
  addInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    authority: payer,
    plugin: {
      __kind: 'ImmutableMetadata',
      fields: [{}],
    },
  });

  result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [addInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  // UA CAN add ImmutableMetadata
  addInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    authority: updateAuthority,
    plugin: {
      __kind: 'ImmutableMetadata',
      fields: [{}],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [addInstruction],
    [updateAuthority, payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    immutableMetadata: {
      authority: {
        __kind: 'UpdateAuthority',
      },
    },
  });
});

test('it prevents both collection and asset from their meta updating when ImmutableMetadata is added', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const updateAuthority = await generateKeyPairSigner();
  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'ImmutableMetadata',
        authority: {
          __kind: 'UpdateAuthority',
        },
      }),
    ],
    updateAuthority: updateAuthority.address,
  });
  const asset = await createAsset(rpc, payer, {
    collection: collection.address,
    authority: updateAuthority,
  });

  let updateInstruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    collection: collection.address,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  });

  let result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  const updateCollectionInstruction = getUpdateCollectionV1Instruction({
    collection: collection.address,
    payer,
    authority: updateAuthority,
    newName: 'Test',
    newUri: 'Test',
  });

  result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateCollectionInstruction],
    [updateAuthority, payer]
  );

  await t.throwsAsync(result);
});
