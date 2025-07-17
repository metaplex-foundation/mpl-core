import { Context } from '@metaplex-foundation/umi';
import { updateGroupPluginV1 } from '../../generated';
import { createGroupPluginV2, GroupAllPluginArgsV2 } from '../../plugins';

export type UpdateGroupPluginArgsPlugin = GroupAllPluginArgsV2;

export type UpdateGroupPluginArgs = Omit<
  Parameters<typeof updateGroupPluginV1>[1],
  'plugin'
> & {
  plugin: UpdateGroupPluginArgsPlugin;
};

export const updateGroupPlugin = (
  context: Pick<Context, 'payer' | 'programs'>,
  { plugin, ...args }: UpdateGroupPluginArgs
) =>
  updateGroupPluginV1(context, {
    ...args,
    plugin: createGroupPluginV2(plugin as GroupAllPluginArgsV2),
  });
