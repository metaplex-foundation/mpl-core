/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import {
  Context,
  Option,
  OptionOrNullable,
  Pda,
  PublicKey,
  Signer,
  TransactionBuilder,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import {
  Serializer,
  mapSerializer,
  option,
  string,
  struct,
  u8,
} from '@metaplex-foundation/umi/serializers';
import {
  ResolvedAccount,
  ResolvedAccountsWithIndices,
  getAccountMetasAndSigners,
} from '../shared';

// Accounts.
export type UpdateCollectionInstructionAccounts = {
  /** The address of the asset */
  collection: PublicKey | Pda;
  /** The update authority or update authority delegate of the asset */
  authority?: Signer;
  /** The account paying for the storage fees */
  payer?: Signer;
  /** The new update authority of the asset */
  newUpdateAuthority?: PublicKey | Pda;
  /** The system program */
  systemProgram?: PublicKey | Pda;
  /** The SPL Noop Program */
  logWrapper?: PublicKey | Pda;
};

// Data.
export type UpdateCollectionInstructionData = {
  discriminator: number;
  newName: Option<string>;
  newUri: Option<string>;
};

export type UpdateCollectionInstructionDataArgs = {
  newName: OptionOrNullable<string>;
  newUri: OptionOrNullable<string>;
};

export function getUpdateCollectionInstructionDataSerializer(): Serializer<
  UpdateCollectionInstructionDataArgs,
  UpdateCollectionInstructionData
> {
  return mapSerializer<
    UpdateCollectionInstructionDataArgs,
    any,
    UpdateCollectionInstructionData
  >(
    struct<UpdateCollectionInstructionData>(
      [
        ['discriminator', u8()],
        ['newName', option(string())],
        ['newUri', option(string())],
      ],
      { description: 'UpdateCollectionInstructionData' }
    ),
    (value) => ({ ...value, discriminator: 16 })
  ) as Serializer<
    UpdateCollectionInstructionDataArgs,
    UpdateCollectionInstructionData
  >;
}

// Args.
export type UpdateCollectionInstructionArgs =
  UpdateCollectionInstructionDataArgs;

// Instruction.
export function updateCollection(
  context: Pick<Context, 'identity' | 'programs'>,
  input: UpdateCollectionInstructionAccounts & UpdateCollectionInstructionArgs
): TransactionBuilder {
  // Program ID.
  const programId = context.programs.getPublicKey(
    'mplCore',
    'CoREzp6dAdLVRKf3EM5tWrsXM2jQwRFeu5uhzsAyjYXL'
  );

  // Accounts.
  const resolvedAccounts: ResolvedAccountsWithIndices = {
    collection: { index: 0, isWritable: true, value: input.collection ?? null },
    authority: { index: 1, isWritable: false, value: input.authority ?? null },
    payer: { index: 2, isWritable: true, value: input.payer ?? null },
    newUpdateAuthority: {
      index: 3,
      isWritable: false,
      value: input.newUpdateAuthority ?? null,
    },
    systemProgram: {
      index: 4,
      isWritable: false,
      value: input.systemProgram ?? null,
    },
    logWrapper: {
      index: 5,
      isWritable: false,
      value: input.logWrapper ?? null,
    },
  };

  // Arguments.
  const resolvedArgs: UpdateCollectionInstructionArgs = { ...input };

  // Default values.
  if (!resolvedAccounts.authority.value) {
    resolvedAccounts.authority.value = context.identity;
  }
  if (!resolvedAccounts.systemProgram.value) {
    resolvedAccounts.systemProgram.value = context.programs.getPublicKey(
      'splSystem',
      '11111111111111111111111111111111'
    );
    resolvedAccounts.systemProgram.isWritable = false;
  }

  // Accounts in order.
  const orderedAccounts: ResolvedAccount[] = Object.values(
    resolvedAccounts
  ).sort((a, b) => a.index - b.index);

  // Keys and Signers.
  const [keys, signers] = getAccountMetasAndSigners(
    orderedAccounts,
    'programId',
    programId
  );

  // Data.
  const data = getUpdateCollectionInstructionDataSerializer().serialize(
    resolvedArgs as UpdateCollectionInstructionDataArgs
  );

  // Bytes Created On Chain.
  const bytesCreatedOnChain = 0;

  return transactionBuilder([
    { instruction: { keys, programId, data }, signers, bytesCreatedOnChain },
  ]);
}
