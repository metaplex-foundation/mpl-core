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

test('it can add PermanentFreezeExecute to collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeExecute',
        data: { frozen: true },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('it can remove PermanentFreezeExecute from collection when unfrozen', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeExecute',
        data: { frozen: false },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  const instruction = getRemoveCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.PermanentFreezeExecute,
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
    permanentFreezeExecute: undefined,
  });
});

test('it cannot remove PermanentFreezeExecute from collection when frozen', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeExecute',
        data: { frozen: true },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  const instruction = getRemoveCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.PermanentFreezeExecute,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('it cannot add PermanentFreezeExecute to collection after creation', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer);

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'PermanentFreezeExecute',
      fields: [{ frozen: true }],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeExecute: undefined,
  });
});

test('it can freeze and unfreeze a collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeExecute',
        data: { frozen: false },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  // Freeze
  const freezeInstruction = getUpdateCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'PermanentFreezeExecute',
      fields: [{ frozen: true }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [freezeInstruction],
    [payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  // Unfreeze
  const unfreezeInstruction = getUpdateCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'PermanentFreezeExecute',
      fields: [{ frozen: false }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [unfreezeInstruction],
    [payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});
