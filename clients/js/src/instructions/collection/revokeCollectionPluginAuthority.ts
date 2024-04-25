import { Context } from '@metaplex-foundation/umi';
import { revokeCollectionPluginAuthorityV1, PluginType } from '../../generated';
import { isExternalPluginType } from '../../plugins';
import { ExternalPluginKey } from '../../plugins/externalPluginKey';

export type RevokeCollectionPluginAuthorityArgs = Omit<
  Parameters<typeof revokeCollectionPluginAuthorityV1>[1],
  'pluginType'
> & {
  plugin:
    | {
        type: keyof typeof PluginType;
      }
    | ExternalPluginKey;
};

export const revokeCollectionPluginAuthority = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, ...args }: RevokeCollectionPluginAuthorityArgs
) => {
  if (isExternalPluginType(plugin)) {
    // TODO implement this
    // return revokeCollectionExternalPluginAuthorityV1(context, {
    //   ...args,
    //   key: externalPluginKeyToBase(plugin as ExternalPluginKey),
    // });
  }

  return revokeCollectionPluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
  });
};
