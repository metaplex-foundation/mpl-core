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
  BasePluginAdapterInitInfoArgs,
  BasePluginAdapterKey,
  BasePluginAdapterUpdateInfoArgs,
  AdapterRegistryRecord,
  getPluginAdapterSerializer,
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

export type PluginAdapterTypeString = BasePluginAdapterKey['__kind'];

export type BasePluginAdapter = BasePlugin & LifecycleChecksContainer;

export type PluginAdaptersList = {
  oracles?: OraclePlugin[];
  dataStores?: DataStorePlugin[];
  lifecycleHooks?: LifecycleHookPlugin[];
};

export type PluginAdapterInitInfoArgs =
  | ({
      type: 'Oracle';
    } & OracleInitInfoArgs)
  | ({
      type: 'LifecycleHook';
    } & LifecycleHookInitInfoArgs)
  | ({
      type: 'DataStore';
    } & DataStoreInitInfoArgs);

export type PluginAdapterUpdateInfoArgs =
  | ({
      type: 'Oracle';
    } & OracleUpdateInfoArgs)
  | ({
      type: 'LifecycleHook';
    } & LifecycleHookUpdateInfoArgs)
  | ({
      type: 'DataStore';
    } & DataStoreUpdateInfoArgs);

export const pluginAdapterManifests = {
  Oracle: oracleManifest,
  DataStore: dataStoreManifest,
  LifecycleHook: lifecycleHookManifest,
};

export type PluginAdapterData = {
  dataLen: bigint;
  dataOffset: bigint;
};

export function adapterRegistryRecordsToPluginAdapterList(
  records: AdapterRegistryRecord[],
  accountData: Uint8Array
): PluginAdaptersList {
  const result: PluginAdaptersList = {};

  records.forEach((record) => {
    const deserializedPlugin = getPluginAdapterSerializer().deserialize(
      accountData,
      Number(record.offset)
    )[0];

    const mappedPlugin: BasePluginAdapter = {
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

export const isPluginAdapterType = (plugin: { type: string }) => {
  if (
    plugin.type === 'Oracle' ||
    plugin.type === 'LifecycleHook' ||
    plugin.type === 'DataStore'
  ) {
    return true;
  }
  return false;
};

export function createPluginAdapterInitInfo({
  type,
  ...args
}: PluginAdapterInitInfoArgs): BasePluginAdapterInitInfoArgs {
  const manifest = pluginAdapterManifests[type];
  return {
    __kind: type,
    fields: [manifest.initToBase(args as any)] as any,
  };
}

export function createPluginAdapterUpdateInfo({
  type,
  ...args
}: PluginAdapterUpdateInfoArgs): BasePluginAdapterUpdateInfoArgs {
  const manifest = pluginAdapterManifests[type];
  return {
    __kind: type,
    fields: [manifest.updateToBase(args as any)] as any,
  };
}

export const findExtraAccounts = (
  context: Pick<Context, 'eddsa'>,
  lifecycle: LifecycleEvent,
  pluginAdapters: PluginAdaptersList,
  inputs: {
    asset: PublicKey;
    collection?: PublicKey;
    owner: PublicKey;
    recipient?: PublicKey;
  }
): AccountMeta[] => {
  const accounts: AccountMeta[] = [];

  pluginAdapters.oracles?.forEach((oracle) => {
    if (oracle.lifecycleChecks?.[lifecycle]) {
      if (oracle.pda) {
        accounts.push(
          extraAccountToAccountMeta(context, oracle.pda, {
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

  pluginAdapters.lifecycleHooks?.forEach((hook) => {
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
