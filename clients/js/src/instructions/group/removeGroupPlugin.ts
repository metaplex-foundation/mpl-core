import { Context } from '@metaplex-foundation/umi';
import {
  PluginType,
  removeGroupExternalPluginAdapterV1,
  removeGroupPluginV1,
} from '../../generated';
import { isExternalPluginAdapterType } from '../../plugins';
import {
  ExternalPluginAdapterKey,
  externalPluginAdapterKeyToBase,
} from '../../plugins/externalPluginAdapterKey';

export type RemoveGroupPluginArgsPlugin =
  | {
      type: Exclude<keyof typeof PluginType, 'Edition'>;
    }
  | ExternalPluginAdapterKey;

export type RemoveGroupPluginArgs = Omit<
  Parameters<typeof removeGroupPluginV1>[1],
  'pluginType'
> & {
  plugin: RemoveGroupPluginArgsPlugin;
};

export const removeGroupPlugin = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, ...args }: RemoveGroupPluginArgs
) => {
  if (isExternalPluginAdapterType(plugin)) {
    return removeGroupExternalPluginAdapterV1(context, {
      ...args,
      key: externalPluginAdapterKeyToBase(plugin as ExternalPluginAdapterKey),
    });
  }

  return removeGroupPluginV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
  });
};
