import { Context } from '@metaplex-foundation/umi';
import { closeGroupV1 } from '../../generated';

export type CloseGroupArgs = Parameters<typeof closeGroupV1>[1];

export const closeGroup = (
  context: Pick<Context, 'payer' | 'programs'>,
  args: CloseGroupArgs
) => closeGroupV1(context, args);
