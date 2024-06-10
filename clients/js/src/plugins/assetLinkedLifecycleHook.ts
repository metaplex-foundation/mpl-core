import { PublicKey } from '@metaplex-foundation/umi';
import {
  ExtraAccount,
  extraAccountFromBase,
  extraAccountToBase,
} from './extraAccount';
import {
  BaseAssetLinkedLifecycleHook,
  BaseAssetLinkedLifecycleHookInitInfoArgs,
  BaseAssetLinkedLifecycleHookUpdateInfoArgs,
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

export type AssetLinkedLifecycleHook = Omit<
  BaseAssetLinkedLifecycleHook,
  'extraAccounts' | 'dataAuthority'
> & {
  extraAccounts?: Array<ExtraAccount>;
  dataAuthority?: PluginAuthority;
  data?: any;
};

export type AssetLinkedLifecycleHookPlugin = BaseExternalPluginAdapter &
  AssetLinkedLifecycleHook & {
    type: 'AssetLinkedLifecycleHook';
    hookedProgram: PublicKey;
  };

export type AssetLinkedLifecycleHookInitInfoArgs = Omit<
  BaseAssetLinkedLifecycleHookInitInfoArgs,
  | 'initPluginAuthority'
  | 'lifecycleChecks'
  | 'schema'
  | 'extraAccounts'
  | 'dataAuthority'
> & {
  type: 'AssetLinkedLifecycleHook';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks: LifecycleChecks;
  schema?: ExternalPluginAdapterSchema;
  extraAccounts?: Array<ExtraAccount>;
  dataAuthority?: PluginAuthority;
};

export type AssetLinkedLifecycleHookUpdateInfoArgs = Omit<
  BaseAssetLinkedLifecycleHookUpdateInfoArgs,
  'lifecycleChecks' | 'extraAccounts' | 'schema'
> & {
  key: ExternalPluginAdapterKey;
  lifecycleChecks?: LifecycleChecks;
  extraAccounts?: Array<ExtraAccount>;
  schema?: ExternalPluginAdapterSchema;
};

export function assetLinkedLifecycleHookInitInfoArgsToBase(
  l: AssetLinkedLifecycleHookInitInfoArgs
): BaseAssetLinkedLifecycleHookInitInfoArgs {
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

export function assetLinkedLifecycleHookUpdateInfoArgsToBase(
  l: AssetLinkedLifecycleHookUpdateInfoArgs
): BaseAssetLinkedLifecycleHookUpdateInfoArgs {
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

export function assetLinkedLifecycleHookFromBase(
  s: BaseAssetLinkedLifecycleHook,
  r: ExternalRegistryRecord,
  account: Uint8Array
): AssetLinkedLifecycleHook {
  return {
    ...s,
    extraAccounts:
      s.extraAccounts.__option === 'Some'
        ? s.extraAccounts.value.map(extraAccountFromBase)
        : undefined,
    // data: parseExternalPluginAdapterData(s, r, account),
    dataAuthority:
      s.dataAuthority.__option === 'Some'
        ? pluginAuthorityFromBase(s.dataAuthority.value)
        : undefined,
  };
}

export const assetLinkedLifecycleHookManifest: ExternalPluginAdapterManifest<
  AssetLinkedLifecycleHook,
  BaseAssetLinkedLifecycleHook,
  AssetLinkedLifecycleHookInitInfoArgs,
  BaseAssetLinkedLifecycleHookInitInfoArgs,
  AssetLinkedLifecycleHookUpdateInfoArgs,
  BaseAssetLinkedLifecycleHookUpdateInfoArgs
> = {
  type: 'AssetLinkedLifecycleHook',
  fromBase: assetLinkedLifecycleHookFromBase,
  initToBase: assetLinkedLifecycleHookInitInfoArgsToBase,
  updateToBase: assetLinkedLifecycleHookUpdateInfoArgsToBase,
};
