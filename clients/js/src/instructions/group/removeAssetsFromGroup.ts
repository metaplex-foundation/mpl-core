import { Context } from '@metaplex-foundation/umi';
import { removeAssetsFromGroupV1 } from '../../generated';

export type RemoveAssetsFromGroupArgs = Parameters<
  typeof removeAssetsFromGroupV1
>[1];

export const removeAssetsFromGroup = (
  context: Pick<Context, 'payer' | 'programs'>,
  args: RemoveAssetsFromGroupArgs
) => removeAssetsFromGroupV1(context, args);
