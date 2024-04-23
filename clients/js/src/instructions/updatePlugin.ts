import { Context } from '@metaplex-foundation/umi';
import { updatePluginV1, updateExternalPluginV1 } from '../generated';
import {
  createExternalPluginUpdateInfo,
  createPluginV2,
  externalPluginKeyToBase,
  isExternalPluginType,
  PluginArgsV2,
} from '../plugins';
import { ExternalPluginUpdateInfoArgs } from '../plugins/externalPlugins';

export type UpdatePluginArgs = Omit<
  Parameters<typeof updatePluginV1>[1],
  'plugin'
> & {
  plugin: PluginArgsV2 | ExternalPluginUpdateInfoArgs;
};

export const updatePlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, ...args }: UpdatePluginArgs
) => {
  if (isExternalPluginType(plugin)) {
    const plug = plugin as ExternalPluginUpdateInfoArgs;
    return updateExternalPluginV1(context, {
      ...args,
      updateInfo: createExternalPluginUpdateInfo(plug),
      key: externalPluginKeyToBase(plug.key),
    });
  }

  return updatePluginV1(context, {
    ...args,
    plugin: createPluginV2(plugin as PluginArgsV2),
  });
};
