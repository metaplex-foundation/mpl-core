import { Context } from '@metaplex-foundation/umi';
import {
  removePluginV1,
  removeExternalPluginAdapterV1,
  PluginType,
} from '../generated';
import { isExternalPluginAdapterType } from '../plugins';
import {
  ExternalPluginAdapterKey,
  externalPluginAdapterKeyToBase,
} from '../plugins/externalPluginAdapterKey';

export type RemovePluginArgsPlugin =
  | {
      type: Exclude<keyof typeof PluginType, 'Edition' | 'Groups'>;
    }
  | ExternalPluginAdapterKey;

export type RemovePluginArgs = Omit<
  Parameters<typeof removePluginV1>[1],
  'pluginType'
> & {
  plugin: RemovePluginArgsPlugin;
};

export const removePlugin = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, ...args }: RemovePluginArgs
) => {
  if (isExternalPluginAdapterType(plugin)) {
    return removeExternalPluginAdapterV1(context, {
      ...args,
      key: externalPluginAdapterKeyToBase(plugin as ExternalPluginAdapterKey),
    });
  }

  const pluginType = plugin.type as keyof typeof PluginType;
  if (pluginType === 'Groups') {
    throw new Error(
      'PluginType.Groups must be managed via group-specific instructions.'
    );
  }

  return removePluginV1(context, {
    ...args,
    pluginType: PluginType[pluginType],
  });
};
