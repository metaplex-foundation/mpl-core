import { Context } from '@metaplex-foundation/umi';
import {
  updateCollectionV1,
  UpdateCollectionV1InstructionDataArgs,
} from '../../generated';

export type UpdateCollectionArgs = Omit<
  Parameters<typeof updateCollectionV1>[1],
  'newName' | 'newUri'
> & {
  name?: UpdateCollectionV1InstructionDataArgs['newName'];
  uri?: UpdateCollectionV1InstructionDataArgs['newUri'];
};

export const updateCollection = (
  context: Pick<Context, 'payer' | 'programs'>,
  { name, uri, ...args }: UpdateCollectionArgs
) =>
  updateCollectionV1(context, {
    ...args,
    newName: name,
    newUri: uri,
  });
