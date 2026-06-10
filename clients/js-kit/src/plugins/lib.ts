import { type Option } from '@solana/codecs';

import {
  type Plugin as BasePlugin,
  type BasePluginAuthority,
  ExternalPluginAdapterSchema,
  getPluginDecoder,
  Key,
  type PluginAuthorityPair,
  type PluginHeaderV1,
  type PluginType,
  type RegistryRecord,
} from '../generated';

import { toWords } from '../utils';
import { masterEditionFromBase, masterEditionToBase } from './masterEdition';
import {
  type PluginAuthority,
  pluginAuthorityFromBase,
  pluginAuthorityToBase,
} from './pluginAuthority';
import { royaltiesFromBase, royaltiesToBase } from './royalties';
import {
  type AssetAllPluginArgsV2,
  type AssetPluginAuthorityPairArgsV2,
  type CreatePluginArgs,
  type PluginAuthorityPairHelperArgs,
  type PluginsList,
} from './types';
import { someOrNone } from '../utils';

export function formPluginHeaderV1(
  pluginRegistryOffset: bigint
): Omit<PluginHeaderV1, 'publicKey' | 'header'> {
  return {
    key: Key.PluginHeaderV1,
    pluginRegistryOffset,
  };
}

/**
 * @deprecated Use the new 1.0 sdk instruction helpers like `create` instead of `createV1` which no longer require sub create functions like this.
 * @param args
 * @returns
 */
export function createPlugin(args: CreatePluginArgs): BasePlugin {
  // TODO refactor when there are more required empty fields in plugins
  if (args.type === 'UpdateDelegate') {
    return {
      __kind: args.type,
      fields: [
        (args as { data?: { additionalDelegates: unknown[] } }).data || {
          additionalDelegates: [],
        },
      ],
    } as BasePlugin;
  }
  return {
    __kind: args.type,
    fields: [(args as { data?: unknown }).data || {}],
  } as BasePlugin;
}

/**
 * @deprecated Use the new 1.0 sdk instruction helpers like `create` instead of `createV1` which no longer require sub create functions like this.
 * @param args
 * @returns
 */
export function pluginAuthorityPair(
  args: PluginAuthorityPairHelperArgs
): PluginAuthorityPair {
  const { type, authority, data } = args as {
    type: string;
    authority?: PluginAuthority | BasePluginAuthority;
    data?: unknown;
  };
  // Handle both PluginAuthority (with 'type') and BasePluginAuthority (with '__kind') formats
  let baseAuthority: BasePluginAuthority | undefined;
  if (authority) {
    if ('__kind' in authority) {
      baseAuthority = authority as BasePluginAuthority;
    } else {
      baseAuthority = pluginAuthorityToBase(authority as PluginAuthority);
    }
  }
  return {
    plugin: createPlugin({
      type,
      data,
    } as CreatePluginArgs),
    authority: baseAuthority ? someOrNone(baseAuthority) : { __option: 'None' },
  };
}

export function createPluginV2(args: AssetAllPluginArgsV2): BasePlugin {
  // TODO refactor when there are more required empty fields in plugins
  const { type } = args;
  if (type === 'UpdateDelegate') {
    return {
      __kind: type,
      fields: [
        (args as unknown) || {
          additionalDelegates: [],
        },
      ],
    } as BasePlugin;
  }
  if (type === 'Royalties') {
    return {
      __kind: type,
      fields: [royaltiesToBase(args)],
    } as BasePlugin;
  }
  if (type === 'MasterEdition') {
    return {
      __kind: type,
      fields: [masterEditionToBase(args)],
    } as BasePlugin;
  }

  return {
    __kind: type,
    fields: [(args as unknown) || {}],
  } as BasePlugin;
}

/**
 * @deprecated Use the new 1.0 sdk instruction helpers like `create` instead of `createV1` which no longer require sub create functions like this.
 * @param args
 * @returns
 */
export function pluginAuthorityPairV2({
  type,
  authority,
  ...args
}: AssetPluginAuthorityPairArgsV2): PluginAuthorityPair {
  return {
    plugin: createPluginV2({
      type,
      ...args,
    } as AssetAllPluginArgsV2),
    authority: authority ? someOrNone(pluginAuthorityToBase(authority)) : { __option: 'None' },
  };
}

export function mapPluginFields(fields: Array<Record<string, unknown>>) {
  return fields.reduce((acc2, field) => ({ ...acc2, ...field }), {});
}

export function mapPlugin({
  plugin: plug,
  authority,
  offset,
}: {
  plugin: Exclude<BasePlugin, { __kind: 'Reserved' }>;
  authority: PluginAuthority;
  offset: bigint;
}): PluginsList {
  const pluginKey = toWords(plug.__kind)
    .toLowerCase()
    .split(' ')
    .reduce((s, c) => s + (c.charAt(0).toUpperCase() + c.slice(1)));

  if (plug.__kind === 'Royalties') {
    return {
      [pluginKey]: {
        authority,
        offset,
        ...royaltiesFromBase(plug.fields[0]),
      },
    };
  }

  if (plug.__kind === 'MasterEdition') {
    return {
      [pluginKey]: {
        authority,
        offset,
        ...masterEditionFromBase(plug.fields[0]),
      },
    };
  }

  return {
    [pluginKey]: {
      authority,
      offset,
      ...('fields' in plug ? mapPluginFields(plug.fields as unknown as Record<string, unknown>[]) : {}),
    },
  };
}

export function registryRecordsToPluginsList(
  registryRecords: RegistryRecord[],
  accountData: Uint8Array
) {
  return registryRecords.reduce((acc: PluginsList, record) => {
    const mappedAuthority = pluginAuthorityFromBase(record.authority);
    const deserializedPlugin = getPluginDecoder().read(
      accountData,
      Number(record.offset)
    )[0];

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

export function pluginKeyToPluginType(pluginKey: keyof PluginsList) {
  return (pluginKey.charAt(0).toUpperCase() +
    pluginKey.slice(1)) as keyof typeof PluginType;
}

export function parseExternalPluginAdapterData(
  plugin: {
    schema: ExternalPluginAdapterSchema;
  },
  record: {
    dataLen: Option<bigint | number>;
    dataOffset: Option<bigint | number>;
  },
  account: Uint8Array
): unknown {
  let data;
  if (record.dataOffset.__option === 'Some' && record.dataLen.__option === 'Some') {
    const dataSlice = account.slice(
      Number(record.dataOffset.value),
      Number(record.dataOffset.value) + Number(record.dataLen.value)
    );

    if (plugin.schema === ExternalPluginAdapterSchema.Binary) {
      data = dataSlice;
    } else if (plugin.schema === ExternalPluginAdapterSchema.Json) {
      // if data is empty, the data slice is uninitialized and should be ignored
      if (dataSlice.length !== 0) {
        try {
          data = JSON.parse(new TextDecoder().decode(dataSlice));
        } catch (e) {
          // eslint-disable-next-line no-console
          console.warn('Invalid JSON in external plugin data', (e as Error).message);
        }
      }
    } else if (plugin.schema === ExternalPluginAdapterSchema.MsgPack) {
      if (dataSlice.length === 0) {
        data = null;
      } else {
        // Note: msgpack decoding requires the @msgpack/msgpack package
        // If not available, return the raw bytes
        try {
          // Dynamic import would be ideal but for now just return binary
          data = dataSlice;
        } catch {
          data = dataSlice;
        }
      }
    }
    return data;
  }
  return undefined;
}
