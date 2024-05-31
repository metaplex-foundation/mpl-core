import { Context, transactionBuilder } from '@metaplex-foundation/umi';
import { AssetV1, PluginType, revokePluginAuthorityV1 } from '../generated';
import { ERR_CANNOT_REVOKE } from './errors';
import { AssetPluginsList, pluginKeyToPluginType } from '../plugins';

export function legacyRevoke(
  context: Pick<Context, 'payer' | 'programs'>,
  asset: AssetV1
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

  const definedPluginsValues = Object.values(definedPlugins);
  const canRevoke =
    definedPluginsValues.length > 0 &&
    definedPluginsValues.every((pluginValue) => {
      const assetOwner = asset.owner;
      const pluginAuthority = pluginValue.authority;

      return (
        pluginAuthority.type !== 'Owner' &&
        pluginAuthority.address !== assetOwner
      );
    });

  if (!canRevoke) {
    throw new Error(ERR_CANNOT_REVOKE);
  }

  let txBuilder = transactionBuilder();

  // Change the plugin authority of the defined plugins.
  Object.keys(definedPlugins).forEach((pluginKey) => {
    const plugType = pluginKeyToPluginType(pluginKey as keyof AssetPluginsList);

    txBuilder = txBuilder.add(
      revokePluginAuthorityV1(context, {
        asset: asset.publicKey,
        pluginType: PluginType[plugType],
      })
    );
  });

  return txBuilder;
}
