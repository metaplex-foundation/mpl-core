import { Context } from '@metaplex-foundation/umi';
import {
  PluginType,
  removeCollectionExternalPluginAdapterV1,
  removeCollectionPluginV1,
} from '../../generated';
import {
  ExternalPluginAdapterKey,
  externalPluginAdapterKeyToBase,
} from '../../plugins';

import { isExternalPluginAdapterType } from '../../plugins/externalPluginAdapters';

export type RemoveCollectionPluginArgs = Omit<
  Parameters<typeof removeCollectionPluginV1>[1],
  'plugin'
> & {
  plugin:
    | {
        type: Exclude<keyof typeof PluginType, 'Edition'>;
      }
    | ExternalPluginAdapterKey;
};

export const removeCollectionPlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { plugin, ...args }: RemoveCollectionPluginArgs
) => {
  if (isExternalPluginAdapterType(plugin)) {
    return removeCollectionExternalPluginAdapterV1(context, {
      ...args,
      key: externalPluginAdapterKeyToBase(plugin as ExternalPluginAdapterKey),
    });
  }

  return removeCollectionPluginV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
  });
};
