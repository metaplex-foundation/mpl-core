import { Context } from '@metaplex-foundation/umi';
import {
  removePluginV1,
  removePluginAdapterV1,
  PluginType,
} from '../generated';
import { isPluginAdapterType } from '../plugins';
import {
  PluginAdapterKey,
  pluginAdapterKeyToBase,
} from '../plugins/pluginAdapterKey';

export type RemovePluginArgsPlugin =
  | {
      type: Exclude<keyof typeof PluginType, 'Edition'>;
    }
  | PluginAdapterKey;

export type RemovePluginArgs = Omit<
  Parameters<typeof removePluginV1>[1],
  'pluginType'
> & {
  plugin: RemovePluginArgsPlugin;
};

export const removePlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, ...args }: RemovePluginArgs
) => {
  if (isPluginAdapterType(plugin)) {
    return removePluginAdapterV1(context, {
      ...args,
      key: pluginAdapterKeyToBase(plugin as PluginAdapterKey),
    });
  }

  return removePluginV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
  });
};
