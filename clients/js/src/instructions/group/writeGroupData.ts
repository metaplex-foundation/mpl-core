import { Context } from '@metaplex-foundation/umi';
import {
  writeGroupExternalPluginAdapterDataV1,
  WriteGroupExternalPluginAdapterDataV1InstructionAccounts,
  WriteGroupExternalPluginAdapterDataV1InstructionArgs,
} from '../../generated';
import {
  ExternalPluginAdapterKey,
  externalPluginAdapterKeyToBase,
} from '../../plugins';

export type WriteGroupDataArgs = Omit<
  WriteGroupExternalPluginAdapterDataV1InstructionArgs,
  'key'
> & {
  key: ExternalPluginAdapterKey;
};

export const writeGroupData = (
  context: Pick<Context, 'payer' | 'programs'>,
  args: WriteGroupDataArgs &
    WriteGroupExternalPluginAdapterDataV1InstructionAccounts
) => {
  const { key, ...rest } = args;
  return writeGroupExternalPluginAdapterDataV1(context, {
    ...rest,
    key: externalPluginAdapterKeyToBase(key),
  });
};
