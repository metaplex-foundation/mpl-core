import {
  type BaseAppData,
  type BaseAppDataInitInfoArgs,
  type BaseAppDataUpdateInfoArgs,
  type ExternalPluginAdapterSchema,
  type ExternalRegistryRecord,
} from '../generated';
import { type ExternalPluginAdapterKey } from './externalPluginAdapterKey';
import { type ExternalPluginAdapterManifest } from './externalPluginAdapterManifest';
import { type BaseExternalPluginAdapter } from './externalPluginAdapters';
import { parseExternalPluginAdapterData } from './lib';
import { type LifecycleChecks } from './lifecycleChecks';
import {
  type PluginAuthority,
  pluginAuthorityFromBase,
  pluginAuthorityToBase,
} from './pluginAuthority';

export type AppData = Omit<BaseAppData, 'dataAuthority'> & {
  dataAuthority: PluginAuthority;
  data?: unknown;
};

export type AppDataPlugin = BaseExternalPluginAdapter &
  AppData & {
    type: 'AppData';
    dataAuthority: PluginAuthority;
  };

export type AppDataInitInfoArgs = Omit<
  BaseAppDataInitInfoArgs,
  'initPluginAuthority' | 'lifecycleChecks' | 'dataAuthority'
> & {
  type: 'AppData';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks?: LifecycleChecks;
  schema?: ExternalPluginAdapterSchema;
  dataAuthority: PluginAuthority;
};

export type AppDataUpdateInfoArgs = Omit<
  BaseAppDataUpdateInfoArgs,
  'schema'
> & {
  key: ExternalPluginAdapterKey;
  schema?: ExternalPluginAdapterSchema;
};

export function appDataInitInfoArgsToBase(
  d: AppDataInitInfoArgs
): BaseAppDataInitInfoArgs {
  return {
    dataAuthority: pluginAuthorityToBase(d.dataAuthority),
    initPluginAuthority: d.initPluginAuthority
      ? pluginAuthorityToBase(d.initPluginAuthority)
      : null,
    schema: d.schema ?? null,
  };
}

export function appDataUpdateInfoArgsToBase(
  d: AppDataUpdateInfoArgs
): BaseAppDataUpdateInfoArgs {
  return {
    schema: d.schema ?? null,
  };
}

export function appDataFromBase(
  s: BaseAppData,
  r: ExternalRegistryRecord,
  account: Uint8Array
): AppData {
  return {
    ...s,
    dataAuthority: pluginAuthorityFromBase(s.dataAuthority),
    data: parseExternalPluginAdapterData(s, r, account),
  };
}

export const appDataManifest: ExternalPluginAdapterManifest<
  AppData,
  BaseAppData,
  AppDataInitInfoArgs,
  BaseAppDataInitInfoArgs,
  AppDataUpdateInfoArgs,
  BaseAppDataUpdateInfoArgs
> = {
  type: 'AppData',
  fromBase: appDataFromBase,
  initToBase: appDataInitInfoArgsToBase,
  updateToBase: appDataUpdateInfoArgsToBase,
};
