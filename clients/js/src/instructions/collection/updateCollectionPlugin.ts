import { Context } from '@metaplex-foundation/umi';
import {
  updateCollectionPluginV1,
  updateCollectionPluginAdapterV1,
} from '../../generated';
import {
  createPluginAdapterUpdateInfo,
  PluginArgsV2,
  createPluginV2,
  pluginAdapterKeyToBase,
  isPluginAdapterType,
} from '../../plugins';
import { PluginAdapterUpdateInfoArgs } from '../../plugins/pluginAdapters';

export type UpdateCollectionPluginArgs = Omit<
  Parameters<typeof updateCollectionPluginV1>[1],
  'plugin'
> & {
  plugin: PluginArgsV2 | PluginAdapterUpdateInfoArgs;
};

export const updateCollectionPlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, ...args }: UpdateCollectionPluginArgs
) => {
  if (isPluginAdapterType(plugin)) {
    const plug = plugin as PluginAdapterUpdateInfoArgs;
    return updateCollectionPluginAdapterV1(context, {
      ...args,
      updateInfo: createPluginAdapterUpdateInfo(plug),
      key: pluginAdapterKeyToBase(plug.key),
    });
  }

  return updateCollectionPluginV1(context, {
    ...args,
    plugin: createPluginV2(plugin as PluginArgsV2),
  });
};
