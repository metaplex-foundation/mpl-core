import {
  lifecycleHookFromBase,
  type LifecycleHookInitInfoArgs,
  lifecycleHookManifest,
  type LifecycleHookPlugin,
  type LifecycleHookUpdateInfoArgs,
} from './lifecycleHook';
import { pluginAuthorityFromBase } from './pluginAuthority';
import {
  type BaseExternalPluginAdapterInitInfoArgs,
  type BaseExternalPluginAdapterKey,
  type BaseExternalPluginAdapterUpdateInfoArgs,
  type ExternalRegistryRecord,
  type ExternalCheckResult,
  type HookableLifecycleEvent,
  getExternalPluginAdapterDecoder,
} from '../generated';

import {
  appDataFromBase,
  type AppDataInitInfoArgs,
  appDataManifest,
  type AppDataPlugin,
  type AppDataUpdateInfoArgs,
} from './appData';
import {
  type LifecycleChecksContainer,
  lifecycleChecksFromBase,
} from './lifecycleChecks';
import {
  oracleFromBase,
  type OracleInitInfoArgs,
  oracleManifest,
  type OraclePlugin,
  type OracleUpdateInfoArgs,
} from './oracle';
import { type BasePlugin } from './types';
import {
  linkedAppDataFromBase,
  type LinkedAppDataInitInfoArgs,
  linkedAppDataManifest,
  type LinkedAppDataPlugin,
  type LinkedAppDataUpdateInfoArgs,
} from './linkedAppData';
import {
  dataSectionFromBase,
  dataSectionManifest,
  type DataSectionPlugin,
} from './dataSection';
import {
  type LinkedLifecycleHookInitInfoArgs,
  type LinkedLifecycleHookPlugin,
  type LinkedLifecycleHookUpdateInfoArgs,
  linkedLifecycleHookFromBase,
  linkedLifecycleHookManifest,
} from './linkedLifecycleHook';

export type ExternalPluginAdapterTypeString =
  BaseExternalPluginAdapterKey['__kind'];

export type BaseExternalPluginAdapter = BasePlugin &
  ExternalPluginAdapterData &
  LifecycleChecksContainer;

export type ExternalPluginAdapters =
  | LifecycleHookPlugin
  | OraclePlugin
  | AppDataPlugin
  | LinkedLifecycleHookPlugin
  | LinkedAppDataPlugin
  | DataSectionPlugin;

export type ExternalPluginAdaptersList = {
  lifecycleHooks?: LifecycleHookPlugin[];
  oracles?: OraclePlugin[];
  appDatas?: AppDataPlugin[];
  linkedLifecycleHooks?: LinkedLifecycleHookPlugin[];
  linkedAppDatas?: LinkedAppDataPlugin[];
  dataSections?: DataSectionPlugin[];
};

export type ExternalPluginAdapterInitInfoArgs =
  | ({
      type: 'LifecycleHook';
    } & LifecycleHookInitInfoArgs)
  | ({
      type: 'Oracle';
    } & OracleInitInfoArgs)
  | ({
      type: 'AppData';
    } & AppDataInitInfoArgs)
  | ({
      type: 'LinkedLifecycleHook';
    } & LinkedLifecycleHookInitInfoArgs)
  | ({
      type: 'LinkedAppData';
    } & LinkedAppDataInitInfoArgs)
  | ({
      type: 'DataSection';
    } & AppDataInitInfoArgs);

export type ExternalPluginAdapterUpdateInfoArgs =
  | ({
      type: 'LifecycleHook';
    } & LifecycleHookUpdateInfoArgs)
  | ({
      type: 'Oracle';
    } & OracleUpdateInfoArgs)
  | ({
      type: 'AppData';
    } & AppDataUpdateInfoArgs)
  | ({
      type: 'LinkedLifecycleHook';
    } & LinkedLifecycleHookUpdateInfoArgs)
  | ({
      type: 'LinkedAppData';
    } & LinkedAppDataUpdateInfoArgs);

export const externalPluginAdapterManifests = {
  LifecycleHook: lifecycleHookManifest,
  Oracle: oracleManifest,
  AppData: appDataManifest,
  LinkedLifecycleHook: linkedLifecycleHookManifest,
  LinkedAppData: linkedAppDataManifest,
  DataSection: dataSectionManifest,
};

export type ExternalPluginAdapterData = {
  dataLen?: bigint;
  dataOffset?: bigint;
};

