import { lamports, SolAmount } from '@metaplex-foundation/umi';
import { BaseSolTransferFee } from '../generated';

export type SolTransferFee = {
  feeAmount: SolAmount;
};

export type SolTransferFeeArgs = SolTransferFee;

export function solTransferFeeToBase(s: SolTransferFee): BaseSolTransferFee {
  return {
    feeAmount: s.feeAmount.basisPoints,
  };
}

export function solTransferFeeFromBase(s: BaseSolTransferFee): SolTransferFee {
  return {
    feeAmount: lamports(s.feeAmount),
  };
}
