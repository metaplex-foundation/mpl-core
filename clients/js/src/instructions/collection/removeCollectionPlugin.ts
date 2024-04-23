import { Context } from '@metaplex-foundation/umi';
import {
  PluginType,
  removeCollectionExternalPluginV1,
  removeCollectionPluginV1,
} from '../../generated';
import { ExternalPluginKey, externalPluginKeyToBase } from '../../plugins';

import { isExternalPluginType } from '../../plugins/externalPlugins';

export type RemoveCollectionPluginArgs = Omit<
  Parameters<typeof removeCollectionPluginV1>[1],
  'plugin'
> & {
  plugin:
    | {
        type: Exclude<keyof typeof PluginType, 'Edition'>;
      }
    | ExternalPluginKey;
};

export const removeCollectionPlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { plugin, ...args }: RemoveCollectionPluginArgs
) => {
  if (isExternalPluginType(plugin)) {
    return removeCollectionExternalPluginV1(context, {
      ...args,
      key: externalPluginKeyToBase(plugin as ExternalPluginKey),
    });
  }

  return removeCollectionPluginV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
  });
};
