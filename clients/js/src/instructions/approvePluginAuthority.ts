import { Context } from '@metaplex-foundation/umi';
import { approvePluginAuthorityV1, PluginType } from '../generated';
import {
  isExternalPluginType,
  PluginAuthority,
  pluginAuthorityToBase,
} from '../plugins';
import { ExternalPluginKey } from '../plugins/externalPluginKey';

export type ApprovePluginAuthorityArgs = Omit<
  Parameters<typeof approvePluginAuthorityV1>[1],
  'pluginType' | 'newAuthority'
> & {
  plugin:
    | {
        type: keyof typeof PluginType;
      }
    | ExternalPluginKey;
  newAuthority: PluginAuthority;
};

export const approvePluginAuthority = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  { plugin, newAuthority, ...args }: ApprovePluginAuthorityArgs
) => {
  if (isExternalPluginType(plugin)) {
    // TODO implement this
    // return approveExternalPluginAuthorityV1(context, {
    //   ...args,
    //   key: externalPluginKeyToBase(plugin as ExternalPluginKey),
    // });
  }

  return approvePluginAuthorityV1(context, {
    ...args,
    pluginType: PluginType[plugin.type as keyof typeof PluginType],
    newAuthority: pluginAuthorityToBase(newAuthority),
  });
};
