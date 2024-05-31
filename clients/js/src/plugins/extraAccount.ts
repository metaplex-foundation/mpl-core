import { AccountMeta, Context, PublicKey } from '@metaplex-foundation/umi';
import {
  string,
  publicKey as publicKeySerializer,
} from '@metaplex-foundation/umi/serializers';
import { BaseExtraAccount } from '../generated';
import { Seed, seedFromBase, seedToBase } from './seed';
import { RenameToType, someOrNone, unwrapOption } from '../utils';

export const PRECONFIGURED_SEED = 'mpl-core';

export const findPreconfiguredPda = (
  context: Pick<Context, 'eddsa'>,
  program: PublicKey,
  key: PublicKey
) =>
  context.eddsa.findPda(program, [
    string({ size: 'variable' }).serialize(PRECONFIGURED_SEED),
    publicKeySerializer().serialize(key),
  ]);

export type ExtraAccount =
  | (Omit<
      Exclude<
        RenameToType<BaseExtraAccount>,
        { type: 'CustomPda' } | { type: 'Address' }
      >,
      'isSigner' | 'isWritable'
    > & {
      isSigner?: boolean;
      isWritable?: boolean;
    })
  | {
      type: 'CustomPda';
      seeds: Array<Seed>;
      customProgramId?: PublicKey;
      isSigner?: boolean;
      isWritable?: boolean;
    }
  | {
      type: 'Address';
      address: PublicKey;
      isSigner?: boolean;
      isWritable?: boolean;
    };

export function extraAccountToAccountMeta(
  context: Pick<Context, 'eddsa'>,
  e: ExtraAccount,
  inputs: {
    program?: PublicKey;
    asset?: PublicKey;
    collection?: PublicKey;
    recipient?: PublicKey;
    owner?: PublicKey;
  }
): AccountMeta {
  const acccountMeta: Pick<AccountMeta, 'isSigner' | 'isWritable'> = {
    isSigner: e.isSigner || false,
    isWritable: e.isWritable || false,
  };

  const requiredInputs = getExtraAccountRequiredInputs(e);
  const missing: string[] = [];

  requiredInputs.forEach((input) => {
    if (!inputs[input]) {
      missing.push(input);
    }
  });

  if (missing.length) {
    throw new Error(
      `Missing required inputs to derive account address: ${missing.join(', ')}`
    );
  }
  switch (e.type) {
    case 'PreconfiguredProgram':
      return {
        ...acccountMeta,
        pubkey: context.eddsa.findPda(inputs.program!, [
          string({ size: 'variable' }).serialize(PRECONFIGURED_SEED),
        ])[0],
      };
    case 'PreconfiguredCollection':
      return {
        ...acccountMeta,
        pubkey: findPreconfiguredPda(
          context,
          inputs.program!,
          inputs.collection!
        )[0],
      };
    case 'PreconfiguredOwner':
      return {
        ...acccountMeta,
        pubkey: findPreconfiguredPda(
          context,
          inputs.program!,
          inputs.owner!
        )[0],
      };
    case 'PreconfiguredRecipient':
      return {
        ...acccountMeta,
        pubkey: findPreconfiguredPda(
          context,
          inputs.program!,
          inputs.recipient!
        )[0],
      };
    case 'PreconfiguredAsset':
      return {
        ...acccountMeta,
        pubkey: findPreconfiguredPda(
          context,
          inputs.program!,
          inputs.asset!
        )[0],
      };
    case 'CustomPda':
      return {
        pubkey: context.eddsa.findPda(
          e.customProgramId ? e.customProgramId : inputs.program!,
          e.seeds.map((seed) => {
            switch (seed.type) {
              case 'Collection':
                return publicKeySerializer().serialize(inputs.collection!);
              case 'Owner':
                return publicKeySerializer().serialize(inputs.owner!);
              case 'Recipient':
                return publicKeySerializer().serialize(inputs.recipient!);
              case 'Asset':
                return publicKeySerializer().serialize(inputs.asset!);
              case 'Address':
                return publicKeySerializer().serialize(seed.pubkey);
              case 'Bytes':
                return seed.bytes;
              default:
                throw new Error('Unknown seed type');
            }
          })
        )[0],
        ...acccountMeta,
      };
    case 'Address':
      return {
        ...acccountMeta,
        pubkey: e.address,
      };
    default:
      throw new Error('Unknown extra account type');
  }
}

export function extraAccountToBase(s: ExtraAccount): BaseExtraAccount {
  const acccountMeta: Pick<BaseExtraAccount, 'isSigner' | 'isWritable'> = {
    isSigner: s.isSigner || false,
    isWritable: s.isWritable || false,
  };
  if (s.type === 'CustomPda') {
    return {
      __kind: 'CustomPda',
      ...acccountMeta,
      seeds: s.seeds.map(seedToBase),
      customProgramId: someOrNone(s.customProgramId),
    };
  }
  if (s.type === 'Address') {
    return {
      __kind: 'Address',
      ...acccountMeta,
      address: s.address,
    };
  }

  return {
    __kind: s.type,
    ...acccountMeta,
  };
}

export function extraAccountFromBase(s: BaseExtraAccount): ExtraAccount {
  if (s.__kind === 'CustomPda') {
    return {
      type: 'CustomPda',
      isSigner: s.isSigner,
      isWritable: s.isWritable,
      seeds: s.seeds.map(seedFromBase),
      customProgramId: unwrapOption(s.customProgramId),
    };
  }
  if (s.__kind === 'Address') {
    return {
      type: 'Address',
      isSigner: s.isSigner,
      isWritable: s.isWritable,
      address: s.address,
    };
  }

  return {
    type: s.__kind,
    isSigner: s.isSigner,
    isWritable: s.isWritable,
  };
}

export type ExtraAccountInput =
  | 'owner'
  | 'recipient'
  | 'asset'
  | 'collection'
  | 'program';

const EXTRA_ACCOUNT_INPUT_MAP: {
  [type in ExtraAccount['type']]?: ExtraAccountInput;
} = {
  PreconfiguredOwner: 'owner',
  PreconfiguredRecipient: 'recipient',
  PreconfiguredAsset: 'asset',
  PreconfiguredCollection: 'collection',
  PreconfiguredProgram: 'program',
};

export function getExtraAccountRequiredInputs(
  s: ExtraAccount
): ExtraAccountInput[] {
  const preconfigured = EXTRA_ACCOUNT_INPUT_MAP[s.type];
  if (preconfigured) {
    return [preconfigured];
  }

  if (s.type === 'CustomPda') {
    return s.seeds
      .map((seed) => {
        switch (seed.type) {
          case 'Collection':
            return 'collection';
          case 'Owner':
            return 'owner';
          case 'Recipient':
            return 'recipient';
          case 'Asset':
            return 'asset';
          default:
            return null;
        }
      })
      .filter((input) => input) as ExtraAccountInput[];
  }

  return [];
}
