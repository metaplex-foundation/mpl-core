import {
  AccountMeta,
  Context,
  Signer,
  TransactionBuilder,
} from '@metaplex-foundation/umi';
import { batchV1 } from '../generated';

export const batch = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  builder: TransactionBuilder
) => {
  const ixes = builder.items;
  let numAccounts = Buffer.alloc(0);
  let instructions = Buffer.alloc(0);
  const remainingAccounts: AccountMeta[] = [];
  const signers: Signer[] = [];

  ixes.forEach((ix) => {
    numAccounts = Buffer.concat([
      numAccounts,
      Buffer.from([ix.instruction.keys.length]),
    ]);
    instructions = Buffer.concat([instructions, ix.instruction.data]);
    remainingAccounts.push(...ix.instruction.keys);
    signers.push(...ix.signers);
  });

  const batchBuilder = batchV1(context, {
    numAccounts,
    instructions,
  }).addRemainingAccounts(remainingAccounts);

  const batchBuilderItems = batchBuilder.items;
  batchBuilderItems[0].signers.push(...signers);
  batchBuilder.setItems(batchBuilderItems);
  return batchBuilder;
};
