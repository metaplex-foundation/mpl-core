import {
  Authority,
  Key,
  PluginHeader,
  PluginRegistry,
  PluginType,
  Plugin,
  getPluginSerializer,
  RegistryRecord,
} from '../generated';
import { BaseAuthorities, PluginsList } from './types';
import { mapAuthorities } from './authorityHelpers';
import { toWords } from './utils';

export function formPluginHeader(
  pluginRegistryOffset: bigint
): Omit<PluginHeader, 'publicKey' | 'header'> {
  return {
    key: Key.PluginHeader,
    pluginRegistryOffset,
  };
}

export function formPluginRegistry({
  pluginType,
  offset,
  authorities,
}: {
  pluginType: PluginType;
  offset: bigint;
  authorities: Authority[];
}): Omit<PluginRegistry, 'publicKey' | 'header' | 'externalPlugins'> {
  return {
    key: Key.PluginRegistry,
    registry: [
      {
        pluginType,
        offset,
        authorities,
      },
    ],
  };
}

export function formPluginWithAuthorities({
  authorities,
  plugin,
}: {
  authorities: Authority[];
  plugin: Plugin;
}) {
  return {
    authorities,
    plugin,
  };
}

export function mapPluginFields(fields: Array<Record<string, any>>) {
  return fields.reduce((acc2, field) => ({ ...acc2, ...field }), {});
}

export function mapPlugin(
  plugin: Plugin,
  authorities: BaseAuthorities
): PluginsList {
  const pluginKey = toWords(plugin.__kind)
    .toLowerCase()
    .split(' ')
    .reduce((s, c) => s + (c.charAt(0).toUpperCase() + c.slice(1)));

  return {
    [pluginKey]: {
      authorities,
      ...('fields' in plugin ? mapPluginFields(plugin.fields) : {}),
    },
  };
}

export function registryRecordsToPluginsList(
  registryRecords: RegistryRecord[],
  accountData: Uint8Array
): PluginsList {
  return registryRecords.reduce((acc, record) => {
    const mappedAuthorities = mapAuthorities(record.authorities);
    const deserializedPlugin = getPluginSerializer().deserialize(
      accountData,
      Number(record.offset)
    )[0];

    acc = {
      ...acc,
      ...mapPlugin(deserializedPlugin, mappedAuthorities),
    };

    return acc;
  }, {});
}
