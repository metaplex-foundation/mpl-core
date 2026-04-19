import { Context } from '@metaplex-foundation/umi';
import { removeCollectionsFromGroupV1 } from '../../generated';

export type RemoveCollectionsFromGroupArgs = Parameters<
  typeof removeCollectionsFromGroupV1
>[1];

export const removeCollectionsFromGroup = (
  context: Pick<Context, 'payer' | 'programs'>,
  args: RemoveCollectionsFromGroupArgs
) => removeCollectionsFromGroupV1(context, args);
