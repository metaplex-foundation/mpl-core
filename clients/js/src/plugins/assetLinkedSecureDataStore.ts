import {
  BaseAssetLinkedSecureDataStore,
  BaseAssetLinkedSecureDataStoreInitInfoArgs,
  BaseAssetLinkedSecureDataStoreUpdateInfoArgs,
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

export type AssetLinkedSecureDataStore = Omit<
  BaseAssetLinkedSecureDataStore,
  'dataAuthority'
> & {
  dataAuthority: PluginAuthority;
  data?: any;
};

export type AssetLinkedSecureDataStorePlugin = BaseExternalPluginAdapter &
  AssetLinkedSecureDataStore & {
    type: 'AssetLinkedSecureDataStore';
    dataAuthority: PluginAuthority;
  };

export type AssetLinkedSecureDataStoreInitInfoArgs = Omit<
  BaseAssetLinkedSecureDataStoreInitInfoArgs,
  'initPluginAuthority' | 'lifecycleChecks' | 'dataAuthority'
> & {
  type: 'AssetLinkedSecureDataStore';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks?: LifecycleChecks;
  schema?: ExternalPluginAdapterSchema;
  dataAuthority: PluginAuthority;
};

export type AssetLinkedSecureDataStoreUpdateInfoArgs = Omit<
  BaseAssetLinkedSecureDataStoreUpdateInfoArgs,
  'schema'
> & {
  key: ExternalPluginAdapterKey;
  schema?: ExternalPluginAdapterSchema;
};

export function assetLinkedSecureDataStoreInitInfoArgsToBase(
  d: AssetLinkedSecureDataStoreInitInfoArgs
): BaseAssetLinkedSecureDataStoreInitInfoArgs {
  return {
    dataAuthority: pluginAuthorityToBase(d.dataAuthority),
    initPluginAuthority: d.initPluginAuthority
      ? pluginAuthorityToBase(d.initPluginAuthority)
      : null,
    schema: d.schema ? d.schema : null,
  };
}

export function assetLinkedSecureDataStoreUpdateInfoArgsToBase(
  d: AssetLinkedSecureDataStoreUpdateInfoArgs
): BaseAssetLinkedSecureDataStoreUpdateInfoArgs {
  return {
    schema: d.schema ? d.schema : null,
  };
}

export function assetLinkedSecureDataStoreFromBase(
  s: BaseAssetLinkedSecureDataStore,
  r: ExternalRegistryRecord,
  account: Uint8Array
): AssetLinkedSecureDataStore {
  return {
    ...s,
    dataAuthority: pluginAuthorityFromBase(s.dataAuthority),
    data: parseExternalPluginAdapterData(s, r, account),
  };
}

export const assetLinkedSecureDataStoreManifest: ExternalPluginAdapterManifest<
  AssetLinkedSecureDataStore,
  BaseAssetLinkedSecureDataStore,
  AssetLinkedSecureDataStoreInitInfoArgs,
  BaseAssetLinkedSecureDataStoreInitInfoArgs,
  AssetLinkedSecureDataStoreUpdateInfoArgs,
  BaseAssetLinkedSecureDataStoreUpdateInfoArgs
> = {
  type: 'AssetLinkedSecureDataStore',
  fromBase: assetLinkedSecureDataStoreFromBase,
  initToBase: assetLinkedSecureDataStoreInitInfoArgsToBase,
  updateToBase: assetLinkedSecureDataStoreUpdateInfoArgsToBase,
};
