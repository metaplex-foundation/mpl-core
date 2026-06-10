import { type Address } from '@solana/addresses';
import { type BaseExtraAccount } from '../generated';
import { type Seed, seedFromBase, seedToBase } from './seed';
import { type RenameToType, someOrNone, unwrapOption } from '../utils';

export const PRECONFIGURED_SEED = 'mpl-core';

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
      customProgramId?: Address;
      isSigner?: boolean;
      isWritable?: boolean;
    }
  | {
      type: 'Address';
      address: Address;
      isSigner?: boolean;
      isWritable?: boolean;
    };

export function extraAccountToBase(s: ExtraAccount): BaseExtraAccount {
  const accountMeta: Pick<BaseExtraAccount, 'isSigner' | 'isWritable'> = {
    isSigner: s.isSigner || false,
    isWritable: s.isWritable || false,
  };
  if (s.type === 'CustomPda') {
    return {
      __kind: 'CustomPda',
      ...accountMeta,
      seeds: s.seeds.map(seedToBase),
      customProgramId: someOrNone(s.customProgramId),
    };
  }
  if (s.type === 'Address') {
    return {
      __kind: 'Address',
      ...accountMeta,
      address: s.address,
    };
  }

  return {
    __kind: s.type,
    ...accountMeta,
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
