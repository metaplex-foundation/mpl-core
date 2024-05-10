import { Context } from '@metaplex-foundation/umi';
import { addPluginV1, addExternalPluginAdapterV1 } from '../generated';
import {
  AddablePluginAuthorityPairArgsV2,
  ExternalPluginAdapterInitInfoArgs,
  createExternalPluginAdapterInitInfo,
  isExternalPluginAdapterType,
  pluginAuthorityPairV2,
} from '../plugins';

export type AddPluginArgsPlugin =
  | AddablePluginAuthorityPairArgsV2
  | ExternalPluginAdapterInitInfoArgs;

export type AddPluginArgs = Omit<
  Parameters<typeof addPluginV1>[1],
  'plugin' | 'initAuthority'
> & {
  plugin: AddPluginArgsPlugin;
};

export const addPlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, ...args }: AddPluginArgs
) => {
  if (isExternalPluginAdapterType(plugin)) {
    return addExternalPluginAdapterV1(context, {
      ...args,
      initInfo: createExternalPluginAdapterInitInfo(
        plugin as ExternalPluginAdapterInitInfoArgs
      ),
    });
  }

  const pair = pluginAuthorityPairV2(
    plugin as AddablePluginAuthorityPairArgsV2
  );
  return addPluginV1(context, {
    ...args,
    plugin: pair.plugin,
    initAuthority: pair.authority,
  });
};
