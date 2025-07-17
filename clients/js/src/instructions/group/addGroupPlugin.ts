import { Context } from '@metaplex-foundation/umi';
import {
  addGroupExternalPluginAdapterV1,
  addGroupPluginV1,
} from '../../generated';
import {
  GroupAddablePluginAuthorityPairArgsV2,
  groupPluginAuthorityPairV2,
} from '../../plugins';
import {
  createExternalPluginAdapterInitInfo,
  ExternalPluginAdapterInitInfoArgs,
  isExternalPluginAdapterType,
} from '../../plugins/externalPluginAdapters';

export type AddGroupPluginArgsPlugin =
  | GroupAddablePluginAuthorityPairArgsV2
  | ExternalPluginAdapterInitInfoArgs;

export type AddGroupPluginArgs = Omit<
  Parameters<typeof addGroupPluginV1>[1],
  'plugin' | 'initAuthority'
> & {
  plugin: AddGroupPluginArgsPlugin;
};

export const addGroupPlugin = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, ...args }: AddGroupPluginArgs
) => {
  if (isExternalPluginAdapterType(plugin)) {
    return addGroupExternalPluginAdapterV1(context, {
      ...args,
      initInfo: createExternalPluginAdapterInitInfo(
        plugin as ExternalPluginAdapterInitInfoArgs
      ),
    });
  }

  const pair = groupPluginAuthorityPairV2(
    plugin as GroupAddablePluginAuthorityPairArgsV2
  );
  return addGroupPluginV1(context, {
    ...args,
    plugin: pair.plugin,
    initAuthority: pair.authority,
  });
};
