import { Context } from '@metaplex-foundation/umi';
import {
  addCollectionExternalPluginAdapterV1,
  addCollectionPluginV1,
} from '../../generated';
import {
  CollectionAddablePluginAuthorityPairArgsV2,
  pluginAuthorityPairV2,
} from '../../plugins';

import {
  createExternalPluginAdapterInitInfo,
  ExternalPluginAdapterInitInfoArgs,
  isExternalPluginAdapterType,
} from '../../plugins/externalPluginAdapters';

export type AddCollectionPluginArgsPlugin =
  | Exclude<CollectionAddablePluginAuthorityPairArgsV2, { type: 'Edition ' }>
  | ExternalPluginAdapterInitInfoArgs;

export type AddCollectionPluginArgs = Omit<
  Parameters<typeof addCollectionPluginV1>[1],
  'plugin' | 'initAuthority'
> & {
  plugin: AddCollectionPluginArgsPlugin;
};

export const addCollectionPlugin = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, ...args }: AddCollectionPluginArgs
) => {
  if (isExternalPluginAdapterType(plugin)) {
    return addCollectionExternalPluginAdapterV1(context, {
      ...args,
      initInfo: createExternalPluginAdapterInitInfo(
        plugin as ExternalPluginAdapterInitInfoArgs
      ),
    });
  }

  const pair = pluginAuthorityPairV2(
    plugin as CollectionAddablePluginAuthorityPairArgsV2
  );
  return addCollectionPluginV1(context, {
    ...args,
    plugin: pair.plugin,
    initAuthority: pair.authority,
  });
};
