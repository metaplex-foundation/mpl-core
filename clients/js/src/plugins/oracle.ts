import { Context, PublicKey } from '@metaplex-foundation/umi';
import {
  ExtraAccount,
  extraAccountFromBase,
  extraAccountToAccountMeta,
  extraAccountToBase,
} from './extraAccount';
import {
  BaseOracle,
  BaseOracleInitInfoArgs,
  BaseOracleUpdateInfoArgs,
  ExternalRegistryRecord,
  getOracleValidationSerializer,
  OracleValidation,
} from '../generated';
import { LifecycleChecks, lifecycleChecksToBase } from './lifecycleChecks';
import { PluginAuthority, pluginAuthorityToBase } from './pluginAuthority';
import { ExternalPluginManifest } from './externalPluginManifest';
import { BaseExternalPlugin } from './externalPlugins';
import { ExternalPluginKey } from './externalPluginKey';
import {
  ValidationResultsOffset,
  validationResultsOffsetFromBase,
  validationResultsOffsetToBase,
} from './validationResultsOffset';

export type Oracle = Omit<BaseOracle, 'pda' | 'resultsOffset'> & {
  pda?: ExtraAccount;
  resultsOffset: ValidationResultsOffset;
};

export type OraclePlugin = BaseExternalPlugin &
  Oracle & {
    type: 'Oracle';
  };

export type OracleInitInfoArgs = Omit<
  BaseOracleInitInfoArgs,
  'initPluginAuthority' | 'lifecycleChecks' | 'pda' | 'resultsOffset'
> & {
  type: 'Oracle';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks: LifecycleChecks;
  pda?: ExtraAccount;
  resultsOffset?: ValidationResultsOffset;
};

export type OracleUpdateInfoArgs = Omit<
  BaseOracleUpdateInfoArgs,
  'lifecycleChecks' | 'pda' | 'resultsOffset'
> & {
  key: ExternalPluginKey;
  lifecycleChecks?: LifecycleChecks;
  pda?: ExtraAccount;
  resultsOffset?: ValidationResultsOffset;
};

export function oracleInitInfoArgsToBase(
  o: OracleInitInfoArgs
): BaseOracleInitInfoArgs {
  return {
    baseAddress: o.baseAddress,
    pda: o.pda ? extraAccountToBase(o.pda) : null,
    lifecycleChecks: lifecycleChecksToBase(o.lifecycleChecks),
    initPluginAuthority: o.initPluginAuthority
      ? pluginAuthorityToBase(o.initPluginAuthority)
      : null,
    resultsOffset: o.resultsOffset
      ? validationResultsOffsetToBase(o.resultsOffset)
      : null,
  };
}

export function oracleUpdateInfoArgsToBase(
  o: OracleUpdateInfoArgs
): BaseOracleUpdateInfoArgs {
  return {
    pda: o.pda ? extraAccountToBase(o.pda) : null,
    lifecycleChecks: o.lifecycleChecks
      ? lifecycleChecksToBase(o.lifecycleChecks)
      : null,
    resultsOffset: o.resultsOffset
      ? validationResultsOffsetToBase(o.resultsOffset)
      : null,
  };
}

export function oracleFromBase(
  s: BaseOracle,
  r: ExternalRegistryRecord,
  account: Uint8Array
): Oracle {
  return {
    ...s,
    pda:
      s.pda.__option === 'Some' ? extraAccountFromBase(s.pda.value) : undefined,
    resultsOffset: validationResultsOffsetFromBase(s.resultsOffset),
  };
}

export function findOracleAccount(
  context: Pick<Context, 'eddsa'>,
  oracle: Pick<Oracle, 'baseAddress' | 'pda'>,
  inputs: {
    asset?: PublicKey;
    collection?: PublicKey;
    recipient?: PublicKey;
    owner?: PublicKey;
  }
): PublicKey {
  if (!oracle.pda) {
    return oracle.baseAddress;
  }

  return extraAccountToAccountMeta(context, oracle.pda, {
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

export const oracleManifest: ExternalPluginManifest<
  Oracle,
  BaseOracle,
  OracleInitInfoArgs,
  BaseOracleInitInfoArgs,
  OracleUpdateInfoArgs,
  BaseOracleUpdateInfoArgs
> = {
  type: 'Oracle',
  fromBase: oracleFromBase,
  initToBase: oracleInitInfoArgsToBase,
  updateToBase: oracleUpdateInfoArgsToBase,
};
