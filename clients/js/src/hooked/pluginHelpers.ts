import {
  Key,
  PluginHeader,
  Plugin,
  getPluginSerializer,
  RegistryRecord,
} from '../generated';
import { BaseAuthority, PluginsList } from './types';
import { mapAuthority } from './authorityHelpers';
import { toWords } from './utils';

export function formPluginHeader(
  pluginRegistryOffset: bigint
): Omit<PluginHeader, 'publicKey' | 'header'> {
  return {
    key: Key.PluginHeader,
    pluginRegistryOffset,
  };
}

export function mapPluginFields(fields: Array<Record<string, any>>) {
  return fields.reduce((acc2, field) => ({ ...acc2, ...field }), {});
}

export function mapPlugin({
  plugin,
  authority,
  offset,
}: {
  plugin: Exclude<Plugin, { __kind: 'Reserved' }>;
  authority: BaseAuthority;
  offset: bigint;
}): PluginsList {
  const pluginKey = toWords(plugin.__kind)
    .toLowerCase()
    .split(' ')
    .reduce((s, c) => s + (c.charAt(0).toUpperCase() + c.slice(1)));

  return {
    [pluginKey]: {
      authority,
      offset,
      ...('fields' in plugin ? mapPluginFields(plugin.fields) : {}),
    },
  };
}

export function registryRecordsToPluginsList(
  registryRecords: RegistryRecord[],
  accountData: Uint8Array
) {
  return registryRecords.reduce((acc: PluginsList, record) => {
    const mappedAuthority = mapAuthority(record.authority);
    const deserializedPlugin = getPluginSerializer().deserialize(
      accountData,
      Number(record.offset)
    )[0];

    if (deserializedPlugin.__kind === 'Reserved') return acc;

    acc = {
      ...acc,
      ...mapPlugin({
        plugin: deserializedPlugin,
        authority: mappedAuthority,
        offset: record.offset,
      }),
    };

    return acc;
  }, {});
}
