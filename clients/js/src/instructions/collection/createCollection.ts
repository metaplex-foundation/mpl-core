import { Context } from '@metaplex-foundation/umi';
import { createCollectionV2 } from '../../generated';
import {
  PluginAuthorityPairArgsV2,
  createExternalPluginAdapterInitInfo,
  pluginAuthorityPairV2,
} from '../../plugins';

import {
  ExternalPluginAdapterInitInfoArgs,
  isExternalPluginAdapterType,
} from '../../plugins/externalPluginAdapters';

export type CreateCollectionArgs = Omit<
  Parameters<typeof createCollectionV2>[1],
  'plugins' | 'externalPluginAdapters'
> & {
  plugins?: (PluginAuthorityPairArgsV2 | ExternalPluginAdapterInitInfoArgs)[];
};

export const createCollection = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { plugins, ...args }: CreateCollectionArgs
) => {
  const firstPartyPlugins: PluginAuthorityPairArgsV2[] = [];
  const externalPluginAdapters: ExternalPluginAdapterInitInfoArgs[] = [];

  plugins?.forEach((plugin) => {
    if (isExternalPluginAdapterType(plugin)) {
      externalPluginAdapters.push(plugin as ExternalPluginAdapterInitInfoArgs);
    } else {
      firstPartyPlugins.push(plugin as PluginAuthorityPairArgsV2);
    }
  });

  return createCollectionV2(context, {
    ...args,
    plugins: firstPartyPlugins.map(pluginAuthorityPairV2),
    externalPluginAdapters: externalPluginAdapters.map(
      createExternalPluginAdapterInitInfo
    ),
  });
};
