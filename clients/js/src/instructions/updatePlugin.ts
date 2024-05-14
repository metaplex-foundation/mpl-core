import { Context } from '@metaplex-foundation/umi';
import { updatePluginV1, updateExternalPluginAdapterV1 } from '../generated';
import {
  createExternalPluginAdapterUpdateInfo,
  createPluginV2,
  externalPluginAdapterKeyToBase,
  isExternalPluginAdapterType,
  AssetAllPluginArgsV2,
} from '../plugins';
import { ExternalPluginAdapterUpdateInfoArgs } from '../plugins/externalPluginAdapters';

export type UpdatePluginArgsPlugin =
  | AssetAllPluginArgsV2
  | ExternalPluginAdapterUpdateInfoArgs;

export type UpdatePluginArgs = Omit<
  Parameters<typeof updatePluginV1>[1],
  'plugin'
> & {
  plugin: UpdatePluginArgsPlugin;
};

export const updatePlugin = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, ...args }: UpdatePluginArgs
) => {
  if (isExternalPluginAdapterType(plugin)) {
    const plug = plugin as ExternalPluginAdapterUpdateInfoArgs;
    return updateExternalPluginAdapterV1(context, {
      ...args,
      updateInfo: createExternalPluginAdapterUpdateInfo(plug),
      key: externalPluginAdapterKeyToBase(plug.key),
    });
  }

  return updatePluginV1(context, {
    ...args,
    plugin: createPluginV2(plugin as AssetAllPluginArgsV2),
  });
};
