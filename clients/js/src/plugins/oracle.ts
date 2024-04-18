import {
  ExtraAccount,
  extraAccountFromBase,
  extraAccountToBase,
} from './extraAccount';
import {
  BaseOracle,
  BaseOracleInitInfoArgs,
  BaseOracleUpdateInfoArgs,
} from '../generated';
import { LifecycleChecks, lifecycleChecksToBase } from './lifecycleChecks';
import { PluginAuthority, pluginAuthorityToBase } from './pluginAuthority';
import { ExternalPluginManifest } from './externalPluginManifest';
import { BaseExternalPlugin } from './externalPlugins';
import { ExternalPluginKey } from './externalPluginKey';

export type Oracle = Omit<BaseOracle, 'pda'> & {
  pda?: ExtraAccount;
};

export type OraclePlugin = BaseExternalPlugin &
  Oracle & {
    type: 'Oracle';
  };

export type OracleInitInfoArgs = Omit<
  BaseOracleInitInfoArgs,
  'initPluginAuthority' | 'lifecycleChecks'
> & {
  type: 'Oracle';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks?: LifecycleChecks;
  pda?: ExtraAccount;
};

export type OracleUpdateInfoArgs = Omit<
  BaseOracleUpdateInfoArgs,
  'lifecycleChecks' | 'pda'
> & {
  key: ExternalPluginKey;
  lifecycleChecks?: LifecycleChecks;
  pda?: ExtraAccount;
};

export function oracleInitInfoArgsToBase(
  o: OracleInitInfoArgs
): BaseOracleInitInfoArgs {
  return {
    baseAddress: o.baseAddress,
    pda: o.pda ? extraAccountToBase(o.pda) : null,
    lifecycleChecks: o.lifecycleChecks
      ? lifecycleChecksToBase(o.lifecycleChecks)
      : null,
    initPluginAuthority: o.initPluginAuthority
      ? pluginAuthorityToBase(o.initPluginAuthority)
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
  };
}

export function oracleFromBase(s: BaseOracle, account: Uint8Array): Oracle {
  return {
    ...s,
    pda:
      s.pda.__option === 'Some' ? extraAccountFromBase(s.pda.value) : undefined,
  };
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
