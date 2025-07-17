import { Context } from '@metaplex-foundation/umi';
import { PluginType, revokeGroupPluginAuthorityV1 } from '../../generated';

export type RevokeGroupPluginAuthorityArgsPlugin = {
  type: keyof typeof PluginType;
};

export type RevokeGroupPluginAuthorityArgs = Omit<
  Parameters<typeof revokeGroupPluginAuthorityV1>[1],
  'pluginType'
> & {
  plugin: RevokeGroupPluginAuthorityArgsPlugin;
};

export const revokeGroupPluginAuthority = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, ...args }: RevokeGroupPluginAuthorityArgs
) =>
  revokeGroupPluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
  });
