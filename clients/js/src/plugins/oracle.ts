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
import { ExternalPluginAdapterManifest } from './externalPluginAdapterManifest';
import { BaseExternalPluginAdapter } from './externalPluginAdapters';
import { ExternalPluginAdapterKey } from './externalPluginAdapterKey';
import {
  ValidationResultsOffset,
  validationResultsOffsetFromBase,
  validationResultsOffsetToBase,
} from './validationResultsOffset';

export type Oracle = Omit<BaseOracle, 'baseAddressConfig' | 'resultsOffset'> & {
  baseAddressConfig?: ExtraAccount;
  resultsOffset: ValidationResultsOffset;
};

export type OraclePlugin = BaseExternalPluginAdapter &
  Oracle & {
    type: 'Oracle';
  };

export type OracleInitInfoArgs = Omit<
  BaseOracleInitInfoArgs,
  | 'initPluginAuthority'
  | 'lifecycleChecks'
  | 'baseAddressConfig'
  | 'resultsOffset'
> & {
  type: 'Oracle';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks: LifecycleChecks;
  baseAddressConfig?: ExtraAccount;
  resultsOffset?: ValidationResultsOffset;
};

export type OracleUpdateInfoArgs = Omit<
  BaseOracleUpdateInfoArgs,
  'lifecycleChecks' | 'baseAddressConfig' | 'resultsOffset'
> & {
  key: ExternalPluginAdapterKey;
  lifecycleChecks?: LifecycleChecks;
  baseAddressConfig?: ExtraAccount;
  resultsOffset?: ValidationResultsOffset;
};

export function oracleInitInfoArgsToBase(
  o: OracleInitInfoArgs
): BaseOracleInitInfoArgs {
  return {
    baseAddress: o.baseAddress,
    baseAddressConfig: o.baseAddressConfig
      ? extraAccountToBase(o.baseAddressConfig)
      : null,
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
    baseAddressConfig: o.baseAddressConfig
      ? extraAccountToBase(o.baseAddressConfig)
      : null,
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
    baseAddressConfig:
      s.baseAddressConfig.__option === 'Some'
        ? extraAccountFromBase(s.baseAddressConfig.value)
        : undefined,
    resultsOffset: validationResultsOffsetFromBase(s.resultsOffset),
  };
}

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

export const oracleManifest: ExternalPluginAdapterManifest<
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
