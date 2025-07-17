import { Context } from '@metaplex-foundation/umi';
import { approveGroupPluginAuthorityV1, PluginType } from '../../generated';
import { PluginAuthority, pluginAuthorityToBase } from '../../plugins';

export type ApproveGroupPluginAuthorityArgsPlugin = {
  type: keyof typeof PluginType;
};

export type ApproveGroupPluginAuthorityArgs = Omit<
  Parameters<typeof approveGroupPluginAuthorityV1>[1],
  'pluginType' | 'newAuthority'
> & {
  plugin: ApproveGroupPluginAuthorityArgsPlugin;
  newAuthority: PluginAuthority;
};

export const approveGroupPluginAuthority = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, newAuthority, ...args }: ApproveGroupPluginAuthorityArgs
) =>
  approveGroupPluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
    newAuthority: pluginAuthorityToBase(newAuthority),
  });
