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
} from '../generated';
import { LifecycleChecks, lifecycleChecksToBase } from './lifecycleChecks';
import { PluginAuthority, pluginAuthorityToBase } from './pluginAuthority';
import { BaseExternalPlugin, parseExternalPluginData } from './externalPlugins';
import { ExternalPluginManifest } from './externalPluginManifest';
import { ExternalPluginKey } from './externalPluginKey';

export type LifecycleHook = Omit<BaseLifecycleHook, 'extraAccounts'> & {
  extraAccounts?: Array<ExtraAccount>;
  data?: any;
};

export type LifecycleHookPlugin = BaseExternalPlugin &
  LifecycleHook & {
    type: 'LifecycleHook';
    hookedProgram: PublicKey;
  };

export type LifecycleHookInitInfoArgs = Omit<
  BaseLifecycleHookInitInfoArgs,
  'initPluginAuthority' | 'lifecycleChecks' | 'schema'
> & {
  type: 'LifecycleHook';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks?: LifecycleChecks;
  schema?: ExternalPluginSchema;
  extraAccounts?: Array<ExtraAccount>;
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
    lifecycleChecks: l.lifecycleChecks
      ? lifecycleChecksToBase(l.lifecycleChecks)
      : null,
    schema: l.schema ? l.schema : null,
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
  };
}

export function lifecycleHookFromBase(
  s: BaseLifecycleHook,
  account: Uint8Array
): LifecycleHook {
  return {
    ...s,
    dataOffset: s.dataOffset,
    dataLen: s.dataLen,
    extraAccounts:
      s.extraAccounts.__option === 'Some'
        ? s.extraAccounts.value.map(extraAccountFromBase)
        : undefined,
    data: parseExternalPluginData(s, account),
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
