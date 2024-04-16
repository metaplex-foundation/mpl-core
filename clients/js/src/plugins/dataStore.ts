import { BaseDataStore, BaseDataStoreInitInfoArgs, BaseDataStoreUpdateInfoArgs, ExternalPluginSchema } from "../generated";
import { ExternalPluginKey } from "./externalPluginKey";
import { ExternalPluginManifest } from "./externalPluginManifest";
import { BaseExternalPlugin } from "./externalPlugins";
import { LifecycleChecks } from "./lifecycleChecks";
import { PluginAuthority, pluginAuthorityToBase } from "./pluginAuthority";

export type DataStore = BaseDataStore & {
  data?: any
}

export type DataStorePlugin = BaseExternalPlugin & DataStore & {
  type: 'DataStore',
  dataAuthority: PluginAuthority,
}

export type DataStoreInitInfoArgs = Omit<BaseDataStoreInitInfoArgs, 'initPluginAuthority' | 'lifecycleChecks' | 'dataAuthority'> & {
  type: 'DataStore'
  initPluginAuthority?: PluginAuthority
  lifecycleChecks?: LifecycleChecks
  schema?: ExternalPluginSchema
  dataAuthority: PluginAuthority
}

export type DataStoreUpdateInfoArgs = Omit<BaseDataStoreUpdateInfoArgs, 'schema'> & {
  key: ExternalPluginKey,
  schema?: ExternalPluginSchema
}

export function dataStoreInitInfoArgsToBase(d: DataStoreInitInfoArgs): BaseDataStoreInitInfoArgs {
  return {
   dataAuthority: pluginAuthorityToBase(d.dataAuthority),
   initPluginAuthority: d.initPluginAuthority ? pluginAuthorityToBase(d.initPluginAuthority) : null,
   schema: d.schema ? d.schema : null,
  }
}

export function dataStoreUpdateInfoArgsToBase(d: DataStoreUpdateInfoArgs): BaseDataStoreUpdateInfoArgs {
  return {
    schema: d.schema ? d.schema : null,
  }
}

export const dataStoreManifest: ExternalPluginManifest<
  DataStoreInitInfoArgs, BaseDataStoreInitInfoArgs,
  DataStoreUpdateInfoArgs, BaseDataStoreUpdateInfoArgs> = {
    type: 'LifecycleHook',
    initToBase: dataStoreInitInfoArgsToBase,
    updateToBase: dataStoreUpdateInfoArgsToBase,
  }