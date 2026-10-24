import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import type { IInstruction } from '@solana/instructions';
import { createAsset, createRpc, createRpcSubscriptions, generateSignerWithSol, assertAsset, sendAndConfirmInstructions } from '../_setup';
import { getRevokePluginAuthorityV1Instruction, fetchAssetV1, PluginType } from '../../src';
import { pluginAuthorityPair, pluginKeyToPluginType } from '../../src/plugins';

const ERR_CANNOT_REVOKE = 'Cannot revoke: no plugins with delegated authorities';

type AssetPluginsList = {
  freezeDelegate?: { authority: { type: string; address?: string } };
  transferDelegate?: { authority: { type: string; address?: string } };
  burnDelegate?: { authority: { type: string; address?: string } };
};

async function legacyRevoke(rpc: ReturnType<typeof createRpc>, asset: Awaited<ReturnType<typeof fetchAssetV1>>): Promise<IInstruction[]> {
  const definedPlugins = (({
    freezeDelegate,
    transferDelegate,
    burnDelegate,
  }) => ({
    ...(freezeDelegate ? { freezeDelegate } : {}),
    ...(transferDelegate ? { transferDelegate } : {}),
    ...(burnDelegate ? { burnDelegate } : {}),
  }))(asset.data as unknown as AssetPluginsList);

  const definedPluginsValues = Object.values(definedPlugins);
  const canRevoke =
    definedPluginsValues.length > 0 &&
    definedPluginsValues.every((pluginValue) => {
      const assetOwner = asset.data.owner;
      const pluginAuthority = pluginValue.authority;

      return (
        pluginAuthority.type !== 'Owner' &&
        pluginAuthority.address !== assetOwner
      );
    });

  if (!canRevoke) {
    throw new Error(ERR_CANNOT_REVOKE);
  }

  const instructions: IInstruction[] = [];

  // Change the plugin authority of the defined plugins.
  Object.keys(definedPlugins).forEach((pluginKey) => {
    const plugType = pluginKeyToPluginType(pluginKey as keyof AssetPluginsList);

    instructions.push(
      getRevokePluginAuthorityV1Instruction({
        asset: asset.address,
        pluginType: PluginType[plugType],
      })
    );
  });

  return instructions;
}

test('it can revoke with one plugin defined', async (t) => {
  // Given an RPC instance and asset with one required plugin with delegated authority.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegateToRevokeFrom = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: { __kind: 'Address', address: delegateToRevokeFrom.address },
      }),
    ],
  });

  // The authority of all defined plugins becomes the asset owner.
  const assetWithPlugins = await fetchAssetV1(rpc, asset.address);
  const instructions = await legacyRevoke(rpc, assetWithPlugins);
  await sendAndConfirmInstructions(rpc, createRpcSubscriptions(), instructions, [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can revoke with all required plugins defined', async (t) => {
  // Given an RPC instance and asset with all required plugins with delegated authorities.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegateToRevokeFrom = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: { __kind: 'Address', address: delegateToRevokeFrom.address },
      }),
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: { __kind: 'Address', address: delegateToRevokeFrom.address },
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: { __kind: 'Address', address: delegateToRevokeFrom.address },
      }),
    ],
  });

  // The authority of all defined plugins becomes the asset owner.
  const assetWithPlugins = await fetchAssetV1(rpc, asset.address);
  const instructions = await legacyRevoke(rpc, assetWithPlugins);
  await sendAndConfirmInstructions(rpc, createRpcSubscriptions(), instructions, [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can revoke with a couple of non-homogenous plugin authorities', async (t) => {
  // Given an RPC instance and asset with a couple required plugins with non-homogenous plugin authorities.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegateToRevokeFrom = await generateKeyPairSigner();
  const delegateToRevokeFrom2 = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: { __kind: 'Address', address: delegateToRevokeFrom.address },
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: { __kind: 'Address', address: delegateToRevokeFrom2.address },
      }),
    ],
  });

  // The authority of all defined plugins becomes the asset owner.
  const assetWithPlugins = await fetchAssetV1(rpc, asset.address);
  const instructions = await legacyRevoke(rpc, assetWithPlugins);
  await sendAndConfirmInstructions(rpc, createRpcSubscriptions(), instructions, [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can revoke with all required plugins with non-homogenous plugin authorities', async (t) => {
  // Given an RPC instance and asset with all required plugin with non-homogenous plugin authorities.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegateToRevokeFrom = await generateKeyPairSigner();
  const delegateToRevokeFrom2 = await generateKeyPairSigner();
  const delegateToRevokeFrom3 = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        authority: { __kind: 'Address', address: delegateToRevokeFrom.address },
        data: {
          frozen: false,
        },
      }),
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: { __kind: 'Address', address: delegateToRevokeFrom2.address },
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: { __kind: 'Address', address: delegateToRevokeFrom3.address },
      }),
    ],
  });

  // The authority of all defined plugins becomes the asset owner.
  const assetWithPlugins = await fetchAssetV1(rpc, asset.address);
  const instructions = await legacyRevoke(rpc, assetWithPlugins);
  await sendAndConfirmInstructions(rpc, createRpcSubscriptions(), instructions, [payer]);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it cannot revoke if no plugins defined', async (t) => {
  // Given an RPC instance and asset with no plugins defined.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);

  // The revoke error is expected.
  const assetWithPlugins = await fetchAssetV1(rpc, asset.address);
  await t.throwsAsync(async () => {
    await legacyRevoke(rpc, assetWithPlugins);
  }, {
    message: ERR_CANNOT_REVOKE,
  });
});

test('it cannot revoke if one of plugin authorities is the asset owner', async (t) => {
  // Given an RPC instance and asset with all required plugins defined, and all the plugins except one have a delegated authority.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegateToRevokeFrom = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: { __kind: 'Address', address: delegateToRevokeFrom.address },
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: { __kind: 'Address', address: delegateToRevokeFrom.address },
      }),
    ],
  });

  // The revoke error is expected.
  const assetWithPlugins = await fetchAssetV1(rpc, asset.address);
  await t.throwsAsync(async () => {
    await legacyRevoke(rpc, assetWithPlugins);
  }, {
    message: ERR_CANNOT_REVOKE,
  });
});

test('it cannot revoke if all of the plugin authorities are the asset owner', async (t) => {
  // Given an RPC instance and asset with all required plugins defined with the asset owner as the plugin authority.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
      pluginAuthorityPair({
        type: 'TransferDelegate',
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
      }),
    ],
  });

  // The revoke error is expected.
  const assetWithPlugins = await fetchAssetV1(rpc, asset.address);
  await t.throwsAsync(async () => {
    await legacyRevoke(rpc, assetWithPlugins);
  }, {
    message: ERR_CANNOT_REVOKE,
  });
});
