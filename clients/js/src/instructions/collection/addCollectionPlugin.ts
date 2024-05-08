import { Context } from '@metaplex-foundation/umi';
import {
  addCollectionExternalPluginV1,
  addCollectionPluginV1,
} from '../../generated';
import { AddablePluginArgsV2, pluginAuthorityPairV2 } from '../../plugins';

import {
  createExternalPluginInitInfo,
  ExternalPluginInitInfoArgs,
  isExternalPluginType,
} from '../../plugins/externalPlugins';

export type AddCollectionPluginArgs = Omit<
  Parameters<typeof addCollectionPluginV1>[1],
  'plugin' | 'initAuthority'
> & {
  plugin:
    | Exclude<AddablePluginArgsV2, { type: 'Edition ' }>
    | ExternalPluginInitInfoArgs;
};

export const addCollectionPlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { plugin, ...args }: AddCollectionPluginArgs
) => {
  if (isExternalPluginType(plugin)) {
    return addCollectionExternalPluginV1(context, {
      ...args,
      initInfo: createExternalPluginInitInfo(
        plugin as ExternalPluginInitInfoArgs
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
