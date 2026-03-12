import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { address } from '@solana/addresses';
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

const MPL_BUBBLEGUM_PROGRAM_ID = address('BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY');

test('it can create collection with BubblegumV2 plugin', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'BubblegumV2',
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 0,
    numMinted: 0,
    bubblegumV2: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
  });
});

test('it cannot add BubblegumV2 to collection after creation', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer);

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'BubblegumV2',
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
    bubblegumV2: undefined,
  });
});

test('Update Authority cannot remove BubblegumV2 from collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'BubblegumV2',
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    currentSize: 0,
    numMinted: 0,
    bubblegumV2: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
  });

  const instruction = getRemoveCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.BubblegumV2,
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
    currentSize: 0,
    numMinted: 0,
    bubblegumV2: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
  });
});

test('it can create collection with BubblegumV2 plugin and other allow-listed plugins', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'BubblegumV2',
      }),
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
      }),
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
    currentSize: 0,
    numMinted: 0,
    bubblegumV2: {
      authority: {
        type: 'Address',
        address: MPL_BUBBLEGUM_PROGRAM_ID,
      },
    },
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('it cannot create collection with BubblegumV2 plugin and non-allow-listed plugins', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const result = createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'BubblegumV2',
      }),
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        data: { additionalDelegates: [] },
      }),
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: false },
      }),
      pluginAuthorityPair({
        type: 'MasterEdition',
        data: { maxSupply: 100, name: 'master', uri: 'uri master' },
      }),
    ],
  });

  await t.throwsAsync(result);
});

test('it cannot create collection with BubblegumV2 plugin using wrong authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const updateAuthorityResult = createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'BubblegumV2',
        authority: { __kind: 'UpdateAuthority' },
      }),
    ],
  });

  await t.throwsAsync(updateAuthorityResult);

  const noneResult = createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'BubblegumV2',
        authority: { __kind: 'None' },
      }),
    ],
  });

  await t.throwsAsync(noneResult);

  const ownerResult = createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'BubblegumV2',
        authority: { __kind: 'Owner' },
      }),
    ],
  });

  await t.throwsAsync(ownerResult);

  const randomSigner = await generateKeyPairSigner();
  const addressResult = createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'BubblegumV2',
        authority: { __kind: 'Address', address: randomSigner.address },
      }),
    ],
  });

  await t.throwsAsync(addressResult);
});
