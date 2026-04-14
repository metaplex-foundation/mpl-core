import { Context } from '@metaplex-foundation/umi';
import { revokeCollectionPluginAuthorityV1, PluginType } from '../../generated';

export type RevokeCollectionPluginAuthorityArgs = Omit<
  Parameters<typeof revokeCollectionPluginAuthorityV1>[1],
  'pluginType'
> & {
  plugin: {
    type: Exclude<keyof typeof PluginType, 'Groups'>;
  };
};

export const revokeCollectionPluginAuthority = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, ...args }: RevokeCollectionPluginAuthorityArgs
) => {
  const pluginType = plugin.type as keyof typeof PluginType;
  if (pluginType === 'Groups') {
    throw new Error(
      'PluginType.Groups must be managed via group-specific instructions.'
    );
  }

  return revokeCollectionPluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[pluginType],
  });
};
