import {
  LifecycleHookInitInfoArgs,
  lifecycleHookManifest,
  LifecycleHookPlugin,
  LifecycleHookUpdateInfoArgs,
  mapPluginFields,
  PluginAuthority,
} from '.';
import { mapPluginAuthority } from '../authority';
import {
  BaseExternalPluginInitInfoArgs,
  BaseExternalPluginKey,
  BaseExternalPluginUpdateInfoArgs,
  ExternalPlugin,
  ExternalRegistryRecord,
  getExternalPluginSerializer,
} from '../generated';
import { toWords } from '../utils';
import {
  DataStoreInitInfoArgs,
  dataStoreManifest,
  DataStorePlugin,
  DataStoreUpdateInfoArgs,
} from './dataStore';
import { LifecycleChecksContainer } from './lifecycleChecks';
import {
  OracleInitInfoArgs,
  oracleManifest,
  OraclePlugin,
  OracleUpdateInfoArgs,
} from './oracle';
import { BasePlugin } from './types';

export type ExternalPluginType = BaseExternalPluginKey['__kind'];

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

export function mapExternalPlugin({
  plugin: plug,
  authority,
  offset,
}: {
  plugin: ExternalPlugin;
  authority: PluginAuthority;
  offset: bigint;
}): ExternalPluginsList {
  const pluginKey = toWords(plug.__kind)
    .toLowerCase()
    .split(' ')
    .reduce((s, c) => s + (c.charAt(0).toUpperCase() + c.slice(1)));

  return {
    [pluginKey]: {
      authority,
      offset,
      ...('fields' in plug ? mapPluginFields(plug.fields) : {}),
    },
  };
}

export function externalRegistryRecordToExternalPluginList(
  records: ExternalRegistryRecord[],
  accountData: Uint8Array
): ExternalPluginsList {
  return records.reduce((acc: ExternalPluginsList, record) => {
    const mappedAuthority = mapPluginAuthority(record.authority);
    const deserializedPlugin = getExternalPluginSerializer().deserialize(
      accountData,
      Number(record.offset)
    )[0];

    acc = {
      ...acc,
      ...mapExternalPlugin({
        plugin: deserializedPlugin,
        authority: mappedAuthority,
        offset: record.offset,
      }),
    };

    return acc;
  }, {});
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
