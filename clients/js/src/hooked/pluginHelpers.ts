import {
  Authority,
  Key,
  PluginHeader,
  PluginRegistry,
  PluginType,
  Plugin,
} from '../generated';

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
