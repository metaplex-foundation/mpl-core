import {
  BaseDataStore,
  BaseDataStoreInitInfoArgs,
  BaseDataStoreUpdateInfoArgs,
  PluginAdapterSchema,
  AdapterRegistryRecord,
} from '../generated';
import { PluginAdapterKey } from './pluginAdapterKey';
import { PluginAdapterManifest } from './pluginAdapterManifest';
import { BasePluginAdapter } from './pluginAdapters';
import { parsePluginAdapterData } from './lib';
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

export type DataStorePlugin = BasePluginAdapter &
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
  schema?: PluginAdapterSchema;
  dataAuthority: PluginAuthority;
};

export type DataStoreUpdateInfoArgs = Omit<
  BaseDataStoreUpdateInfoArgs,
  'schema'
> & {
  key: PluginAdapterKey;
  schema?: PluginAdapterSchema;
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
  r: AdapterRegistryRecord,
  account: Uint8Array
): DataStore {
  return {
    ...s,
    dataAuthority: pluginAuthorityFromBase(s.dataAuthority),
    data: parsePluginAdapterData(s, r, account),
  };
}

export const dataStoreManifest: PluginAdapterManifest<
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
