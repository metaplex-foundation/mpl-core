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

export const COMPRESS_V1_DISCRIMINATOR = 17;

export function getCompressV1DiscriminatorBytes() {
  return getU8Encoder().encode(COMPRESS_V1_DISCRIMINATOR);
}

export type CompressV1Instruction<
  TProgram extends string = typeof MPL_CORE_PROGRAM_ADDRESS,
  TAccountAsset extends string | IAccountMeta<string> = string,
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
      TAccountAsset extends string
        ? WritableAccount<TAccountAsset>
        : TAccountAsset,
      TAccountCollection extends string
        ? ReadonlyAccount<TAccountCollection>
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

export type CompressV1InstructionData = { discriminator: number };

export type CompressV1InstructionDataArgs = {};

export function getCompressV1InstructionDataEncoder(): Encoder<CompressV1InstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: COMPRESS_V1_DISCRIMINATOR })
  );
}

export function getCompressV1InstructionDataDecoder(): Decoder<CompressV1InstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getCompressV1InstructionDataCodec(): Codec<
  CompressV1InstructionDataArgs,
  CompressV1InstructionData
> {
  return combineCodec(
    getCompressV1InstructionDataEncoder(),
    getCompressV1InstructionDataDecoder()
  );
}

export type CompressV1Input<
  TAccountAsset extends string = string,
  TAccountCollection extends string = string,
  TAccountPayer extends string = string,
  TAccountAuthority extends string = string,
  TAccountSystemProgram extends string = string,
  TAccountLogWrapper extends string = string,
> = {
  /** The address of the asset */
  asset: Address<TAccountAsset>;
  /** The collection to which the asset belongs */
  collection?: Address<TAccountCollection>;
  /** The account receiving the storage fees */
  payer: TransactionSigner<TAccountPayer>;
  /** The owner or delegate of the asset */
  authority?: TransactionSigner<TAccountAuthority>;
  /** The system program */
  systemProgram?: Address<TAccountSystemProgram>;
  /** The SPL Noop Program */
  logWrapper?: Address<TAccountLogWrapper>;
};

export function getCompressV1Instruction<
  TAccountAsset extends string,
  TAccountCollection extends string,
  TAccountPayer extends string,
  TAccountAuthority extends string,
  TAccountSystemProgram extends string,
  TAccountLogWrapper extends string,
  TProgramAddress extends Address = typeof MPL_CORE_PROGRAM_ADDRESS,
>(
  input: CompressV1Input<
    TAccountAsset,
    TAccountCollection,
    TAccountPayer,
    TAccountAuthority,
    TAccountSystemProgram,
    TAccountLogWrapper
  >,
  config?: { programAddress?: TProgramAddress }
): CompressV1Instruction<
  TProgramAddress,
  TAccountAsset,
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
    asset: { value: input.asset ?? null, isWritable: true },
    collection: { value: input.collection ?? null, isWritable: false },
    payer: { value: input.payer ?? null, isWritable: true },
    authority: { value: input.authority ?? null, isWritable: false },
    systemProgram: { value: input.systemProgram ?? null, isWritable: false },
    logWrapper: { value: input.logWrapper ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Resolve default values.
  if (!accounts.systemProgram.value) {
    accounts.systemProgram.value =
      '11111111111111111111111111111111' as Address<'11111111111111111111111111111111'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.asset),
      getAccountMeta(accounts.collection),
      getAccountMeta(accounts.payer),
      getAccountMeta(accounts.authority),
      getAccountMeta(accounts.systemProgram),
      getAccountMeta(accounts.logWrapper),
    ],
    programAddress,
    data: getCompressV1InstructionDataEncoder().encode({}),
  } as CompressV1Instruction<
    TProgramAddress,
    TAccountAsset,
    TAccountCollection,
    TAccountPayer,
    TAccountAuthority,
    TAccountSystemProgram,
    TAccountLogWrapper
  >;

  return instruction;
}

export type ParsedCompressV1Instruction<
  TProgram extends string = typeof MPL_CORE_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** The address of the asset */
    asset: TAccountMetas[0];
    /** The collection to which the asset belongs */
    collection?: TAccountMetas[1] | undefined;
    /** The account receiving the storage fees */
    payer: TAccountMetas[2];
    /** The owner or delegate of the asset */
    authority?: TAccountMetas[3] | undefined;
    /** The system program */
    systemProgram: TAccountMetas[4];
    /** The SPL Noop Program */
    logWrapper?: TAccountMetas[5] | undefined;
  };
  data: CompressV1InstructionData;
};

export function parseCompressV1Instruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedCompressV1Instruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 6) {
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
      asset: getNextAccount(),
      collection: getNextOptionalAccount(),
      payer: getNextAccount(),
      authority: getNextOptionalAccount(),
      systemProgram: getNextAccount(),
      logWrapper: getNextOptionalAccount(),
    },
    data: getCompressV1InstructionDataDecoder().decode(instruction.data),
  };
}
