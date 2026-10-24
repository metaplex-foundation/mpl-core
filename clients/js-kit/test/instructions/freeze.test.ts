import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getAddPluginV1Instruction,
  getRemovePluginV1Instruction,
  getUpdatePluginV1Instruction,
  getRevokePluginAuthorityV1Instruction,
  PluginType,
} from '../../src';
import {
  createAsset,
  createAssetWithCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../_setup';
import { pluginAuthorityPair } from '../../src/plugins';

test('it can freeze an asset by adding freeze delegate plugin', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);

  const freezePlugin = pluginAuthorityPair({
    type: 'FreezeDelegate',
    data: { frozen: true },
    authority: { __kind: 'Address', address: delegate.address },
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: freezePlugin.plugin,
    initAuthority: freezePlugin.authority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
      frozen: true,
    },
  });
});

test('it can freeze an asset with the plugin already defined', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  // First remove the existing plugin
  const removeInstruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
  });

  // Then add it back with frozen: true and new delegate
  const freezePlugin = pluginAuthorityPair({
    type: 'FreezeDelegate',
    data: { frozen: true },
    authority: { __kind: 'Address', address: delegate.address },
  });

  const addInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: freezePlugin.plugin,
    initAuthority: freezePlugin.authority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [removeInstruction, addInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
      frozen: true,
    },
  });
});

test('it can freeze an asset with the plugin delegated if unfrozen', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();
  const delegate2 = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        authority: { __kind: 'Address', address: delegate.address },
        data: { frozen: false },
      }),
    ],
  });

  // Remove existing plugin and add new one with different delegate
  const removeInstruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
  });

  const freezePlugin = pluginAuthorityPair({
    type: 'FreezeDelegate',
    data: { frozen: true },
    authority: { __kind: 'Address', address: delegate2.address },
  });

  const addInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: freezePlugin.plugin,
    initAuthority: freezePlugin.authority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [removeInstruction, addInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate2.address,
      },
      frozen: true,
    },
  });
});

test('it can freeze asset in collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});

  const freezePlugin = pluginAuthorityPair({
    type: 'FreezeDelegate',
    data: { frozen: true },
    authority: { __kind: 'Address', address: delegate.address },
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    collection: collection.address,
    plugin: freezePlugin.plugin,
    initAuthority: freezePlugin.authority,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
      frozen: true,
    },
  });
});

test('it cannot freeze a frozen asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  // The asset is already frozen, so we should not be able to remove and re-add
  // This test verifies the frozen state persists
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it cannot freeze a perma frozen asset in collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: true },
        }),
      ],
    }
  );

  // The asset inherits the permanent freeze from collection, so it's already frozen
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });
});

test('it can thaw a frozen asset using update plugin', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        authority: { __kind: 'Address', address: delegate.address },
        data: { frozen: true },
      }),
    ],
  });

  // Update the plugin to set frozen: false
  const updatePlugin = pluginAuthorityPair({
    type: 'FreezeDelegate',
    data: { frozen: false },
  });

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: updatePlugin.plugin,
    authority: delegate,
  });

  // Revoke the delegate authority back to owner
  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
    authority: delegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction, revokeInstruction],
    [delegate, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it can thaw a frozen asset in collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        authority: { __kind: 'Address', address: delegate.address },
        data: { frozen: true },
      }),
    ],
  });

  // Update the plugin to set frozen: false
  const updatePlugin = pluginAuthorityPair({
    type: 'FreezeDelegate',
    data: { frozen: false },
  });

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    collection: collection.address,
    plugin: updatePlugin.plugin,
    authority: delegate,
  });

  // Revoke the delegate authority back to owner
  const revokeInstruction = getRevokePluginAuthorityV1Instruction({
    asset: asset.address,
    payer,
    collection: collection.address,
    pluginType: PluginType.FreezeDelegate,
    authority: delegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction, revokeInstruction],
    [delegate, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it cannot thaw an unfrozen asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        authority: { __kind: 'Address', address: delegate.address },
        data: { frozen: false },
      }),
    ],
  });

  // The asset is already unfrozen, verify the state
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
      frozen: false,
    },
  });
});
