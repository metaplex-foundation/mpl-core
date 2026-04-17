import { Context } from '@metaplex-foundation/umi';
import { removeGroupsFromGroupV1 } from '../../generated';

export type RemoveGroupsFromGroupArgs = Parameters<
  typeof removeGroupsFromGroupV1
>[1];

export const removeGroupsFromGroup = (
  context: Pick<Context, 'payer' | 'programs'>,
  args: RemoveGroupsFromGroupArgs
) => removeGroupsFromGroupV1(context, args);
