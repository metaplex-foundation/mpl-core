import {
  BaseAssetLinkedAppData,
  BaseAssetLinkedAppDataInitInfoArgs,
  BaseAssetLinkedAppDataUpdateInfoArgs,
  ExternalPluginAdapterSchema,
  ExternalRegistryRecord,
} from '../generated';
import { ExternalPluginAdapterKey } from './externalPluginAdapterKey';
import { ExternalPluginAdapterManifest } from './externalPluginAdapterManifest';
import { BaseExternalPluginAdapter } from './externalPluginAdapters';
import { LifecycleChecks } from './lifecycleChecks';
import {
  PluginAuthority,
  pluginAuthorityFromBase,
  pluginAuthorityToBase,
} from './pluginAuthority';

export type AssetLinkedAppData = Omit<
  BaseAssetLinkedAppData,
  'dataAuthority'
> & {
  dataAuthority: PluginAuthority;
  data?: any;
};

export type AssetLinkedAppDataPlugin = BaseExternalPluginAdapter &
  AssetLinkedAppData & {
    type: 'AssetLinkedAppData';
    dataAuthority: PluginAuthority;
  };

export type AssetLinkedAppDataInitInfoArgs = Omit<
  BaseAssetLinkedAppDataInitInfoArgs,
  'initPluginAuthority' | 'lifecycleChecks' | 'dataAuthority'
> & {
  type: 'AssetLinkedAppData';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks?: LifecycleChecks;
  schema?: ExternalPluginAdapterSchema;
  dataAuthority: PluginAuthority;
};

export type AssetLinkedAppDataUpdateInfoArgs = Omit<
  BaseAssetLinkedAppDataUpdateInfoArgs,
  'schema'
> & {
  key: ExternalPluginAdapterKey;
  schema?: ExternalPluginAdapterSchema;
};

export function assetLinkedAppDataInitInfoArgsToBase(
  d: AssetLinkedAppDataInitInfoArgs
): BaseAssetLinkedAppDataInitInfoArgs {
  return {
    dataAuthority: pluginAuthorityToBase(d.dataAuthority),
    initPluginAuthority: d.initPluginAuthority
      ? pluginAuthorityToBase(d.initPluginAuthority)
      : null,
    schema: d.schema ? d.schema : null,
  };
}

export function assetLinkedAppDataUpdateInfoArgsToBase(
  d: AssetLinkedAppDataUpdateInfoArgs
): BaseAssetLinkedAppDataUpdateInfoArgs {
  return {
    schema: d.schema ? d.schema : null,
  };
}

export function assetLinkedAppDataFromBase(
  s: BaseAssetLinkedAppData,
  r: ExternalRegistryRecord,
  account: Uint8Array
): AssetLinkedAppData {
  return {
    ...s,
    dataAuthority: pluginAuthorityFromBase(s.dataAuthority),
    // plugin has no data but injected in the derivation of the asset
  };
}

export const assetLinkedAppDataManifest: ExternalPluginAdapterManifest<
  AssetLinkedAppData,
  BaseAssetLinkedAppData,
  AssetLinkedAppDataInitInfoArgs,
  BaseAssetLinkedAppDataInitInfoArgs,
  AssetLinkedAppDataUpdateInfoArgs,
  BaseAssetLinkedAppDataUpdateInfoArgs
> = {
  type: 'AssetLinkedAppData',
  fromBase: assetLinkedAppDataFromBase,
  initToBase: assetLinkedAppDataInitInfoArgsToBase,
  updateToBase: assetLinkedAppDataUpdateInfoArgsToBase,
};
