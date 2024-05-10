import { Context } from '@metaplex-foundation/umi';
import { updatePluginV1, updatePluginAdapterV1 } from '../generated';
import {
  createPluginAdapterUpdateInfo,
  createPluginV2,
  pluginAdapterKeyToBase,
  isPluginAdapterType,
  PluginArgsV2,
} from '../plugins';
import { PluginAdapterUpdateInfoArgs } from '../plugins/pluginAdapters';

export type UpdatePluginArgsPlugin =
  | PluginArgsV2
  | PluginAdapterUpdateInfoArgs;

export type UpdatePluginArgs = Omit<
  Parameters<typeof updatePluginV1>[1],
  'plugin'
> & {
  plugin: UpdatePluginArgsPlugin;
};

export const updatePlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, ...args }: UpdatePluginArgs
) => {
  if (isPluginAdapterType(plugin)) {
    const plug = plugin as PluginAdapterUpdateInfoArgs;
    return updatePluginAdapterV1(context, {
      ...args,
      updateInfo: createPluginAdapterUpdateInfo(plug),
      key: pluginAdapterKeyToBase(plug.key),
    });
  }

  return updatePluginV1(context, {
    ...args,
    plugin: createPluginV2(plugin as PluginArgsV2),
  });
};
