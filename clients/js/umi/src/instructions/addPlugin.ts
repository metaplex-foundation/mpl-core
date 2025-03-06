import { Context } from '@metaplex-foundation/umi';
import { addPluginV1, addExternalPluginAdapterV1 } from '../generated';
import {
  AssetAddablePluginAuthorityPairArgsV2,
  ExternalPluginAdapterInitInfoArgs,
  createExternalPluginAdapterInitInfo,
  isExternalPluginAdapterType,
  pluginAuthorityPairV2,
} from '../plugins';

export type AddPluginArgsPlugin =
  | AssetAddablePluginAuthorityPairArgsV2
  | ExternalPluginAdapterInitInfoArgs;

export type AddPluginArgs = Omit<
  Parameters<typeof addPluginV1>[1],
  'plugin' | 'initAuthority'
> & {
  plugin: AddPluginArgsPlugin;
};

export const addPlugin = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, ...args }: AddPluginArgs
) => {
  if (isExternalPluginAdapterType(plugin)) {
    return addExternalPluginAdapterV1(context, {
      ...args,
      initInfo: createExternalPluginAdapterInitInfo(
        plugin as ExternalPluginAdapterInitInfoArgs
      ),
    });
  }

  const pair = pluginAuthorityPairV2(
    plugin as AssetAddablePluginAuthorityPairArgsV2
  );
  return addPluginV1(context, {
    ...args,
    plugin: pair.plugin,
    initAuthority: pair.authority,
  });
};
