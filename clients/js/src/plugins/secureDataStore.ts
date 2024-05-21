import {
  BaseSecureDataStore,
  BaseSecureDataStoreInitInfoArgs,
  BaseSecureDataStoreUpdateInfoArgs,
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

export type SecureDataStore = Omit<BaseSecureDataStore, 'dataAuthority'> & {
  dataAuthority: PluginAuthority;
  data?: any;
};

export type SecureDataStorePlugin = BaseExternalPluginAdapter &
  SecureDataStore & {
    type: 'SecureDataStore';
    dataAuthority: PluginAuthority;
  };

export type SecureDataStoreInitInfoArgs = Omit<
  BaseSecureDataStoreInitInfoArgs,
  'initPluginAuthority' | 'lifecycleChecks' | 'dataAuthority'
> & {
  type: 'SecureDataStore';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks?: LifecycleChecks;
  schema?: ExternalPluginAdapterSchema;
  dataAuthority: PluginAuthority;
};

export type SecureDataStoreUpdateInfoArgs = Omit<
  BaseSecureDataStoreUpdateInfoArgs,
  'schema'
> & {
  key: ExternalPluginAdapterKey;
  schema?: ExternalPluginAdapterSchema;
};

export function secureDataStoreInitInfoArgsToBase(
  d: SecureDataStoreInitInfoArgs
): BaseSecureDataStoreInitInfoArgs {
  return {
    dataAuthority: pluginAuthorityToBase(d.dataAuthority),
    initPluginAuthority: d.initPluginAuthority
      ? pluginAuthorityToBase(d.initPluginAuthority)
      : null,
    schema: d.schema ? d.schema : null,
  };
}

export function secureDataStoreUpdateInfoArgsToBase(
  d: SecureDataStoreUpdateInfoArgs
): BaseSecureDataStoreUpdateInfoArgs {
  return {
    schema: d.schema ? d.schema : null,
  };
}

export function secureDataStoreFromBase(
  s: BaseSecureDataStore,
  r: ExternalRegistryRecord,
  account: Uint8Array
): SecureDataStore {
  return {
    ...s,
    dataAuthority: pluginAuthorityFromBase(s.dataAuthority),
    data: parseExternalPluginAdapterData(s, r, account),
  };
}

export const secureDataStoreManifest: ExternalPluginAdapterManifest<
  SecureDataStore,
  BaseSecureDataStore,
  SecureDataStoreInitInfoArgs,
  BaseSecureDataStoreInitInfoArgs,
  SecureDataStoreUpdateInfoArgs,
  BaseSecureDataStoreUpdateInfoArgs
> = {
  type: 'SecureDataStore',
  fromBase: secureDataStoreFromBase,
  initToBase: secureDataStoreInitInfoArgsToBase,
  updateToBase: secureDataStoreUpdateInfoArgsToBase,
};
