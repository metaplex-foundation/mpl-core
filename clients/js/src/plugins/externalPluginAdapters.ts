import {
  AccountMeta,
  Context,
  isSome,
  PublicKey,
} from '@metaplex-foundation/umi';
import {
  lifecycleHookFromBase,
  LifecycleHookInitInfoArgs,
  lifecycleHookManifest,
  LifecycleHookPlugin,
  LifecycleHookUpdateInfoArgs,
  pluginAuthorityFromBase,
} from '.';
import {
  BaseExternalPluginAdapterInitInfoArgs,
  BaseExternalPluginAdapterKey,
  BaseExternalPluginAdapterUpdateInfoArgs,
  ExternalRegistryRecord,
  getExternalPluginAdapterSerializer,
} from '../generated';

import {
  appDataFromBase,
  AppDataInitInfoArgs,
  appDataManifest,
  AppDataPlugin,
  AppDataUpdateInfoArgs,
} from './appData';
import {
  LifecycleChecksContainer,
  lifecycleChecksFromBase,
  LifecycleEvent,
} from './lifecycleChecks';
import {
  oracleFromBase,
  OracleInitInfoArgs,
  oracleManifest,
  OraclePlugin,
  OracleUpdateInfoArgs,
} from './oracle';
import { BasePlugin } from './types';
import { extraAccountToAccountMeta } from './extraAccount';
import {
  linkedAppDataFromBase,
  LinkedAppDataInitInfoArgs,
  linkedAppDataManifest,
  LinkedAppDataPlugin,
  LinkedAppDataUpdateInfoArgs,
} from './linkedAppData';
import {
  dataSectionFromBase,
  dataSectionManifest,
  DataSectionPlugin,
} from './dataSection';
import {
  LinkedLifecycleHookInitInfoArgs,
  LinkedLifecycleHookPlugin,
  LinkedLifecycleHookUpdateInfoArgs,
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
    const deserializedPlugin = getExternalPluginAdapterSerializer().deserialize(
      accountData,
      Number(record.offset)
    )[0];

    const mappedPlugin: BaseExternalPluginAdapter = {
      lifecycleChecks:
        record.lifecycleChecks.__option === 'Some'
          ? lifecycleChecksFromBase(record.lifecycleChecks.value)
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
        dataOffset: isSome(record.dataOffset)
          ? record.dataOffset.value
          : undefined,
        dataLen: isSome(record.dataLen) ? record.dataLen.value : undefined,
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
        dataOffset: isSome(record.dataOffset)
          ? record.dataOffset.value
          : undefined,
        dataLen: isSome(record.dataLen) ? record.dataLen.value : undefined,
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
        dataOffset: isSome(record.dataOffset)
          ? record.dataOffset.value
          : undefined,
        dataLen: isSome(record.dataLen) ? record.dataLen.value : undefined,
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
  return {
    __kind: type,
    fields: [manifest.initToBase(args as any)] as any,
  };
}

export function createExternalPluginAdapterUpdateInfo({
  type,
  ...args
}: ExternalPluginAdapterUpdateInfoArgs): BaseExternalPluginAdapterUpdateInfoArgs {
  const manifest = externalPluginAdapterManifests[type];
  return {
    __kind: type,
    fields: [manifest.updateToBase(args as any)] as any,
  };
}

export const findExtraAccounts = (
  context: Pick<Context, 'eddsa'>,
  lifecycle: LifecycleEvent,
  externalPluginAdapters: ExternalPluginAdaptersList,
  inputs: {
    asset: PublicKey;
    collection?: PublicKey;
    owner: PublicKey;
    recipient?: PublicKey;
  }
): AccountMeta[] => {
  const accounts: AccountMeta[] = [];

  externalPluginAdapters.oracles?.forEach((oracle) => {
    if (oracle.lifecycleChecks?.[lifecycle]) {
      if (oracle.baseAddressConfig) {
        accounts.push(
          extraAccountToAccountMeta(context, oracle.baseAddressConfig, {
            ...inputs,
            program: oracle.baseAddress,
          })
        );
      } else {
        accounts.push({
          pubkey: oracle.baseAddress,
          isSigner: false,
          isWritable: false,
        });
      }
    }
  });

  externalPluginAdapters.lifecycleHooks?.forEach((hook) => {
    if (hook.lifecycleChecks?.[lifecycle]) {
      accounts.push({
        pubkey: hook.hookedProgram,
        isSigner: false,
        isWritable: false,
      });

      hook.extraAccounts?.forEach((extra) => {
        accounts.push(
          extraAccountToAccountMeta(context, extra, {
            ...inputs,
            program: hook.hookedProgram,
          })
        );
      });
    }
  });

  externalPluginAdapters.linkedLifecycleHooks?.forEach((hook) => {
    if (hook.lifecycleChecks?.[lifecycle]) {
      accounts.push({
        pubkey: hook.hookedProgram,
        isSigner: false,
        isWritable: false,
      });

      hook.extraAccounts?.forEach((extra) => {
        accounts.push(
          extraAccountToAccountMeta(context, extra, {
            ...inputs,
            program: hook.hookedProgram,
          })
        );
      });
    }
  });

  return accounts;
};
