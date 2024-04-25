import {
  BaseDataStore,
  BaseDataStoreInitInfoArgs,
  BaseDataStoreUpdateInfoArgs,
  ExternalPluginSchema,
  ExternalRegistryRecord,
} from '../generated';
import { ExternalPluginKey } from './externalPluginKey';
import { ExternalPluginManifest } from './externalPluginManifest';
import { BaseExternalPlugin, parseExternalPluginData } from './externalPlugins';
import { LifecycleChecks } from './lifecycleChecks';
import {
  PluginAuthority,
  pluginAuthorityFromBase,
  pluginAuthorityToBase,
} from './pluginAuthority';

export type DataStore = Omit<BaseDataStore, 'dataAuthority'> & {
  dataAuthority: PluginAuthority;
  data?: any;
};

export type DataStorePlugin = BaseExternalPlugin &
  DataStore & {
    type: 'DataStore';
    dataAuthority: PluginAuthority;
  };

export type DataStoreInitInfoArgs = Omit<
  BaseDataStoreInitInfoArgs,
  'initPluginAuthority' | 'lifecycleChecks' | 'dataAuthority'
> & {
  type: 'DataStore';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks?: LifecycleChecks;
  schema?: ExternalPluginSchema;
  dataAuthority: PluginAuthority;
};

export type DataStoreUpdateInfoArgs = Omit<
  BaseDataStoreUpdateInfoArgs,
  'schema'
> & {
  key: ExternalPluginKey;
  schema?: ExternalPluginSchema;
};

export function dataStoreInitInfoArgsToBase(
  d: DataStoreInitInfoArgs
): BaseDataStoreInitInfoArgs {
  return {
    dataAuthority: pluginAuthorityToBase(d.dataAuthority),
    initPluginAuthority: d.initPluginAuthority
      ? pluginAuthorityToBase(d.initPluginAuthority)
      : null,
    schema: d.schema ? d.schema : null,
  };
}

export function dataStoreUpdateInfoArgsToBase(
  d: DataStoreUpdateInfoArgs
): BaseDataStoreUpdateInfoArgs {
  return {
    schema: d.schema ? d.schema : null,
  };
}

export function dataStoreFromBase(
  s: BaseDataStore,
  r: ExternalRegistryRecord,
  account: Uint8Array
): DataStore {
  return {
    ...s,
    dataAuthority: pluginAuthorityFromBase(s.dataAuthority),
    data: parseExternalPluginData(s, r, account),
  };
}

export const dataStoreManifest: ExternalPluginManifest<
  DataStore,
  BaseDataStore,
  DataStoreInitInfoArgs,
  BaseDataStoreInitInfoArgs,
  DataStoreUpdateInfoArgs,
  BaseDataStoreUpdateInfoArgs
> = {
  type: 'DataStore',
  fromBase: dataStoreFromBase,
  initToBase: dataStoreInitInfoArgsToBase,
  updateToBase: dataStoreUpdateInfoArgsToBase,
};