export function externalRegistryRecordsToExternalPluginAdapterList(
  records: ExternalRegistryRecord[],
  accountData: Uint8Array
): ExternalPluginAdaptersList {
  const result: ExternalPluginAdaptersList = {};

  records.forEach((record) => {
    const deserializedPlugin = getExternalPluginAdapterDecoder().read(
      accountData,
      Number(record.offset)
    )[0];

    const mappedPlugin: BaseExternalPluginAdapter = {
      lifecycleChecks:
        record.lifecycleChecks.__option === 'Some'
          ? lifecycleChecksFromBase(record.lifecycleChecks.value as [HookableLifecycleEvent, ExternalCheckResult][])
          : undefined,
      authority: pluginAuthorityFromBase(record.authority),
      offset: record.offset,
    };

    if (deserializedPlugin.__kind === 'LifecycleHook') {
      if (!result.lifecycleHooks) {
        result.lifecycleHooks = [];
      }
      result.lifecycleHooks.push({
        type: 'LifecycleHook',
        dataOffset: record.dataOffset.__option === 'Some'
          ? record.dataOffset.value
          : undefined,
        dataLen: record.dataLen.__option === 'Some' ? record.dataLen.value : undefined,
        ...mappedPlugin,
        ...lifecycleHookFromBase(
          deserializedPlugin.fields[0],
          record,
          accountData
        ),
      });
    } else if (deserializedPlugin.__kind === 'AppData') {
      if (!result.appDatas) {
        result.appDatas = [];
      }
      result.appDatas.push({
        type: 'AppData',
        dataOffset: record.dataOffset.__option === 'Some'
          ? record.dataOffset.value
          : undefined,
        dataLen: record.dataLen.__option === 'Some' ? record.dataLen.value : undefined,
        ...mappedPlugin,
        ...appDataFromBase(deserializedPlugin.fields[0], record, accountData),
      });
    } else if (deserializedPlugin.__kind === 'Oracle') {
      if (!result.oracles) {
        result.oracles = [];
      }

      result.oracles.push({
        type: 'Oracle',
        ...mappedPlugin,
        ...oracleFromBase(deserializedPlugin.fields[0], record, accountData),
      });
    } else if (deserializedPlugin.__kind === 'LinkedLifecycleHook') {
      if (!result.linkedLifecycleHooks) {
        result.linkedLifecycleHooks = [];
      }
      result.linkedLifecycleHooks.push({
        type: 'LinkedLifecycleHook',
        ...mappedPlugin,
        ...linkedLifecycleHookFromBase(
          deserializedPlugin.fields[0],
          record,
          accountData
        ),
      });
    } else if (deserializedPlugin.__kind === 'LinkedAppData') {
      if (!result.linkedAppDatas) {
        result.linkedAppDatas = [];
      }
      result.linkedAppDatas.push({
        type: 'LinkedAppData',
        ...mappedPlugin,
        ...linkedAppDataFromBase(
          deserializedPlugin.fields[0],
          record,
          accountData
        ),
      });
    } else if (deserializedPlugin.__kind === 'DataSection') {
      if (!result.dataSections) {
        result.dataSections = [];
      }
      result.dataSections.push({
        type: 'DataSection',
        dataOffset: record.dataOffset.__option === 'Some'
          ? record.dataOffset.value
          : undefined,
        dataLen: record.dataLen.__option === 'Some' ? record.dataLen.value : undefined,
        ...mappedPlugin,
        ...dataSectionFromBase(
          deserializedPlugin.fields[0],
          record,
          accountData
        ),
      });
    }
  });

  return result;
}

export const isExternalPluginAdapterType = (plugin: { type: string }) => {
  if (
    plugin.type === 'LifecycleHook' ||
    plugin.type === 'Oracle' ||
    plugin.type === 'AppData' ||
    plugin.type === 'LinkedLifecycleHook' ||
    plugin.type === 'DataSection' ||
    plugin.type === 'LinkedAppData'
  ) {
    return true;
  }
  return false;
};

export function createExternalPluginAdapterInitInfo({
  type,
  ...args
}: ExternalPluginAdapterInitInfoArgs): BaseExternalPluginAdapterInitInfoArgs {
  const manifest = externalPluginAdapterManifests[type];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return {
    __kind: type,
    fields: [manifest.initToBase(args as any)],
  } as BaseExternalPluginAdapterInitInfoArgs;
}

export function createExternalPluginAdapterUpdateInfo({
  type,
  ...args
}: ExternalPluginAdapterUpdateInfoArgs): BaseExternalPluginAdapterUpdateInfoArgs {
  const manifest = externalPluginAdapterManifests[type as keyof typeof externalPluginAdapterManifests];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return {
    __kind: type,
    fields: [manifest.updateToBase(args as any)],
  } as unknown as BaseExternalPluginAdapterUpdateInfoArgs;
}
