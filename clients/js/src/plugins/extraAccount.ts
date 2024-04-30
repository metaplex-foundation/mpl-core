import { AccountMeta, Context, PublicKey } from '@metaplex-foundation/umi';
import {
  string,
  publicKey as publicKeySerializer,
} from '@metaplex-foundation/umi/serializers';
import { BaseExtraAccount } from '../generated';
import { Seed, seedFromBase, seedToBase } from './seed';
import { RenameToType } from '../utils';

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

  switch (e.type) {
    case 'PreconfiguredProgram':
      if (!inputs.program) throw new Error('Program address is required');
      return {
        ...acccountMeta,
        pubkey: context.eddsa.findPda(inputs.program, [
          string({ size: 'variable' }).serialize(PRECONFIGURED_SEED),
        ])[0],
      };
    case 'PreconfiguredCollection':
      if (!inputs.program) throw new Error('Program address is required');
      if (!inputs.collection) throw new Error('Collection address is required');
      return {
        ...acccountMeta,
        pubkey: findPreconfiguredPda(
          context,
          inputs.program,
          inputs.collection
        )[0],
      };
    case 'PreconfiguredOwner':
      if (!inputs.program) throw new Error('Program address is required');
      if (!inputs.owner) throw new Error('Owner address is required');
      return {
        ...acccountMeta,
        pubkey: findPreconfiguredPda(context, inputs.program, inputs.owner)[0],
      };
    case 'PreconfiguredRecipient':
      if (!inputs.program) throw new Error('Program address is required');
      if (!inputs.recipient) throw new Error('Recipient address is required');
      return {
        ...acccountMeta,
        pubkey: findPreconfiguredPda(
          context,
          inputs.program,
          inputs.recipient
        )[0],
      };
    case 'PreconfiguredAsset':
      if (!inputs.program) throw new Error('Program address is required');
      if (!inputs.asset) throw new Error('Asset address is required');
      return {
        ...acccountMeta,
        pubkey: findPreconfiguredPda(context, inputs.program, inputs.asset)[0],
      };
    case 'CustomPda':
      if (!inputs.program) throw new Error('Program address is required');
      return {
        pubkey: context.eddsa.findPda(
          inputs.program,
          e.seeds.map((seed) => {
            switch (seed.type) {
              case 'Collection':
                if (!inputs.collection)
                  throw new Error('Collection address is required');
                return publicKeySerializer().serialize(inputs.collection);
              case 'Owner':
                if (!inputs.owner) throw new Error('Owner address is required');
                return publicKeySerializer().serialize(inputs.owner);
              case 'Recipient':
                if (!inputs.recipient)
                  throw new Error('Recipient address is required');
                return publicKeySerializer().serialize(inputs.recipient);
              case 'Asset':
                if (!inputs.asset) throw new Error('Asset address is required');
                return publicKeySerializer().serialize(inputs.asset);
              case 'Address':
                return publicKeySerializer().serialize(seed.pubkey);
              case 'Bytes':
                return seed.bytes;
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
