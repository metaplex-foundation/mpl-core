import {
  Context,
  PublicKey,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import { ERR_CANNOT_DELEGATE } from './errors';
import {
  addPluginV1,
  approvePluginAuthorityV1,
  AssetV1,
  PluginType,
} from '../generated';
import { createPlugin, pluginKeyToPluginType, PluginsList } from '../plugins';
import { addressPluginAuthority } from '../authority';

export function legacyDelegate(
  context: Pick<Context, 'payer' | 'programs'>,
  asset: AssetV1,
  targetDelegate: PublicKey
) {
  const definedPlugins = (({
    freezeDelegate,
    transferDelegate,
    burnDelegate,
  }) => ({
    ...(freezeDelegate ? { freezeDelegate } : {}),
    ...(transferDelegate ? { transferDelegate } : {}),
    ...(burnDelegate ? { burnDelegate } : {}),
  }))(asset);

  const canDelegate = Object.values(definedPlugins).every((pluginValue) => {
    const assetOwner = asset.owner;
    const pluginAuthority = pluginValue.authority;
    const pluginAuthorityAddress = pluginAuthority.address;

    if (pluginAuthority.type === 'Owner') return targetDelegate !== assetOwner;

    return (
      pluginAuthorityAddress !== targetDelegate &&
      pluginAuthorityAddress !== assetOwner
    );
  });

  if (!canDelegate) {
    throw new Error(ERR_CANNOT_DELEGATE);
  }

  let txBuilder = transactionBuilder();
  const definedPluginsKeys = Object.keys(definedPlugins);

  // Change the plugin authority of the defined plugins.
  definedPluginsKeys.forEach((pluginKey) => {
    const plugType = pluginKeyToPluginType(pluginKey as keyof PluginsList);

    txBuilder = txBuilder.add(
      approvePluginAuthorityV1(context, {
        asset: asset.publicKey,
        pluginType: PluginType[plugType],
        newAuthority: addressPluginAuthority(targetDelegate),
      })
    );
  });

  // Add missing plugins with a new plugin authority.
  const requiredPlugins = [
    'freezeDelegate',
    'transferDelegate',
    'burnDelegate',
  ];
  const missingPlugins = requiredPlugins.filter(
    (requiredPlugin) => !definedPluginsKeys.includes(requiredPlugin)
  );
  missingPlugins.forEach((missingPlugin) => {
    const plugin = (() => {
      if (missingPlugin === 'freezeDelegate') {
        return createPlugin({
          type: 'FreezeDelegate',
          data: { frozen: false },
        });
      }

      if (missingPlugin === 'transferDelegate') {
        return createPlugin({ type: 'TransferDelegate' });
      }

      return createPlugin({ type: 'BurnDelegate' });
    })();

    txBuilder = txBuilder.add(
      addPluginV1(context, {
        asset: asset.publicKey,
        plugin,
        initAuthority: addressPluginAuthority(targetDelegate),
      })
    );
  });

  return txBuilder;
}
