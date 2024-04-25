import { Context } from '@metaplex-foundation/umi';
import { createCollectionV2 } from '../../generated';
import {
  PluginAuthorityPairArgsV2,
  createExternalPluginInitInfo,
  pluginAuthorityPairV2,
} from '../../plugins';

import {
  ExternalPluginInitInfoArgs,
  isExternalPluginType,
} from '../../plugins/externalPlugins';

export type CreateCollectionArgs = Omit<
  Parameters<typeof createCollectionV2>[1],
  'plugins' | 'externalPlugins'
> & {
  plugins?: (PluginAuthorityPairArgsV2 | ExternalPluginInitInfoArgs)[];
};

export const createCollection = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { plugins, ...args }: CreateCollectionArgs
) => {
  const firstPartyPlugins: PluginAuthorityPairArgsV2[] = [];
  const externalPlugins: ExternalPluginInitInfoArgs[] = [];

  plugins?.forEach((plugin) => {
    if (isExternalPluginType(plugin)) {
      externalPlugins.push(plugin as ExternalPluginInitInfoArgs);
    } else {
      firstPartyPlugins.push(plugin as PluginAuthorityPairArgsV2);
    }
  });

  return createCollectionV2(context, {
    ...args,
    plugins: firstPartyPlugins.map(pluginAuthorityPairV2),
    externalPlugins: externalPlugins.map(createExternalPluginInitInfo),
  });
};
