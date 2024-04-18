import { Context } from '@metaplex-foundation/umi';
import {
  removePluginV1,
  removeExternalPluginV1,
  PluginType,
} from '../generated';
import { isExternalPluginType } from '../plugins';
import {
  ExternalPluginKey,
  externalPluginKeyToBase,
} from '../plugins/externalPluginKey';

export type RemovePluginArgs = Omit<
  Parameters<typeof removePluginV1>[1],
  'pluginType'
> & {
  plugin:
    | {
        type: keyof typeof PluginType;
      }
    | ExternalPluginKey;
};

export const removePlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, ...args }: RemovePluginArgs
) => {
  if (isExternalPluginType(plugin)) {
    return removeExternalPluginV1(context, {
      ...args,
      key: externalPluginKeyToBase(plugin as ExternalPluginKey),
    });
  }

  return removePluginV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
  });
};
