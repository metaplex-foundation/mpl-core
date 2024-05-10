import { Context } from '@metaplex-foundation/umi';
import { createCollectionV2 } from '../../generated';
import {
  PluginAuthorityPairArgsV2,
  createPluginAdapterInitInfo,
  pluginAuthorityPairV2,
} from '../../plugins';

import {
  PluginAdapterInitInfoArgs,
  isPluginAdapterType,
} from '../../plugins/pluginAdapters';

export type CreateCollectionArgs = Omit<
  Parameters<typeof createCollectionV2>[1],
  'plugins' | 'pluginAdapters'
> & {
  plugins?: (PluginAuthorityPairArgsV2 | PluginAdapterInitInfoArgs)[];
};

export const createCollection = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { plugins, ...args }: CreateCollectionArgs
) => {
  const firstPartyPlugins: PluginAuthorityPairArgsV2[] = [];
  const pluginAdapters: PluginAdapterInitInfoArgs[] = [];

  plugins?.forEach((plugin) => {
    if (isPluginAdapterType(plugin)) {
      pluginAdapters.push(plugin as PluginAdapterInitInfoArgs);
    } else {
      firstPartyPlugins.push(plugin as PluginAuthorityPairArgsV2);
    }
  });

  return createCollectionV2(context, {
    ...args,
    plugins: firstPartyPlugins.map(pluginAuthorityPairV2),
    pluginAdapters: pluginAdapters.map(createPluginAdapterInitInfo),
  });
};
