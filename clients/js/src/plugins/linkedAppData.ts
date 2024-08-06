import {
  BaseLinkedAppData,
  BaseLinkedAppDataInitInfoArgs,
  BaseLinkedAppDataUpdateInfoArgs,
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

export type LinkedAppData = Omit<BaseLinkedAppData, 'dataAuthority'> & {
  dataAuthority: PluginAuthority;
  data?: any;
};

export type LinkedAppDataPlugin = BaseExternalPluginAdapter &
  LinkedAppData & {
    type: 'LinkedAppData';
    dataAuthority: PluginAuthority;
  };

export type LinkedAppDataInitInfoArgs = Omit<
  BaseLinkedAppDataInitInfoArgs,
  'initPluginAuthority' | 'lifecycleChecks' | 'dataAuthority'
> & {
  type: 'LinkedAppData';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks?: LifecycleChecks;
  schema?: ExternalPluginAdapterSchema;
  dataAuthority: PluginAuthority;
};

export type LinkedAppDataUpdateInfoArgs = Omit<
  BaseLinkedAppDataUpdateInfoArgs,
  'schema'
> & {
  key: ExternalPluginAdapterKey;
  schema?: ExternalPluginAdapterSchema;
};

export function linkedAppDataInitInfoArgsToBase(
  d: LinkedAppDataInitInfoArgs
): BaseLinkedAppDataInitInfoArgs {
  return {
    dataAuthority: pluginAuthorityToBase(d.dataAuthority),
    initPluginAuthority: d.initPluginAuthority
      ? pluginAuthorityToBase(d.initPluginAuthority)
      : null,
    schema: d.schema ?? null,
  };
}

export function linkedAppDataUpdateInfoArgsToBase(
  d: LinkedAppDataUpdateInfoArgs
): BaseLinkedAppDataUpdateInfoArgs {
  return {
    schema: d.schema ?? null,
  };
}

export function linkedAppDataFromBase(
  s: BaseLinkedAppData,
  r: ExternalRegistryRecord,
  account: Uint8Array
): LinkedAppData {
  return {
    ...s,
    dataAuthority: pluginAuthorityFromBase(s.dataAuthority),
    // plugin has no data but injected in the derivation of the asset
  };
}

export const linkedAppDataManifest: ExternalPluginAdapterManifest<
  LinkedAppData,
  BaseLinkedAppData,
  LinkedAppDataInitInfoArgs,
  BaseLinkedAppDataInitInfoArgs,
  LinkedAppDataUpdateInfoArgs,
  BaseLinkedAppDataUpdateInfoArgs
> = {
  type: 'LinkedAppData',
  fromBase: linkedAppDataFromBase,
  initToBase: linkedAppDataInitInfoArgsToBase,
  updateToBase: linkedAppDataUpdateInfoArgsToBase,
};
