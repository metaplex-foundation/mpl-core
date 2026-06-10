import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getAddCollectionPluginV1Instruction,
  getAddPluginV1Instruction,
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

test('it can add addBlocker to collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer);

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'AddBlocker',
      fields: [{}],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    addBlocker: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it cannot add UA-managed plugin to a collection if addBlocker had been added on creation', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'AddBlocker',
      }),
    ],
  });

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'ImmutableMetadata',
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

test('it cannot add UA-managed plugin to an asset in a collection if addBlocker had been added on creation', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'AddBlocker',
      }),
    ],
  });

  const asset = await createAsset(rpc, payer, {
    collection: collection.address,
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'ImmutableMetadata',
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

test('it prevents plugins from being added to both collection and plugins when collection is created with AddBlocker', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateAuthority = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'AddBlocker',
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
    updateAuthority: updateAuthority.address,
  });

  const asset = await createAsset(rpc, payer, {
    collection: collection.address,
    authority: updateAuthority,
  });

  const instruction1 = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'ImmutableMetadata',
      fields: [{}],
    },
  });

  const result1 = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction1],
    [payer]
  );

  await t.throwsAsync(result1);

  const instruction2 = getAddPluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'ImmutableMetadata',
      fields: [{}],
    },
  });

  const result2 = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction2],
    [payer]
  );

  await t.throwsAsync(result2);
});

test('it prevents plugins from being added to both collection and plugins when AddBlocker is added to a collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer);
  const asset = await createAsset(rpc, payer, { collection: collection.address });

  const addBlockerInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'AddBlocker',
      fields: [{}],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addBlockerInstruction],
    [payer]
  );

  const instruction1 = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'ImmutableMetadata',
      fields: [{}],
    },
  });

  const result1 = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction1],
    [payer]
  );

  await t.throwsAsync(result1);

  const instruction2 = getAddPluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'ImmutableMetadata',
      fields: [{}],
    },
  });

  const result2 = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction2],
    [payer]
  );

  await t.throwsAsync(result2);
});
