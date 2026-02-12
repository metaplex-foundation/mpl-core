import { address } from '@solana/addresses';
import test from 'ava';
import { ExternalPluginAdapterSchema } from '../../src';
import {
  getCreateV2Instruction,
  getCreateCollectionV2Instruction,
  getAddExternalPluginAdapterV1Instruction,
  getAddCollectionExternalPluginAdapterV1Instruction,
} from '../../src';
import {
  createAsset,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../_setup';
import { generateKeyPairSigner } from '@solana/signers';

const SPL_SYSTEM_PROGRAM_ADDRESS = address('11111111111111111111111111111111');

test('it cannot create an asset with a DataSection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const asset = await generateKeyPairSigner();

  const instruction = getCreateV2Instruction({
    asset,
    payer,
    name: DEFAULT_ASSET.name,
    uri: DEFAULT_ASSET.uri,
    externalPluginAdapters: [
      {
        __kind: 'DataSection',
        fields: [
          {
            parentKey: {
              __kind: 'LinkedLifecycleHook',
              fields: [SPL_SYSTEM_PROGRAM_ADDRESS],
            },
            schema: ExternalPluginAdapterSchema.Binary,
          },
        ],
      },
    ],
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [asset, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot create a collection with a DataSection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collection = await generateKeyPairSigner();

  const instruction = getCreateCollectionV2Instruction({
    collection,
    payer,
    name: DEFAULT_ASSET.name,
    uri: DEFAULT_ASSET.uri,
    externalPluginAdapters: [
      {
        __kind: 'DataSection',
        fields: [
          {
            parentKey: {
              __kind: 'LinkedLifecycleHook',
              fields: [SPL_SYSTEM_PROGRAM_ADDRESS],
            },
            schema: ExternalPluginAdapterSchema.Binary,
          },
        ],
      },
    ],
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [collection, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot add a DataSection to an asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction = getAddExternalPluginAdapterV1Instruction({
    asset: asset.address,
    payer,
    initInfo: {
      __kind: 'DataSection',
      fields: [
        {
          parentKey: {
            __kind: 'LinkedLifecycleHook',
            fields: [SPL_SYSTEM_PROGRAM_ADDRESS],
          },
          schema: ExternalPluginAdapterSchema.Binary,
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

test('it cannot add a DataSection to a collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer);

  const instruction = getAddCollectionExternalPluginAdapterV1Instruction({
    collection: collection.address,
    payer,
    initInfo: {
      __kind: 'DataSection',
      fields: [
        {
          parentKey: {
            __kind: 'LinkedLifecycleHook',
            fields: [SPL_SYSTEM_PROGRAM_ADDRESS],
          },
          schema: ExternalPluginAdapterSchema.Binary,
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
