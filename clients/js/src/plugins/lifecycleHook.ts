import { PublicKey } from '@metaplex-foundation/umi';
import {
  ExtraAccount,
  extraAccountFromBase,
  extraAccountToBase,
} from './extraAccount';
import {
  BaseLifecycleHook,
  BaseLifecycleHookInitInfoArgs,
  BaseLifecycleHookUpdateInfoArgs,
  ExternalPluginSchema,
  ExternalRegistryRecord,
} from '../generated';
import { LifecycleChecks, lifecycleChecksToBase } from './lifecycleChecks';
import {
  PluginAuthority,
  pluginAuthorityFromBase,
  pluginAuthorityToBase,
} from './pluginAuthority';
import { BaseExternalPlugin } from './externalPlugins';
import { ExternalPluginManifest } from './externalPluginManifest';
import { ExternalPluginKey } from './externalPluginKey';
import { parseExternalPluginData } from './lib';

export type LifecycleHook = Omit<
  BaseLifecycleHook,
  'extraAccounts' | 'dataAuthority'
> & {
  extraAccounts?: Array<ExtraAccount>;
  dataAuthority?: PluginAuthority;
  data?: any;
};

export type LifecycleHookPlugin = BaseExternalPlugin &
  LifecycleHook & {
    type: 'LifecycleHook';
    hookedProgram: PublicKey;
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
  schema?: ExternalPluginSchema;
  extraAccounts?: Array<ExtraAccount>;
  dataAuthority?: PluginAuthority;
};

export type LifecycleHookUpdateInfoArgs = Omit<
  BaseLifecycleHookUpdateInfoArgs,
  'lifecycleChecks' | 'extraAccounts' | 'schema'
> & {
  key: ExternalPluginKey;
  lifecycleChecks?: LifecycleChecks;
  extraAccounts?: Array<ExtraAccount>;
  schema?: ExternalPluginSchema;
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
    schema: l.schema ? l.schema : null,
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
    schema: l.schema ? l.schema : null,
    // TODO update dataAuthority?
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
    data: parseExternalPluginData(s, r, account),
    dataAuthority:
      s.dataAuthority.__option === 'Some'
        ? pluginAuthorityFromBase(s.dataAuthority.value)
        : undefined,
  };
}

export const lifecycleHookManifest: ExternalPluginManifest<
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
