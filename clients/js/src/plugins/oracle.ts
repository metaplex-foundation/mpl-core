import { Context, PublicKey } from '@metaplex-foundation/umi';
import { getOracleValidationSerializer, OracleValidation } from '../generated';
import { extraAccountToAccountMeta } from './extraAccount';
import { ValidationResultsOffset } from './validationResultsOffset';
import { Oracle } from '../generated/plugins/oracle';

export * from '../generated/plugins/oracle';

export function findOracleAccount(
  context: Pick<Context, 'eddsa'>,
  oracle: Pick<Oracle, 'baseAddress' | 'baseAddressConfig'>,
  inputs: {
    asset?: PublicKey;
    collection?: PublicKey;
    recipient?: PublicKey;
    owner?: PublicKey;
  }
): PublicKey {
  if (!oracle.baseAddressConfig) {
    return oracle.baseAddress;
  }

  return extraAccountToAccountMeta(context, oracle.baseAddressConfig, {
    ...inputs,
    program: oracle.baseAddress,
  }).pubkey;
}

export function deserializeOracleValidation(
  data: Uint8Array,
  offset: ValidationResultsOffset
): OracleValidation {
  let offs = 0;
  if (offset.type === 'Custom') {
    offs = Number(offset.offset);
  } else if (offset.type === 'Anchor') {
    offs = 8;
  }

  return getOracleValidationSerializer().deserialize(data, offs)[0];
}
