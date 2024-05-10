import { Context } from '@metaplex-foundation/umi';
import { addPluginV1, addPluginAdapterV1 } from '../generated';
import {
  AddablePluginAuthorityPairArgsV2,
  PluginAdapterInitInfoArgs,
  createPluginAdapterInitInfo,
  isPluginAdapterType,
  pluginAuthorityPairV2,
} from '../plugins';

export type AddPluginArgsPlugin =
  | AddablePluginAuthorityPairArgsV2
  | PluginAdapterInitInfoArgs;

export type AddPluginArgs = Omit<
  Parameters<typeof addPluginV1>[1],
  'plugin' | 'initAuthority'
> & {
  plugin: AddPluginArgsPlugin;
};

export const addPlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, ...args }: AddPluginArgs
) => {
  if (isPluginAdapterType(plugin)) {
    return addPluginAdapterV1(context, {
      ...args,
      initInfo: createPluginAdapterInitInfo(
        plugin as PluginAdapterInitInfoArgs
      ),
    });
  }

  const pair = pluginAuthorityPairV2(
    plugin as AddablePluginAuthorityPairArgsV2
  );
  return addPluginV1(context, {
    ...args,
    plugin: pair.plugin,
    initAuthority: pair.authority,
  });
};
