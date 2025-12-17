import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getCreateCollectionV1Instruction,
  getApproveCollectionPluginAuthorityV1Instruction,
  PluginType,
} from '../src';
import { pluginAuthorityPair } from '../src/plugins';
import {
  createAsset,
  createAssetWithCollection,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  assertCollection,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  sendAndConfirmInstructions,
} from './_setup';

test('it can create a new collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collectionAddress = await generateKeyPairSigner();

  await createCollection(rpc, payer, {
    collection: collectionAddress,
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collectionAddress.address,
    updateAuthority: payer.address,
  });
});

test('it can create a new collection with plugins', async (t) => {
  const rpc = createRpc();
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
        __kind: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('it can create a new asset with a collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
        }),
      ],
    }
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    updateAuthority: payer.address,
    updateDelegate: {
      authority: {
        __kind: 'UpdateAuthority',
      },
      additionalDelegates: [],
    },
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });
});

test('it can create a new asset with a collection with collection delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateSignerWithSol(rpc);

  const collection = await createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
      }),
    ],
  });

  const approveInstruction = getApproveCollectionPluginAuthorityV1Instruction({
    collection: collection.address,
    payer,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: { __kind: 'Address', address: delegate.address },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [approveInstruction],
    [payer]
  );

  const payer2 = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer2, {
    collection: collection.address,
    authority: delegate,
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer2,
    updateAuthority: { type: 'Collection', address: collection.address },
  });
});

test('it cannot create a new asset with an update authority that is not the collection', async (t) => {
  t.pass();
});

test('it cannot create a new asset with a collection if it is not the collection auth', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collectionAuth = await generateKeyPairSigner();

  const collection = await createCollection(rpc, payer, {
    updateAuthority: collectionAuth.address,
  });

  const result = createAsset(rpc, payer, {
    collection: collection.address,
  });

  await t.throwsAsync(result);
});

test('it cannot create a collection with an owner managed plugin', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const result = createCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: {
          frozen: false,
        },
      }),
    ],
  });

  await t.throwsAsync(result);
});

test('it cannot use an invalid system program', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collectionAddress = await generateKeyPairSigner();
  const fakeSystemProgram = await generateKeyPairSigner();

  const instruction = getCreateCollectionV1Instruction({
    collection: collectionAddress,
    payer,
    name: 'Test',
    uri: 'Test',
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [collectionAddress, payer]
  );

  await t.throwsAsync(result);
});
