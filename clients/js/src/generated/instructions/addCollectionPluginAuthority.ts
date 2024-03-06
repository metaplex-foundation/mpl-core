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
import {
  Authority,
  AuthorityArgs,
  PluginType,
  PluginTypeArgs,
  getAuthoritySerializer,
  getPluginTypeSerializer,
} from '../types';

// Accounts.
export type AddCollectionPluginAuthorityInstructionAccounts = {
  /** The address of the asset */
  collection: PublicKey | Pda;
  /** The owner or delegate of the asset */
  authority?: Signer;
  /** The account paying for the storage fees */
  payer?: Signer;
  /** The system program */
  systemProgram?: PublicKey | Pda;
  /** The SPL Noop Program */
  logWrapper?: PublicKey | Pda;
};

// Data.
export type AddCollectionPluginAuthorityInstructionData = {
  discriminator: number;
  pluginType: PluginType;
  newAuthority: Authority;
};

export type AddCollectionPluginAuthorityInstructionDataArgs = {
  pluginType: PluginTypeArgs;
  newAuthority: AuthorityArgs;
};

export function getAddCollectionPluginAuthorityInstructionDataSerializer(): Serializer<
  AddCollectionPluginAuthorityInstructionDataArgs,
  AddCollectionPluginAuthorityInstructionData
> {
  return mapSerializer<
    AddCollectionPluginAuthorityInstructionDataArgs,
    any,
    AddCollectionPluginAuthorityInstructionData
  >(
    struct<AddCollectionPluginAuthorityInstructionData>(
      [
        ['discriminator', u8()],
        ['pluginType', getPluginTypeSerializer()],
        ['newAuthority', getAuthoritySerializer()],
      ],
      { description: 'AddCollectionPluginAuthorityInstructionData' }
    ),
    (value) => ({ ...value, discriminator: 9 })
  ) as Serializer<
    AddCollectionPluginAuthorityInstructionDataArgs,
    AddCollectionPluginAuthorityInstructionData
  >;
}

// Args.
export type AddCollectionPluginAuthorityInstructionArgs =
  AddCollectionPluginAuthorityInstructionDataArgs;

// Instruction.
export function addCollectionPluginAuthority(
  context: Pick<Context, 'identity' | 'programs'>,
  input: AddCollectionPluginAuthorityInstructionAccounts &
    AddCollectionPluginAuthorityInstructionArgs
): TransactionBuilder {
  // Program ID.
  const programId = context.programs.getPublicKey(
    'mplCore',
    'CoREzp6dAdLVRKf3EM5tWrsXM2jQwRFeu5uhzsAyjYXL'
  );

  // Accounts.
  const resolvedAccounts = {
    collection: {
      index: 0,
      isWritable: true as boolean,
      value: input.collection ?? null,
    },
    authority: {
      index: 1,
      isWritable: false as boolean,
      value: input.authority ?? null,
    },
    payer: {
      index: 2,
      isWritable: true as boolean,
      value: input.payer ?? null,
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
  const resolvedArgs: AddCollectionPluginAuthorityInstructionArgs = {
    ...input,
  };

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
  const data =
    getAddCollectionPluginAuthorityInstructionDataSerializer().serialize(
      resolvedArgs as AddCollectionPluginAuthorityInstructionDataArgs
    );

  // Bytes Created On Chain.
  const bytesCreatedOnChain = 0;

  return transactionBuilder([
    { instruction: { keys, programId, data }, signers, bytesCreatedOnChain },
  ]);
}
