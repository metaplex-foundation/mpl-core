import { AccountMeta, Context, PublicKey } from '@metaplex-foundation/umi';
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
  dataStoreFromBase,
  DataStoreInitInfoArgs,
  dataStoreManifest,
  DataStorePlugin,
  DataStoreUpdateInfoArgs,
} from './dataStore';
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

export type ExternalPluginAdapterTypeString =
  BaseExternalPluginAdapterKey['__kind'];

export type BaseExternalPluginAdapter = BasePlugin & LifecycleChecksContainer;

export type ExternalPluginAdaptersList = {
  oracles?: OraclePlugin[];
  dataStores?: DataStorePlugin[];
  lifecycleHooks?: LifecycleHookPlugin[];
};

export type ExternalPluginAdapterInitInfoArgs =
  | ({
      type: 'Oracle';
    } & OracleInitInfoArgs)
  | ({
      type: 'LifecycleHook';
    } & LifecycleHookInitInfoArgs)
  | ({
      type: 'DataStore';
    } & DataStoreInitInfoArgs);

export type ExternalPluginAdapterUpdateInfoArgs =
  | ({
      type: 'Oracle';
    } & OracleUpdateInfoArgs)
  | ({
      type: 'LifecycleHook';
    } & LifecycleHookUpdateInfoArgs)
  | ({
      type: 'DataStore';
    } & DataStoreUpdateInfoArgs);

export const externalPluginAdapterManifests = {
  Oracle: oracleManifest,
  DataStore: dataStoreManifest,
  LifecycleHook: lifecycleHookManifest,
};

export type ExternalPluginAdapterData = {
  dataLen: bigint;
  dataOffset: bigint;
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

    if (deserializedPlugin.__kind === 'Oracle') {
      if (!result.oracles) {
        result.oracles = [];
      }

      result.oracles.push({
        type: 'Oracle',
        ...mappedPlugin,
        ...oracleFromBase(deserializedPlugin.fields[0], record, accountData),
      });
    } else if (deserializedPlugin.__kind === 'DataStore') {
      if (!result.dataStores) {
        result.dataStores = [];
      }
      result.dataStores.push({
        type: 'DataStore',
        ...mappedPlugin,
        ...dataStoreFromBase(deserializedPlugin.fields[0], record, accountData),
      });
    } else if (deserializedPlugin.__kind === 'LifecycleHook') {
      if (!result.lifecycleHooks) {
        result.lifecycleHooks = [];
      }
      result.lifecycleHooks.push({
        type: 'LifecycleHook',
        ...mappedPlugin,
        ...lifecycleHookFromBase(
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
    plugin.type === 'Oracle' ||
    plugin.type === 'LifecycleHook' ||
    plugin.type === 'DataStore'
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

  return accounts;
};
