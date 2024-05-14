import { Context } from '@metaplex-foundation/umi';
import { approvePluginAuthorityV1, PluginType } from '../generated';
import { PluginAuthority, pluginAuthorityToBase } from '../plugins';

export type ApprovePluginAuthorityArgsPlugin = {
  type: keyof typeof PluginType;
};

export type ApprovePluginAuthorityArgs = Omit<
  Parameters<typeof approvePluginAuthorityV1>[1],
  'pluginType' | 'newAuthority'
> & {
  plugin: ApprovePluginAuthorityArgsPlugin;
  newAuthority: PluginAuthority;
};

export const approvePluginAuthority = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, newAuthority, ...args }: ApprovePluginAuthorityArgs
) =>
  approvePluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
    newAuthority: pluginAuthorityToBase(newAuthority),
  });
