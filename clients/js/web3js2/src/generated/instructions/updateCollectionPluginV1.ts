/**
 * This code was AUTOGENERATED using the codama library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun codama to update it.
 *
 * @see https://github.com/codama-idl/codama
 */

import {
  combineCodec,
  getStructDecoder,
  getStructEncoder,
  getU8Decoder,
  getU8Encoder,
  transformEncoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
  type IAccountMeta,
  type IAccountSignerMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type ReadonlySignerAccount,
  type TransactionSigner,
  type WritableAccount,
  type WritableSignerAccount,
} from '@solana/kit';
import { MPL_CORE_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';
import {
  getPluginDecoder,
  getPluginEncoder,
  type Plugin,
  type PluginArgs,
} from '../types';

export const UPDATE_COLLECTION_PLUGIN_V1_DISCRIMINATOR = 7;

export function getUpdateCollectionPluginV1DiscriminatorBytes() {
  return getU8Encoder().encode(UPDATE_COLLECTION_PLUGIN_V1_DISCRIMINATOR);
}

export type UpdateCollectionPluginV1Instruction<
  TProgram extends string = typeof MPL_CORE_PROGRAM_ADDRESS,
  TAccountCollection extends string | IAccountMeta<string> = string,
  TAccountPayer extends string | IAccountMeta<string> = string,
  TAccountAuthority extends string | IAccountMeta<string> = string,
  TAccountSystemProgram extends
    | string
    | IAccountMeta<string> = '11111111111111111111111111111111',
  TAccountLogWrapper extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountCollection extends string
        ? WritableAccount<TAccountCollection>
        : TAccountCollection,
      TAccountPayer extends string
        ? WritableSignerAccount<TAccountPayer> &
            IAccountSignerMeta<TAccountPayer>
        : TAccountPayer,
      TAccountAuthority extends string
        ? ReadonlySignerAccount<TAccountAuthority> &
            IAccountSignerMeta<TAccountAuthority>
        : TAccountAuthority,
      TAccountSystemProgram extends string
        ? ReadonlyAccount<TAccountSystemProgram>
        : TAccountSystemProgram,
      TAccountLogWrapper extends string
        ? ReadonlyAccount<TAccountLogWrapper>
        : TAccountLogWrapper,
      ...TRemainingAccounts,
    ]
  >;

export type UpdateCollectionPluginV1InstructionData = {
  discriminator: number;
  plugin: Plugin;
};

export type UpdateCollectionPluginV1InstructionDataArgs = {
  plugin: PluginArgs;
};

export function getUpdateCollectionPluginV1InstructionDataEncoder(): Encoder<UpdateCollectionPluginV1InstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['plugin', getPluginEncoder()],
    ]),
    (value) => ({
      ...value,
      discriminator: UPDATE_COLLECTION_PLUGIN_V1_DISCRIMINATOR,
    })
  );
}

export function getUpdateCollectionPluginV1InstructionDataDecoder(): Decoder<UpdateCollectionPluginV1InstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['plugin', getPluginDecoder()],
  ]);
}

export function getUpdateCollectionPluginV1InstructionDataCodec(): Codec<
  UpdateCollectionPluginV1InstructionDataArgs,
  UpdateCollectionPluginV1InstructionData
> {
  return combineCodec(
    getUpdateCollectionPluginV1InstructionDataEncoder(),
    getUpdateCollectionPluginV1InstructionDataDecoder()
  );
}

export type UpdateCollectionPluginV1Input<
  TAccountCollection extends string = string,
  TAccountPayer extends string = string,
  TAccountAuthority extends string = string,
  TAccountSystemProgram extends string = string,
  TAccountLogWrapper extends string = string,
> = {
  /** The address of the asset */
  collection: Address<TAccountCollection>;
  /** The account paying for the storage fees */
  payer: TransactionSigner<TAccountPayer>;
  /** The owner or delegate of the asset */
  authority?: TransactionSigner<TAccountAuthority>;
  /** The system program */
  systemProgram?: Address<TAccountSystemProgram>;
  /** The SPL Noop Program */
  logWrapper?: Address<TAccountLogWrapper>;
  plugin: UpdateCollectionPluginV1InstructionDataArgs['plugin'];
};

export function getUpdateCollectionPluginV1Instruction<
  TAccountCollection extends string,
  TAccountPayer extends string,
  TAccountAuthority extends string,
  TAccountSystemProgram extends string,
  TAccountLogWrapper extends string,
  TProgramAddress extends Address = typeof MPL_CORE_PROGRAM_ADDRESS,
>(
  input: UpdateCollectionPluginV1Input<
    TAccountCollection,
    TAccountPayer,
    TAccountAuthority,
    TAccountSystemProgram,
    TAccountLogWrapper
  >,
  config?: { programAddress?: TProgramAddress }
): UpdateCollectionPluginV1Instruction<
  TProgramAddress,
  TAccountCollection,
  TAccountPayer,
  TAccountAuthority,
  TAccountSystemProgram,
  TAccountLogWrapper
> {
  // Program address.
  const programAddress = config?.programAddress ?? MPL_CORE_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    collection: { value: input.collection ?? null, isWritable: true },
    payer: { value: input.payer ?? null, isWritable: true },
    authority: { value: input.authority ?? null, isWritable: false },
    systemProgram: { value: input.systemProgram ?? null, isWritable: false },
    logWrapper: { value: input.logWrapper ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  // Resolve default values.
  if (!accounts.systemProgram.value) {
    accounts.systemProgram.value =
      '11111111111111111111111111111111' as Address<'11111111111111111111111111111111'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.collection),
      getAccountMeta(accounts.payer),
      getAccountMeta(accounts.authority),
      getAccountMeta(accounts.systemProgram),
      getAccountMeta(accounts.logWrapper),
    ],
    programAddress,
    data: getUpdateCollectionPluginV1InstructionDataEncoder().encode(
      args as UpdateCollectionPluginV1InstructionDataArgs
    ),
  } as UpdateCollectionPluginV1Instruction<
    TProgramAddress,
    TAccountCollection,
    TAccountPayer,
    TAccountAuthority,
    TAccountSystemProgram,
    TAccountLogWrapper
  >;

  return instruction;
}

export type ParsedUpdateCollectionPluginV1Instruction<
  TProgram extends string = typeof MPL_CORE_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** The address of the asset */
    collection: TAccountMetas[0];
    /** The account paying for the storage fees */
    payer: TAccountMetas[1];
    /** The owner or delegate of the asset */
    authority?: TAccountMetas[2] | undefined;
    /** The system program */
    systemProgram: TAccountMetas[3];
    /** The SPL Noop Program */
    logWrapper?: TAccountMetas[4] | undefined;
  };
  data: UpdateCollectionPluginV1InstructionData;
};

export function parseUpdateCollectionPluginV1Instruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedUpdateCollectionPluginV1Instruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 5) {
    // TODO: Coded error.
    throw new Error('Not enough accounts');
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts![accountIndex]!;
    accountIndex += 1;
    return accountMeta;
  };
  const getNextOptionalAccount = () => {
    const accountMeta = getNextAccount();
    return accountMeta.address === MPL_CORE_PROGRAM_ADDRESS
      ? undefined
      : accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      collection: getNextAccount(),
      payer: getNextAccount(),
      authority: getNextOptionalAccount(),
      systemProgram: getNextAccount(),
      logWrapper: getNextOptionalAccount(),
    },
    data: getUpdateCollectionPluginV1InstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
