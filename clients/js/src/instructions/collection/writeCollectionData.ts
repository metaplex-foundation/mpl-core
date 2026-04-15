import { Context } from '@metaplex-foundation/umi';
import {
  writeCollectionExternalPluginAdapterDataV1,
  WriteCollectionExternalPluginAdapterDataV1InstructionAccounts,
  WriteCollectionExternalPluginAdapterDataV1InstructionArgs,
} from '../../generated';
import {
  ExternalPluginAdapterKey,
  externalPluginAdapterKeyToBase,
} from '../../plugins';

export type WriteCollectionDataArgs = Omit<
  WriteCollectionExternalPluginAdapterDataV1InstructionArgs,
  'key'
> & {
  key: ExternalPluginAdapterKey;
};

export const writeCollectionData = (
  context: Pick<Context, 'payer' | 'programs'>,
  args: WriteCollectionDataArgs &
    WriteCollectionExternalPluginAdapterDataV1InstructionAccounts
) => {
  const { key, ...rest } = args;
  return writeCollectionExternalPluginAdapterDataV1(context, {
    ...rest,
    key: externalPluginAdapterKeyToBase(key),
  });
};
