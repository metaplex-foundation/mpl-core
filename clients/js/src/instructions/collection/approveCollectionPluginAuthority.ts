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
    type: Exclude<keyof typeof PluginType, 'Groups'>;
  };
  newAuthority: PluginAuthority;
};

export const approveCollectionPluginAuthority = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, newAuthority, ...args }: ApproveCollectionPluginAuthorityArgs
) => {
  const pluginType = plugin.type as keyof typeof PluginType;
  if (pluginType === 'Groups') {
    throw new Error(
      'PluginType.Groups must be managed via group-specific instructions.'
    );
  }

  return approveCollectionPluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[pluginType],
    newAuthority: pluginAuthorityToBase(newAuthority),
  });
};
