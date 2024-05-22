import { Context } from '@metaplex-foundation/umi';
import {
  writeExternalPluginAdapterDataV1,
  WriteExternalPluginAdapterDataV1InstructionArgs,
  WriteExternalPluginAdapterDataV1InstructionAccounts,
} from '../generated';
import {
  ExternalPluginAdapterKey,
  externalPluginAdapterKeyToBase,
} from '../plugins';

export type WriteDataArgs = Omit<
  WriteExternalPluginAdapterDataV1InstructionArgs,
  'key'
> & {
  key: ExternalPluginAdapterKey;
};

export const writeData = (
  context: Pick<Context, 'payer' | 'programs'>,
  args: WriteDataArgs & WriteExternalPluginAdapterDataV1InstructionAccounts
) => {
  const { key, ...rest } = args;
  return writeExternalPluginAdapterDataV1(context, {
    ...rest,
    key: externalPluginAdapterKeyToBase(key),
  });
};
