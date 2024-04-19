import { Context } from '@metaplex-foundation/umi';
import {
  updateCollectionPluginV1,
  updateCollectionExternalPluginV1,
} from '../../generated';
import {
  createExternalPluginUpdateInfo,
  CreatePluginArgsV2,
  createPluginV2,
  externalPluginKeyToBase,
  isExternalPluginType,
} from '../../plugins';
import { ExternalPluginUpdateInfoArgs } from '../../plugins/externalPlugins';

export type UpdateCollectionPluginArgs = Omit<
  Parameters<typeof updateCollectionPluginV1>[1],
  'plugin'
> & {
  plugin: CreatePluginArgsV2 | ExternalPluginUpdateInfoArgs;
};

export const updateCollectionPlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, ...args }: UpdateCollectionPluginArgs
) => {
  if (isExternalPluginType(plugin)) {
    const plug = plugin as ExternalPluginUpdateInfoArgs;
    return updateCollectionExternalPluginV1(context, {
      ...args,
      updateInfo: createExternalPluginUpdateInfo(plug),
      pluginKey: externalPluginKeyToBase(plug.key),
    });
  }

  return updateCollectionPluginV1(context, {
    ...args,
    plugin: createPluginV2(plugin as CreatePluginArgsV2),
  });
};
