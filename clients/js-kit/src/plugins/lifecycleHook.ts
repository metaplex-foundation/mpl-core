import { type Address } from '@solana/addresses';
import {
  type ExtraAccount,
  extraAccountFromBase,
  extraAccountToBase,
} from './extraAccount';
import {
  type BaseLifecycleHook,
  type BaseLifecycleHookInitInfoArgs,
  type BaseLifecycleHookUpdateInfoArgs,
  type ExternalPluginAdapterSchema,
  type ExternalRegistryRecord,
} from '../generated';
import { type LifecycleChecks, lifecycleChecksToBase } from './lifecycleChecks';
import {
  type PluginAuthority,
  pluginAuthorityFromBase,
  pluginAuthorityToBase,
} from './pluginAuthority';
import { type BaseExternalPluginAdapter } from './externalPluginAdapters';
import { type ExternalPluginAdapterManifest } from './externalPluginAdapterManifest';
import { type ExternalPluginAdapterKey } from './externalPluginAdapterKey';
import { parseExternalPluginAdapterData } from './lib';

export type LifecycleHook = Omit<
  BaseLifecycleHook,
  'extraAccounts' | 'dataAuthority'
> & {
  extraAccounts?: Array<ExtraAccount>;
  dataAuthority?: PluginAuthority;
  data?: unknown;
};

export type LifecycleHookPlugin = BaseExternalPluginAdapter &
  LifecycleHook & {
    type: 'LifecycleHook';
    hookedProgram: Address;
  };

export type LifecycleHookInitInfoArgs = Omit<
  BaseLifecycleHookInitInfoArgs,
  | 'initPluginAuthority'
  | 'lifecycleChecks'
  | 'schema'
  | 'extraAccounts'
  | 'dataAuthority'
> & {
  type: 'LifecycleHook';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks: LifecycleChecks;
  schema?: ExternalPluginAdapterSchema;
  extraAccounts?: Array<ExtraAccount>;
  dataAuthority?: PluginAuthority;
};

export type LifecycleHookUpdateInfoArgs = Omit<
  BaseLifecycleHookUpdateInfoArgs,
  'lifecycleChecks' | 'extraAccounts' | 'schema'
> & {
  key: ExternalPluginAdapterKey;
  lifecycleChecks?: LifecycleChecks;
  extraAccounts?: Array<ExtraAccount>;
  schema?: ExternalPluginAdapterSchema;
};

export function lifecycleHookInitInfoArgsToBase(
  l: LifecycleHookInitInfoArgs
): BaseLifecycleHookInitInfoArgs {
  return {
    extraAccounts: l.extraAccounts
      ? l.extraAccounts.map(extraAccountToBase)
      : null,
    hookedProgram: l.hookedProgram,
    initPluginAuthority: l.initPluginAuthority
      ? pluginAuthorityToBase(l.initPluginAuthority)
      : null,
    lifecycleChecks: lifecycleChecksToBase(l.lifecycleChecks),
    schema: l.schema ?? null,
    dataAuthority: l.dataAuthority
      ? pluginAuthorityToBase(l.dataAuthority)
      : null,
  };
}

export function lifecycleHookUpdateInfoArgsToBase(
  l: LifecycleHookUpdateInfoArgs
): BaseLifecycleHookUpdateInfoArgs {
  return {
    lifecycleChecks: l.lifecycleChecks
      ? lifecycleChecksToBase(l.lifecycleChecks)
      : null,
    extraAccounts: l.extraAccounts
      ? l.extraAccounts.map(extraAccountToBase)
      : null,
    schema: l.schema ?? null,
  };
}

export function lifecycleHookFromBase(
  s: BaseLifecycleHook,
  r: ExternalRegistryRecord,
  account: Uint8Array
): LifecycleHook {
  return {
    ...s,
    extraAccounts:
      s.extraAccounts.__option === 'Some'
        ? s.extraAccounts.value.map(extraAccountFromBase)
        : undefined,
    data: parseExternalPluginAdapterData(s, r, account),
    dataAuthority:
      s.dataAuthority.__option === 'Some'
        ? pluginAuthorityFromBase(s.dataAuthority.value)
        : undefined,
  };
}

export const lifecycleHookManifest: ExternalPluginAdapterManifest<
  LifecycleHook,
  BaseLifecycleHook,
  LifecycleHookInitInfoArgs,
  BaseLifecycleHookInitInfoArgs,
  LifecycleHookUpdateInfoArgs,
  BaseLifecycleHookUpdateInfoArgs
> = {
  type: 'LifecycleHook',
  fromBase: lifecycleHookFromBase,
  initToBase: lifecycleHookInitInfoArgsToBase,
  updateToBase: lifecycleHookUpdateInfoArgsToBase,
};
