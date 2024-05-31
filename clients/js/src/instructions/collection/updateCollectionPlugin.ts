import { Context } from '@metaplex-foundation/umi';
import {
  updateCollectionPluginV1,
  updateCollectionExternalPluginAdapterV1,
} from '../../generated';
import {
  createExternalPluginAdapterUpdateInfo,
  createPluginV2,
  externalPluginAdapterKeyToBase,
  isExternalPluginAdapterType,
  CollectionAllPluginArgsV2,
} from '../../plugins';
import { ExternalPluginAdapterUpdateInfoArgs } from '../../plugins/externalPluginAdapters';

export type UpdateCollectionPluginArgsPlugin =
  | CollectionAllPluginArgsV2
  | ExternalPluginAdapterUpdateInfoArgs;

export type UpdateCollectionPluginArgs = Omit<
  Parameters<typeof updateCollectionPluginV1>[1],
  'plugin'
> & {
  plugin: UpdateCollectionPluginArgsPlugin;
};

export const updateCollectionPlugin = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, ...args }: UpdateCollectionPluginArgs
) => {
  if (isExternalPluginAdapterType(plugin)) {
    const plug = plugin as ExternalPluginAdapterUpdateInfoArgs;
    return updateCollectionExternalPluginAdapterV1(context, {
      ...args,
      updateInfo: createExternalPluginAdapterUpdateInfo(plug),
      key: externalPluginAdapterKeyToBase(plug.key),
    });
  }

  return updateCollectionPluginV1(context, {
    ...args,
    plugin: createPluginV2(plugin as CollectionAllPluginArgsV2),
  });
};
