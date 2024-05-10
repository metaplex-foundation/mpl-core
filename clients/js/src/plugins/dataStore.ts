import {
  BaseDataStore,
  BaseDataStoreInitInfoArgs,
  BaseDataStoreUpdateInfoArgs,
  ExternalPluginAdapterSchema,
  ExternalRegistryRecord,
} from '../generated';
import { ExternalPluginAdapterKey } from './externalPluginAdapterKey';
import { ExternalPluginAdapterManifest } from './externalPluginAdapterManifest';
import { BaseExternalPluginAdapter } from './externalPluginAdapters';
import { parseExternalPluginAdapterData } from './lib';
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

export type DataStorePlugin = BaseExternalPluginAdapter &
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
  schema?: ExternalPluginAdapterSchema;
  dataAuthority: PluginAuthority;
};

export type DataStoreUpdateInfoArgs = Omit<
  BaseDataStoreUpdateInfoArgs,
  'schema'
> & {
  key: ExternalPluginAdapterKey;
  schema?: ExternalPluginAdapterSchema;
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
    data: parseExternalPluginAdapterData(s, r, account),
  };
}

export const dataStoreManifest: ExternalPluginAdapterManifest<
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
