import { PublicKey } from "@metaplex-foundation/umi";
import { ExtraAccount, extraAccountToBase } from "./extraAccount";
import { BaseOracleInitInfoArgs, BaseOracleUpdateInfoArgs } from "../generated";
import { LifecycleChecks, lifecycleChecksToBase } from "./lifecycleChecks";
import { PluginAuthority, pluginAuthorityToBase } from "./pluginAuthority";
import { ExternalPluginManifest } from "./externalPluginManifest";
import { BaseExternalPlugin } from "./externalPlugins";
import { ExternalPluginKey } from "./externalPluginKey";

export type Oracle = {
  pda?: ExtraAccount
}

export type OraclePlugin = BaseExternalPlugin & Oracle & {
  type: 'Oracle',
  baseAddress: PublicKey,
}

export type OracleInitInfoArgs = Omit<BaseOracleInitInfoArgs, 'initPluginAuthority' | 'lifecycleChecks'> & {
  type: 'Oracle'
  initPluginAuthority?: PluginAuthority
  lifecycleChecks?: LifecycleChecks
  pda?: ExtraAccount
}

export type OracleUpdateInfoArgs = Omit<BaseOracleUpdateInfoArgs, 'lifecycleChecks' | 'pda'> & {
  key: ExternalPluginKey,
  lifecycleChecks?: LifecycleChecks
  pda?: ExtraAccount
}


export function oracleInitInfoArgsToBase(o: OracleInitInfoArgs): BaseOracleInitInfoArgs {
  return {
    baseAddress: o.baseAddress,
    pda: o.pda ? extraAccountToBase(o.pda) : null,
    lifecycleChecks: o.lifecycleChecks ? lifecycleChecksToBase(o.lifecycleChecks) : null,
    initPluginAuthority: o.initPluginAuthority ? pluginAuthorityToBase(o.initPluginAuthority) : null,
  }
}

export function oracleUpdateInfoArgsToBase(o: OracleUpdateInfoArgs): BaseOracleUpdateInfoArgs {
  return {
    pda: o.pda ? extraAccountToBase(o.pda) : null,
    lifecycleChecks: o.lifecycleChecks ? lifecycleChecksToBase(o.lifecycleChecks) : null,
  }
}

export const oracleManifest: ExternalPluginManifest<
  OracleInitInfoArgs, BaseOracleInitInfoArgs,
  OracleUpdateInfoArgs, BaseOracleUpdateInfoArgs> = {
    type: 'Oracle',
    initToBase: oracleInitInfoArgsToBase,
    updateToBase: oracleUpdateInfoArgsToBase,
  }