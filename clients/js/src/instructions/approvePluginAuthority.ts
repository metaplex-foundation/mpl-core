import { Context } from '@metaplex-foundation/umi';
import { approvePluginAuthorityV1, PluginType } from '../generated';
import { PluginAuthority, pluginAuthorityToBase } from '../plugins';

export type ApprovePluginAuthorityArgsPlugin = {
  type: Exclude<keyof typeof PluginType, 'Groups'>;
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
) => {
  const pluginType = plugin.type as keyof typeof PluginType;
  if (pluginType === 'Groups') {
    throw new Error(
      'PluginType.Groups must be managed via group-specific instructions.'
    );
  }

  return approvePluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[pluginType],
    newAuthority: pluginAuthorityToBase(newAuthority),
  });
};
