import { Context } from '@metaplex-foundation/umi';
import { revokePluginAuthorityV1, PluginType } from '../generated';

export type RevokePluginAuthorityArgsPlugin = {
  type: keyof typeof PluginType;
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
) =>
  revokePluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
  });
