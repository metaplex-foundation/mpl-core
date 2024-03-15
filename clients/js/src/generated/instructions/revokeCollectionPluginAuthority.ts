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
  mapSerializer,
  struct,
  u8,
} from '@metaplex-foundation/umi/serializers';
import {
  ResolvedAccount,
  ResolvedAccountsWithIndices,
  getAccountMetasAndSigners,
} from '../shared';
import { PluginType, PluginTypeArgs, getPluginTypeSerializer } from '../types';

// Accounts.
export type RevokeCollectionPluginAuthorityInstructionAccounts = {
  /** The address of the asset */
  collection: PublicKey | Pda;
  /** The account paying for the storage fees */
  payer?: Signer;
  /** The owner or delegate of the asset */
  authority?: Signer;
  /** The system program */
  systemProgram?: PublicKey | Pda;
  /** The SPL Noop Program */
  logWrapper?: PublicKey | Pda;
};

// Data.
export type RevokeCollectionPluginAuthorityInstructionData = {
  discriminator: number;
  pluginType: PluginType;
};

export type RevokeCollectionPluginAuthorityInstructionDataArgs = {
  pluginType: PluginTypeArgs;
};

export function getRevokeCollectionPluginAuthorityInstructionDataSerializer(): Serializer<
  RevokeCollectionPluginAuthorityInstructionDataArgs,
  RevokeCollectionPluginAuthorityInstructionData
> {
  return mapSerializer<
    RevokeCollectionPluginAuthorityInstructionDataArgs,
    any,
    RevokeCollectionPluginAuthorityInstructionData
  >(
    struct<RevokeCollectionPluginAuthorityInstructionData>(
      [
        ['discriminator', u8()],
        ['pluginType', getPluginTypeSerializer()],
      ],
      { description: 'RevokeCollectionPluginAuthorityInstructionData' }
    ),
    (value) => ({ ...value, discriminator: 11 })
  ) as Serializer<
    RevokeCollectionPluginAuthorityInstructionDataArgs,
    RevokeCollectionPluginAuthorityInstructionData
  >;
}

// Args.
export type RevokeCollectionPluginAuthorityInstructionArgs =
  RevokeCollectionPluginAuthorityInstructionDataArgs;

// Instruction.
export function revokeCollectionPluginAuthority(
  context: Pick<Context, 'payer' | 'programs'>,
  input: RevokeCollectionPluginAuthorityInstructionAccounts &
    RevokeCollectionPluginAuthorityInstructionArgs
): TransactionBuilder {
  // Program ID.
  const programId = context.programs.getPublicKey(
    'mplCore',
    'CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d'
  );

  // Accounts.
  const resolvedAccounts = {
    collection: {
      index: 0,
      isWritable: true as boolean,
      value: input.collection ?? null,
    },
    payer: {
      index: 1,
      isWritable: true as boolean,
      value: input.payer ?? null,
    },
    authority: {
      index: 2,
      isWritable: false as boolean,
      value: input.authority ?? null,
    },
    systemProgram: {
      index: 3,
      isWritable: false as boolean,
      value: input.systemProgram ?? null,
    },
    logWrapper: {
      index: 4,
      isWritable: false as boolean,
      value: input.logWrapper ?? null,
    },
  } satisfies ResolvedAccountsWithIndices;

  // Arguments.
  const resolvedArgs: RevokeCollectionPluginAuthorityInstructionArgs = {
    ...input,
  };

  // Default values.
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
  const data =
    getRevokeCollectionPluginAuthorityInstructionDataSerializer().serialize(
      resolvedArgs as RevokeCollectionPluginAuthorityInstructionDataArgs
    );

  // Bytes Created On Chain.
  const bytesCreatedOnChain = 0;

  return transactionBuilder([
    { instruction: { keys, programId, data }, signers, bytesCreatedOnChain },
  ]);
}
