import { lamports, SolAmount } from '@metaplex-foundation/umi';
import { BaseTreasury } from '../generated';

export type Treasury = {
  withdrawn: SolAmount;
};

export type TreasuryArgs = Treasury;

export function treasuryToBase(s: Treasury): BaseTreasury {
  return {
    withdrawn: s.withdrawn.basisPoints,
  };
}

export function treasuryFromBase(s: BaseTreasury): Treasury {
  return {
    withdrawn: lamports(s.withdrawn),
  };
}
