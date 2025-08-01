import { Context } from '@metaplex-foundation/umi';
import { updateGroupV1 } from '../../generated';

export type UpdateGroupArgs = Parameters<typeof updateGroupV1>[1];

export const updateGroup = (
  context: Pick<Context, 'payer' | 'programs'>,
  args: UpdateGroupArgs
) => updateGroupV1(context, args);
