import { Context } from '@metaplex-foundation/umi';
import { addGroupsToGroupV1 } from '../../generated';

export type AddGroupsToGroupArgs = Parameters<typeof addGroupsToGroupV1>[1];

export const addGroupsToGroup = (
  context: Pick<Context, 'payer' | 'programs'>,
  args: AddGroupsToGroupArgs
) => addGroupsToGroupV1(context, args);
