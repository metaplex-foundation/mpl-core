import { type Address } from '@solana/addresses';
import {
  type ExtraAccount,
  extraAccountFromBase,
  extraAccountToBase,
} from './extraAccount';
import {
  type BaseLinkedLifecycleHook,
  type BaseLinkedLifecycleHookInitInfoArgs,
  type BaseLinkedLifecycleHookUpdateInfoArgs,
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

export type LinkedLifecycleHook = Omit<
  BaseLinkedLifecycleHook,
  'extraAccounts' | 'dataAuthority'
> & {
  extraAccounts?: Array<ExtraAccount>;
  dataAuthority?: PluginAuthority;
  data?: unknown;
};

export type LinkedLifecycleHookPlugin = BaseExternalPluginAdapter &
  LinkedLifecycleHook & {
    type: 'LinkedLifecycleHook';
    hookedProgram: Address;
  };

export type LinkedLifecycleHookInitInfoArgs = Omit<
  BaseLinkedLifecycleHookInitInfoArgs,
  | 'initPluginAuthority'
  | 'lifecycleChecks'
  | 'schema'
  | 'extraAccounts'
  | 'dataAuthority'
> & {
  type: 'LinkedLifecycleHook';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks: LifecycleChecks;
  schema?: ExternalPluginAdapterSchema;
  extraAccounts?: Array<ExtraAccount>;
  dataAuthority?: PluginAuthority;
};

export type LinkedLifecycleHookUpdateInfoArgs = Omit<
  BaseLinkedLifecycleHookUpdateInfoArgs,
  'lifecycleChecks' | 'extraAccounts' | 'schema'
> & {
  key: ExternalPluginAdapterKey;
  lifecycleChecks?: LifecycleChecks;
  extraAccounts?: Array<ExtraAccount>;
  schema?: ExternalPluginAdapterSchema;
};

export function linkedLifecycleHookInitInfoArgsToBase(
  l: LinkedLifecycleHookInitInfoArgs
): BaseLinkedLifecycleHookInitInfoArgs {
  return {
    extraAccounts: l.extraAccounts
      ? l.extraAccounts.map(extraAccountToBase)
      : null,
    hookedProgram: l.hookedProgram,
    initPluginAuthority: l.initPluginAuthority
      ? pluginAuthorityToBase(l.initPluginAuthority)
      : null,
    lifecycleChecks: lifecycleChecksToBase(l.lifecycleChecks),
    schema: l.schema ? l.schema : null,
    dataAuthority: l.dataAuthority
      ? pluginAuthorityToBase(l.dataAuthority)
      : null,
  };
}

export function linkedLifecycleHookUpdateInfoArgsToBase(
  l: LinkedLifecycleHookUpdateInfoArgs
): BaseLinkedLifecycleHookUpdateInfoArgs {
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

export function linkedLifecycleHookFromBase(
  s: BaseLinkedLifecycleHook,
  _r: ExternalRegistryRecord,
  _account: Uint8Array
): LinkedLifecycleHook {
  return {
    ...s,
    extraAccounts:
      s.extraAccounts.__option === 'Some'
        ? s.extraAccounts.value.map(extraAccountFromBase)
        : undefined,
    dataAuthority:
      s.dataAuthority.__option === 'Some'
        ? pluginAuthorityFromBase(s.dataAuthority.value)
        : undefined,
  };
}

export const linkedLifecycleHookManifest: ExternalPluginAdapterManifest<
  LinkedLifecycleHook,
  BaseLinkedLifecycleHook,
  LinkedLifecycleHookInitInfoArgs,
  BaseLinkedLifecycleHookInitInfoArgs,
  LinkedLifecycleHookUpdateInfoArgs,
  BaseLinkedLifecycleHookUpdateInfoArgs
> = {
  type: 'LinkedLifecycleHook',
  fromBase: linkedLifecycleHookFromBase,
  initToBase: linkedLifecycleHookInitInfoArgsToBase,
  updateToBase: linkedLifecycleHookUpdateInfoArgsToBase,
};
