import test from 'ava';
import {
  getAddCollectionPluginV1Instruction,
  getUpdateCollectionPluginV1Instruction,
  getRemoveCollectionPluginV1Instruction,
  PluginType,
} from '../../../src';
import { pluginAuthorityPair } from '../../../src/plugins';
import {
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertCollection,
  DEFAULT_COLLECTION,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can add permanentFreezeDelegate to collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('it can remove permanentFreezeDelegate from collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  const removeInstruction = getRemoveCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.PermanentFreezeDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [removeInstruction],
    [payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeDelegate: undefined,
  });
});

test('it cannot remove permanentFreezeDelegate from collection when frozen', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  const removeInstruction = getRemoveCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.PermanentFreezeDelegate,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [removeInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('it cannot add permanentFreezeDelegate to collection after creation', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer);

  const addInstruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'PermanentFreezeDelegate',
      fields: [{ frozen: true }],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [addInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeDelegate: undefined,
  });
});

test('it can freeze and unfreeze a collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  const updateInstruction = getUpdateCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'PermanentFreezeDelegate',
      fields: [{ frozen: false }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction],
    [payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});
