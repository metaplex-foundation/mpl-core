import { Context } from '@metaplex-foundation/umi';
import { addAssetsToGroupV1 } from '../../generated';

export type AddAssetsToGroupArgs = Parameters<typeof addAssetsToGroupV1>[1];

export const addAssetsToGroup = (
  context: Pick<Context, 'payer' | 'programs'>,
  args: AddAssetsToGroupArgs
) => addAssetsToGroupV1(context, args);
