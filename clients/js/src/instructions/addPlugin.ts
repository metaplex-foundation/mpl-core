import { Context } from '@metaplex-foundation/umi';
import { addPluginV1, addExternalPluginV1 } from '../generated';
import {
  AddablePluginAuthorityPairArgsV2,
  ExternalPluginInitInfoArgs,
  createExternalPluginInitInfo,
  isExternalPluginType,
  pluginAuthorityPairV2,
} from '../plugins';

export type AddPluginArgs = Omit<
  Parameters<typeof addPluginV1>[1],
  'plugin'
> & {
  plugin: AddablePluginAuthorityPairArgsV2 | ExternalPluginInitInfoArgs;
};

export const addPlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, ...args }: AddPluginArgs
) => {
  if (isExternalPluginType(plugin)) {
    return addExternalPluginV1(context, {
      ...args,
      initInfo: createExternalPluginInitInfo(
        plugin as ExternalPluginInitInfoArgs
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
