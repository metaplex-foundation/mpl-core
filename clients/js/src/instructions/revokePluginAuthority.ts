import { Context } from '@metaplex-foundation/umi';
import { revokePluginAuthorityV1, PluginType } from '../generated';
import { isExternalPluginType } from '../plugins';
import { ExternalPluginKey } from '../plugins/externalPluginKey';

export type RevokePluginAuthorityArgs = Omit<
  Parameters<typeof revokePluginAuthorityV1>[1],
  'pluginType'
> & {
  plugin:
    | {
        type: keyof typeof PluginType;
      }
    | ExternalPluginKey;
};

export const revokePluginAuthority = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, ...args }: RevokePluginAuthorityArgs
) => {
  if (isExternalPluginType(plugin)) {
    // TODO implement this
    // return revokeExternalPluginAuthorityV1(context, {
    //   ...args,
    //   key: externalPluginKeyToBase(plugin as ExternalPluginKey),
    // });
  }

  return revokePluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
  });
};
