import { Context } from '@metaplex-foundation/umi';
import { addCollectionsToGroupV1 } from '../../generated';

export type AddCollectionsToGroupArgs = Parameters<
  typeof addCollectionsToGroupV1
>[1];

export const addCollectionsToGroup = (
  context: Pick<Context, 'payer' | 'programs'>,
  args: AddCollectionsToGroupArgs
) => addCollectionsToGroupV1(context, args);
