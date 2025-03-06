import { Context } from '@metaplex-foundation/umi';
import { revokeCollectionPluginAuthorityV1, PluginType } from '../../generated';

export type RevokeCollectionPluginAuthorityArgs = Omit<
  Parameters<typeof revokeCollectionPluginAuthorityV1>[1],
  'pluginType'
> & {
  plugin: {
    type: keyof typeof PluginType;
  };
};

export const revokeCollectionPluginAuthority = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, ...args }: RevokeCollectionPluginAuthorityArgs
) =>
  revokeCollectionPluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
  });
