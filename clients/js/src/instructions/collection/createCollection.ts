import { Context } from '@metaplex-foundation/umi';
import { createCollectionV2 } from '../../generated';
import {
  PluginAuthorityPairHelperArgsV2,
  createExternalPluginInitInfo,
  pluginAuthorityPairV2,
} from '../../plugins';

import { ExternalPluginInitInfoArgs } from '../../plugins/externalPlugins';

export type CreateCollectionArgs = Omit<
  Parameters<typeof createCollectionV2>[1],
  'plugins' | 'externalPlugins'
> & {
  plugins?: PluginAuthorityPairHelperArgsV2[];
  externalPlugins?: ExternalPluginInitInfoArgs[];
};

export const createCollection = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { plugins, externalPlugins, ...args }: CreateCollectionArgs
) =>
  createCollectionV2(context, {
    ...args,
    plugins: plugins?.map(pluginAuthorityPairV2),
    externalPlugins: externalPlugins?.map(createExternalPluginInitInfo),
  });
