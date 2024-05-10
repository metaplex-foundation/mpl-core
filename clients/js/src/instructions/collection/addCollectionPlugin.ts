import { Context } from '@metaplex-foundation/umi';
import {
  addCollectionExternalPluginAdapterV1,
  addCollectionPluginV1,
} from '../../generated';
import { AddablePluginArgsV2, pluginAuthorityPairV2 } from '../../plugins';

import {
  createExternalPluginAdapterInitInfo,
  ExternalPluginAdapterInitInfoArgs,
  isExternalPluginAdapterType,
} from '../../plugins/externalPluginAdapters';

export type AddCollectionPluginArgs = Omit<
  Parameters<typeof addCollectionPluginV1>[1],
  'plugin' | 'initAuthority'
> & {
  plugin:
    | Exclude<AddablePluginArgsV2, { type: 'Edition ' }>
    | ExternalPluginAdapterInitInfoArgs;
};

export const addCollectionPlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
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

  const pair = pluginAuthorityPairV2(plugin as AddablePluginArgsV2);
  return addCollectionPluginV1(context, {
    ...args,
    plugin: pair.plugin,
    initAuthority: pair.authority,
  });
};
