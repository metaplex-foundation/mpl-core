import { Context } from '@metaplex-foundation/umi';
import { createGroupV1 } from '../../generated';

export type CreateGroupArgs = Parameters<typeof createGroupV1>[1];

export const createGroup = (
  context: Pick<Context, 'payer' | 'programs'>,
  args: CreateGroupArgs
) => createGroupV1(context, args);
