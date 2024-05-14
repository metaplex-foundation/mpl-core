import { Context } from '@metaplex-foundation/umi';
import { writeExternalPluginAdapterDataV1, WriteExternalPluginAdapterDataV1InstructionArgs, WriteExternalPluginAdapterDataV1InstructionAccounts } from '../generated';

export type WriteDataArgs = WriteExternalPluginAdapterDataV1InstructionArgs;

export const writeData = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>,
  args: WriteDataArgs & WriteExternalPluginAdapterDataV1InstructionAccounts,
) => writeExternalPluginAdapterDataV1(context, {
  ...args,
});
