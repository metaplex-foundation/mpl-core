import { BaseTreasury } from '../generated';

export type Treasury = {
  withdrawn: number;
};

export type TreasuryArgs = Treasury;

export function treasuryToBase(s: Treasury): BaseTreasury {
  return {
    withdrawn: BigInt(s.withdrawn),
  };
}

export function treasuryFromBase(s: BaseTreasury): Treasury {
  return {
    withdrawn: Number(s.withdrawn),
  };
}
