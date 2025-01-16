/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import {
  Context,
  Pda,
  PublicKey,
  Signer,
  TransactionBuilder,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import {
  Serializer,
  bytes,
  mapSerializer,
  struct,
  u32,
  u8,
} from '@metaplex-foundation/umi/serializers';
import { findAssetSignerPda } from '../accounts';
import {
  ResolvedAccount,
  ResolvedAccountsWithIndices,
  expectPublicKey,
  getAccountMetasAndSigners,
} from '../shared';

// Accounts.
export type ExecuteV1InstructionAccounts = {
  /** The address of the asset */
  asset: PublicKey | Pda;
  /** The collection to which the asset belongs */
  collection?: PublicKey | Pda;
  /** The signing PDA for the asset */
  assetSigner?: PublicKey | Pda;
  /** The account paying for the storage fees */
  payer?: Signer;
  /** The owner or delegate of the asset */
  authority?: Signer;
  /** The system program */
  systemProgram?: PublicKey | Pda;
  /** The program id of the instruction */
  programId?: PublicKey | Pda;
};

// Data.
export type ExecuteV1InstructionData = {
  discriminator: number;
  instructionData: Uint8Array;
};

export type ExecuteV1InstructionDataArgs = { instructionData: Uint8Array };

export function getExecuteV1InstructionDataSerializer(): Serializer<
  ExecuteV1InstructionDataArgs,
  ExecuteV1InstructionData
> {
  return mapSerializer<
    ExecuteV1InstructionDataArgs,
    any,
    ExecuteV1InstructionData
  >(
    struct<ExecuteV1InstructionData>(
      [
        ['discriminator', u8()],
        ['instructionData', bytes({ size: u32() })],
      ],
      { description: 'ExecuteV1InstructionData' }
    ),
    (value) => ({ ...value, discriminator: 31 })
  ) as Serializer<ExecuteV1InstructionDataArgs, ExecuteV1InstructionData>;
}

// Args.
export type ExecuteV1InstructionArgs = ExecuteV1InstructionDataArgs;

// Instruction.
export function executeV1(
  context: Pick<Context, 'eddsa' | 'payer' | 'programs'>,
  input: ExecuteV1InstructionAccounts & ExecuteV1InstructionArgs
): TransactionBuilder {
  // Program ID.
  const programId = context.programs.getPublicKey(
    'mplCore',
    'CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d'
  );

  // Accounts.
  const resolvedAccounts = {
    asset: {
      index: 0,
      isWritable: true as boolean,
      value: input.asset ?? null,
    },
    collection: {
      index: 1,
      isWritable: true as boolean,
      value: input.collection ?? null,
    },
    assetSigner: {
      index: 2,
      isWritable: false as boolean,
      value: input.assetSigner ?? null,
    },
    payer: {
      index: 3,
      isWritable: true as boolean,
      value: input.payer ?? null,
    },
    authority: {
      index: 4,
      isWritable: false as boolean,
      value: input.authority ?? null,
    },
    systemProgram: {
      index: 5,
      isWritable: false as boolean,
      value: input.systemProgram ?? null,
    },
    programId: {
      index: 6,
      isWritable: false as boolean,
      value: input.programId ?? null,
    },
  } satisfies ResolvedAccountsWithIndices;

  // Arguments.
  const resolvedArgs: ExecuteV1InstructionArgs = { ...input };

  // Default values.
  if (!resolvedAccounts.assetSigner.value) {
    resolvedAccounts.assetSigner.value = findAssetSignerPda(context, {
      asset: expectPublicKey(resolvedAccounts.asset.value),
    });
  }
  if (!resolvedAccounts.payer.value) {
    resolvedAccounts.payer.value = context.payer;
  }
  if (!resolvedAccounts.systemProgram.value) {
    resolvedAccounts.systemProgram.value = context.programs.getPublicKey(
      'splSystem',
      '11111111111111111111111111111111'
    );
    resolvedAccounts.systemProgram.isWritable = false;
  }
  if (!resolvedAccounts.programId.value) {
    resolvedAccounts.programId.value = programId;
    resolvedAccounts.programId.isWritable = false;
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
  const data = getExecuteV1InstructionDataSerializer().serialize(
    resolvedArgs as ExecuteV1InstructionDataArgs
  );

  // Bytes Created On Chain.
  const bytesCreatedOnChain = 0;

  return transactionBuilder([
    { instruction: { keys, programId, data }, signers, bytesCreatedOnChain },
  ]);
}