import test from 'ava';
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

test('it can add masterEdition to collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer);

  const instruction = getAddCollectionPluginV1Instruction({
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'MasterEdition',
      fields: [
        {
          maxSupply: { __option: 'Some', value: 100 },
          name: { __option: 'Some', value: 'name' },
          uri: { __option: 'Some', value: 'uri' },
        },
      ],
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
    masterEdition: {
      authority: {
        type: 'UpdateAuthority',
      },
      maxSupply: 100,
      name: 'name',
      uri: 'uri',
    },
  });
});

test('it can create collection with masterEdition', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'MasterEdition',
        data: {
          maxSupply: 100,
          name: 'name',
          uri: 'uri',
        },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    masterEdition: {
      authority: {
        type: 'UpdateAuthority',
      },
      maxSupply: 100,
      name: 'name',
      uri: 'uri',
    },
  });
});

test('it can create master edition with default values', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'MasterEdition',
        data: {
          maxSupply: null,
          name: null,
          uri: null,
        },
      }),
    ],
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    masterEdition: {
      authority: {
        type: 'UpdateAuthority',
      },
      maxSupply: undefined,
      name: undefined,
      uri: undefined,
    },
  });
});

test('it cannot add masterEdition to asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'MasterEdition',
      fields: [
        {
          maxSupply: { __option: 'Some', value: 100 },
          name: { __option: 'Some', value: 'name' },
          uri: { __option: 'Some', value: 'uri' },
        },
      ],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot create asset with masterEdition', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const result = createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'MasterEdition',
        data: {
          maxSupply: 100,
          name: 'name',
          uri: 'uri',
        },
      }),
    ],
  });

  await t.throwsAsync(result);
});
