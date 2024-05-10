import { Context } from '@metaplex-foundation/umi';
import {
  PluginType,
  removeCollectionPluginAdapterV1,
  removeCollectionPluginV1,
} from '../../generated';
import { PluginAdapterKey, pluginAdapterKeyToBase } from '../../plugins';

import { isPluginAdapterType } from '../../plugins/pluginAdapters';

export type RemoveCollectionPluginArgs = Omit<
  Parameters<typeof removeCollectionPluginV1>[1],
  'plugin'
> & {
  plugin:
    | {
        type: Exclude<keyof typeof PluginType, 'Edition'>;
      }
    | PluginAdapterKey;
};

export const removeCollectionPlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { plugin, ...args }: RemoveCollectionPluginArgs
) => {
  if (isPluginAdapterType(plugin)) {
    return removeCollectionPluginAdapterV1(context, {
      ...args,
      key: pluginAdapterKeyToBase(plugin as PluginAdapterKey),
    });
  }

  return removeCollectionPluginV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
  });
};
