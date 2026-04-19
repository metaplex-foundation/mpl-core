import { Context } from '@metaplex-foundation/umi';
import { revokePluginAuthorityV1, PluginType } from '../generated';

export type RevokePluginAuthorityArgsPlugin = {
  type: Exclude<keyof typeof PluginType, 'Groups'>;
};

export type RevokePluginAuthorityArgs = Omit<
  Parameters<typeof revokePluginAuthorityV1>[1],
  'pluginType'
> & {
  plugin: RevokePluginAuthorityArgsPlugin;
};

export const revokePluginAuthority = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, ...args }: RevokePluginAuthorityArgs
) => {
  const pluginType = plugin.type as keyof typeof PluginType;
  if (pluginType === 'Groups') {
    throw new Error(
      'PluginType.Groups must be managed via group-specific instructions.'
    );
  }

  return revokePluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[pluginType],
  });
};
