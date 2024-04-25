import { Context } from '@metaplex-foundation/umi';
import {
  approveCollectionPluginAuthorityV1,
  PluginType,
} from '../../generated';
import {
  isExternalPluginType,
  PluginAuthority,
  pluginAuthorityToBase,
} from '../../plugins';
import { ExternalPluginKey } from '../../plugins/externalPluginKey';

export type ApproveCollectionPluginAuthorityArgs = Omit<
  Parameters<typeof approveCollectionPluginAuthorityV1>[1],
  'pluginType' | 'newAuthority'
> & {
  plugin:
    | {
        type: keyof typeof PluginType;
      }
    | ExternalPluginKey;
  newAuthority: PluginAuthority;
};

export const approveCollectionPluginAuthority = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, newAuthority, ...args }: ApproveCollectionPluginAuthorityArgs
) => {
  if (isExternalPluginType(plugin)) {
    // TODO implement this
    // return approveCollectionExternalPluginAuthorityV1(context, {
    //   ...args,
    //   key: externalPluginKeyToBase(plugin as ExternalPluginKey),
    // });
  }

  return approveCollectionPluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
    newAuthority: pluginAuthorityToBase(newAuthority),
  });
};
