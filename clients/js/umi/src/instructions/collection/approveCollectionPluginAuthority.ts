import { Context } from '@metaplex-foundation/umi';
import {
  approveCollectionPluginAuthorityV1,
  PluginType,
} from '../../generated';
import { PluginAuthority, pluginAuthorityToBase } from '../../plugins';

export type ApproveCollectionPluginAuthorityArgs = Omit<
  Parameters<typeof approveCollectionPluginAuthorityV1>[1],
  'pluginType' | 'newAuthority'
> & {
  plugin: {
    type: keyof typeof PluginType;
  };
  newAuthority: PluginAuthority;
};

export const approveCollectionPluginAuthority = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, newAuthority, ...args }: ApproveCollectionPluginAuthorityArgs
) =>
  approveCollectionPluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
    newAuthority: pluginAuthorityToBase(newAuthority),
  });
