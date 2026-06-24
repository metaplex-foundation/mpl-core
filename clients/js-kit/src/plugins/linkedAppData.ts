import {
  type BaseLinkedAppData,
  type BaseLinkedAppDataInitInfoArgs,
  type BaseLinkedAppDataUpdateInfoArgs,
  type ExternalPluginAdapterSchema,
  type ExternalRegistryRecord,
} from '../generated';
import { type ExternalPluginAdapterKey } from './externalPluginAdapterKey';
import { type ExternalPluginAdapterManifest } from './externalPluginAdapterManifest';
import { type BaseExternalPluginAdapter } from './externalPluginAdapters';
import { type LifecycleChecks } from './lifecycleChecks';
import {
  type PluginAuthority,
  pluginAuthorityFromBase,
  pluginAuthorityToBase,
} from './pluginAuthority';

export type LinkedAppData = Omit<BaseLinkedAppData, 'dataAuthority'> & {
  dataAuthority: PluginAuthority;
  data?: unknown;
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
  _r: ExternalRegistryRecord,
  _account: Uint8Array
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
