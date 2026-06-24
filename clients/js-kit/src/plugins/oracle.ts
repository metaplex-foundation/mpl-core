import {
  type ExtraAccount,
  extraAccountFromBase,
  extraAccountToBase,
} from './extraAccount';
import {
  type BaseOracle,
  type BaseOracleInitInfoArgs,
  type BaseOracleUpdateInfoArgs,
  type ExternalRegistryRecord,
  getOracleValidationDecoder,
  type OracleValidation,
} from '../generated';
import { type LifecycleChecks, lifecycleChecksToBase } from './lifecycleChecks';
import { type PluginAuthority, pluginAuthorityToBase } from './pluginAuthority';
import { type ExternalPluginAdapterManifest } from './externalPluginAdapterManifest';
import { type BaseExternalPluginAdapter } from './externalPluginAdapters';
import { type ExternalPluginAdapterKey } from './externalPluginAdapterKey';
import {
  type ValidationResultsOffset,
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
  _r: ExternalRegistryRecord,
  _account: Uint8Array
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

  return getOracleValidationDecoder().read(data, offs)[0];
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
