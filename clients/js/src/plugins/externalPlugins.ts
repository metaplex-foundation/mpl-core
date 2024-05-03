import {
  AccountMeta,
  Context,
  PublicKey,
  Option,
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
  BaseExternalPluginInitInfoArgs,
  BaseExternalPluginKey,
  BaseExternalPluginUpdateInfoArgs,
  ExternalPluginSchema,
  ExternalRegistryRecord,
  getExternalPluginSerializer,
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

export type ExternalPluginTypeString = BaseExternalPluginKey['__kind'];

export type BaseExternalPlugin = BasePlugin & LifecycleChecksContainer;

export type ExternalPluginsList = {
  oracles?: OraclePlugin[];
  dataStores?: DataStorePlugin[];
  lifecycleHooks?: LifecycleHookPlugin[];
};

export type ExternalPluginInitInfoArgs =
  | ({
      type: 'Oracle';
    } & OracleInitInfoArgs)
  | ({
      type: 'LifecycleHook';
    } & LifecycleHookInitInfoArgs)
  | ({
      type: 'DataStore';
    } & DataStoreInitInfoArgs);

export type ExternalPluginUpdateInfoArgs =
  | ({
      type: 'Oracle';
    } & OracleUpdateInfoArgs)
  | ({
      type: 'LifecycleHook';
    } & LifecycleHookUpdateInfoArgs)
  | ({
      type: 'DataStore';
    } & DataStoreUpdateInfoArgs);

export const externalPluginManifests = {
  Oracle: oracleManifest,
  DataStore: dataStoreManifest,
  LifecycleHook: lifecycleHookManifest,
};

export type ExternalPluginData = {
  dataLen: bigint;
  dataOffset: bigint;
};

export function externalRegistryRecordsToExternalPluginList(
  records: ExternalRegistryRecord[],
  accountData: Uint8Array
): ExternalPluginsList {
  const result: ExternalPluginsList = {};

  records.forEach((record) => {
    const deserializedPlugin = getExternalPluginSerializer().deserialize(
      accountData,
      Number(record.offset)
    )[0];

    const mappedPlugin: BaseExternalPlugin = {
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

export const isExternalPluginType = (plugin: { type: string }) => {
  if (
    plugin.type === 'Oracle' ||
    plugin.type === 'LifecycleHook' ||
    plugin.type === 'DataStore'
  ) {
    return true;
  }
  return false;
};

export function createExternalPluginInitInfo({
  type,
  ...args
}: ExternalPluginInitInfoArgs): BaseExternalPluginInitInfoArgs {
  const manifest = externalPluginManifests[type];
  return {
    __kind: type,
    fields: [manifest.initToBase(args as any)] as any,
  };
}

export function createExternalPluginUpdateInfo({
  type,
  ...args
}: ExternalPluginUpdateInfoArgs): BaseExternalPluginUpdateInfoArgs {
  const manifest = externalPluginManifests[type];
  return {
    __kind: type,
    fields: [manifest.updateToBase(args as any)] as any,
  };
}

export const findExtraAccounts = (
  context: Pick<Context, 'eddsa'>,
  lifecycle: LifecycleEvent,
  externalPlugins: ExternalPluginsList,
  inputs: {
    asset: PublicKey;
    collection?: PublicKey;
    owner: PublicKey;
    recipient?: PublicKey;
  }
): AccountMeta[] => {
  const accounts: AccountMeta[] = [];

  externalPlugins.oracles?.forEach((oracle) => {
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

  externalPlugins.lifecycleHooks?.forEach((hook) => {
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

export function parseExternalPluginData(
  plugin: {
    schema: ExternalPluginSchema;
  },
  record: {
    dataLen: Option<bigint | number>;
    dataOffset: Option<bigint | number>;
  },
  account: Uint8Array
): any {
  let data;
  const dataSlice = account.slice(
    Number(record.dataOffset),
    Number(record.dataOffset) + Number(record.dataLen)
  );

  if (plugin.schema === ExternalPluginSchema.Binary) {
    data = dataSlice;
  } else if (plugin.schema === ExternalPluginSchema.Json) {
    data = JSON.parse(new TextDecoder().decode(dataSlice));
  } else if (plugin.schema === ExternalPluginSchema.MsgPack) {
    // eslint-disable-next-line no-console
    console.warn(
      'MsgPack schema currently not supported, falling back to binary'
    );
    data = dataSlice;
  }
  return data;
}
