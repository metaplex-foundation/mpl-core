import { Context } from '@metaplex-foundation/umi';
import {
  addCollectionPluginAdapterV1,
  addCollectionPluginV1,
} from '../../generated';
import { AddablePluginArgsV2, pluginAuthorityPairV2 } from '../../plugins';

import {
  createPluginAdapterInitInfo,
  PluginAdapterInitInfoArgs,
  isPluginAdapterType,
} from '../../plugins/pluginAdapters';

export type AddCollectionPluginArgs = Omit<
  Parameters<typeof addCollectionPluginV1>[1],
  'plugin' | 'initAuthority'
> & {
  plugin:
    | Exclude<AddablePluginArgsV2, { type: 'Edition ' }>
    | PluginAdapterInitInfoArgs;
};

export const addCollectionPlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { plugin, ...args }: AddCollectionPluginArgs
) => {
  if (isPluginAdapterType(plugin)) {
    return addCollectionPluginAdapterV1(context, {
      ...args,
      initInfo: createPluginAdapterInitInfo(
        plugin as PluginAdapterInitInfoArgs
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
