import test from 'ava';
import {
  getAddCollectionPluginV1Instruction,
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

test('it can add permanentBurnDelegate to collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentBurnDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it cannot add permanentBurnDelegate to collection after creation', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer);

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'PermanentBurnDelegate',
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

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentBurnDelegate: undefined,
  });
});

test('it can remove permanentBurnDelegate from collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    permanentBurnDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const instruction = getRemoveCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.PermanentBurnDelegate,
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
    permanentBurnDelegate: undefined,
  });
});
