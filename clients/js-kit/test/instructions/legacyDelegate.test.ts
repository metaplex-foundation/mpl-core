import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import type { Address } from '@solana/addresses';
import {
  getAddPluginV1Instruction,
  getApprovePluginAuthorityV1Instruction,
  fetchAssetV1,
  type AssetV1,
  type Account,
} from '../../src';
import {
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../_setup';
import { pluginAuthorityPair } from '../../src/plugins';

/**
 * Helper function to implement legacy delegate behavior
 * This mimics the old legacyDelegate function from the Umi client
 */
async function legacyDelegate(
  rpc: ReturnType<typeof createRpc>,
  asset: Account<AssetV1>,
  targetDelegate: Address,
  payer: Awaited<ReturnType<typeof generateSignerWithSol>>
) {
  const rpcSubscriptions = createRpcSubscriptions();
  const assetData = asset.data;

  // Check if delegation is allowed
  const definedPlugins = {
    ...(assetData.freezeDelegate ? { freezeDelegate: assetData.freezeDelegate } : {}),
    ...(assetData.transferDelegate ? { transferDelegate: assetData.transferDelegate } : {}),
    ...(assetData.burnDelegate ? { burnDelegate: assetData.burnDelegate } : {}),
  };

  const canDelegate = Object.values(definedPlugins).every((pluginValue) => {
    const assetOwner = assetData.owner;
    const pluginAuthority = pluginValue.authority;

    if (pluginAuthority.type === 'Owner') {
      return targetDelegate !== assetOwner;
    }

    if (pluginAuthority.type === 'Address') {
      const pluginAuthorityAddress = pluginAuthority.address;
      return (
        pluginAuthorityAddress !== targetDelegate &&
        pluginAuthorityAddress !== assetOwner
      );
    }

    return true;
  });

  if (!canDelegate) {
    throw new Error('Cannot delegate: target delegate is already set as owner or authority');
  }

  const instructions = [];
  const definedPluginKeys = Object.keys(definedPlugins);

  // Change the plugin authority of the defined plugins
  for (const pluginKey of definedPluginKeys) {
    const plugType = pluginKey === 'freezeDelegate'
      ? 'FreezeDelegate'
      : pluginKey === 'transferDelegate'
      ? 'TransferDelegate'
      : 'BurnDelegate';

    instructions.push(
      getApprovePluginAuthorityV1Instruction({
        asset: asset.address,
        payer,
        plugin: { __kind: plugType },
        newAuthority: {
          __kind: 'Address',
          fields: [targetDelegate],
        },
      })
    );
  }

  // Add missing plugins with the new plugin authority
  const requiredPlugins = ['freezeDelegate', 'transferDelegate', 'burnDelegate'];
  const missingPlugins = requiredPlugins.filter(
    (requiredPlugin) => !definedPluginKeys.includes(requiredPlugin)
  );

  for (const missingPlugin of missingPlugins) {
    let pluginType: 'FreezeDelegate' | 'TransferDelegate' | 'BurnDelegate';
    let plugin;

    if (missingPlugin === 'freezeDelegate') {
      pluginType = 'FreezeDelegate';
      plugin = {
        __kind: pluginType,
        fields: [{ frozen: false }],
      };
    } else if (missingPlugin === 'transferDelegate') {
      pluginType = 'TransferDelegate';
      plugin = {
        __kind: pluginType,
        fields: [{}],
      };
    } else {
      pluginType = 'BurnDelegate';
      plugin = {
        __kind: pluginType,
        fields: [{}],
      };
    }

    instructions.push(
      getAddPluginV1Instruction({
        asset: asset.address,
        payer,
        plugin,
        initAuthority: {
          __option: 'Some',
          value: {
            __kind: 'Address',
            fields: [targetDelegate],
          },
        },
      })
    );
  }

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, instructions, [payer]);
}

test('it can delegate with a new delegate address and no defined plugins', async (t) => {
  // Given an Umi instance, asset with no plugins and new delegate.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer);
  const delegate = await generateKeyPairSigner();

  // It adds all required plugins and sets the new delegate as their authority.
  await legacyDelegate(rpc, asset, delegate.address, payer);

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
    transferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
    },
    burnDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
    },
  });
});

test('it can delegate with a new delegate address and one defined plugin', async (t) => {
  // Given an Umi instance, asset with one defined plugin and new delegate.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'BurnDelegate',
      }),
    ],
  });
  const delegate = await generateKeyPairSigner();

  // It adds all missing plugins and sets the new delegate as the authority of all required plugins.
  await legacyDelegate(rpc, asset, delegate.address, payer);

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
    transferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
    },
    burnDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
    },
  });
});

test('it can delegate with a new delegate address and all required plugins defined', async (t) => {
  // Given an Umi instance, asset with all required plugins and new delegate.
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
  const delegate = await generateKeyPairSigner();

  // It sets the new delegate as the authority of all defined plugins.
  await legacyDelegate(rpc, asset, delegate.address, payer);

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
    transferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
    },
    burnDelegate: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
    },
  });
});

test('it cannot delegate if the target delegate is the asset owner', async (t) => {
  // Given an Umi instance and an asset with the owner plugin authority.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  // The delegate error is expected.
  await t.throwsAsync(
    async () => {
      await legacyDelegate(rpc, asset, asset.data.owner, payer);
    }
  );
});

test('it cannot delegate if the target delegate is the asset owner and the plugin authority is an explicit public key of the asset owner', async (t) => {
  // Given an Umi instance and an asset with an explicit public key of the asset owner as the plugin authority.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: { __kind: 'Address', address: payer.address },
      }),
    ],
  });

  // The delegate error is expected.
  await t.throwsAsync(
    async () => {
      await legacyDelegate(rpc, asset, asset.data.owner, payer);
    }
  );
});

test('it cannot delegate if the target delegate is already set as an authority of a plugin', async (t) => {
  // Given an Umi instance and an asset with the target delegate as the initial plugin authority.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const delegate = await generateKeyPairSigner();
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: { __kind: 'Address', address: delegate.address },
      }),
    ],
  });

  // The delegate error is expected.
  await t.throwsAsync(
    async () => {
      await legacyDelegate(rpc, asset, delegate.address, payer);
    }
  );
});
