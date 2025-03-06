import { PublicKey } from '@metaplex-foundation/umi';
import {
  ExtraAccount,
  extraAccountFromBase,
  extraAccountToBase,
} from './extraAccount';
import {
  BaseLinkedLifecycleHook,
  BaseLinkedLifecycleHookInitInfoArgs,
  BaseLinkedLifecycleHookUpdateInfoArgs,
  ExternalPluginAdapterSchema,
  ExternalRegistryRecord,
} from '../generated';
import { LifecycleChecks, lifecycleChecksToBase } from './lifecycleChecks';
import {
  PluginAuthority,
  pluginAuthorityFromBase,
  pluginAuthorityToBase,
} from './pluginAuthority';
import { BaseExternalPluginAdapter } from './externalPluginAdapters';
import { ExternalPluginAdapterManifest } from './externalPluginAdapterManifest';
import { ExternalPluginAdapterKey } from './externalPluginAdapterKey';
// import { parseExternalPluginAdapterData } from './lib';

export type LinkedLifecycleHook = Omit<
  BaseLinkedLifecycleHook,
  'extraAccounts' | 'dataAuthority'
> & {
  extraAccounts?: Array<ExtraAccount>;
  dataAuthority?: PluginAuthority;
  data?: any;
};

export type LinkedLifecycleHookPlugin = BaseExternalPluginAdapter &
  LinkedLifecycleHook & {
    type: 'LinkedLifecycleHook';
    hookedProgram: PublicKey;
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
  r: ExternalRegistryRecord,
  account: Uint8Array
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
