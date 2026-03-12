import { isSome, none, Option, some } from '@metaplex-foundation/umi';

import { decode } from '@msgpack/msgpack';
import {
  Plugin as BasePlugin,
  ExternalPluginAdapterSchema,
  getPluginSerializer,
  Key,
  PluginAuthorityPair,
  PluginHeaderV1,
  PluginType,
  RegistryRecord,
} from '../generated';

import { toWords } from '../utils';
import { masterEditionFromBase, masterEditionToBase } from './masterEdition';
import {
  PluginAuthority,
  pluginAuthorityFromBase,
  pluginAuthorityToBase,
} from './pluginAuthority';
import { royaltiesFromBase, royaltiesToBase } from './royalties';
import {
  AssetAllPluginArgsV2,
  AssetPluginAuthorityPairArgsV2,
  CreatePluginArgs,
  PluginAuthorityPairHelperArgs,
  PluginsList,
} from './types';

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
        (args as any).data || {
          additionalDelegates: [],
        },
      ],
    };
  }
  return {
    __kind: args.type,
    fields: [(args as any).data || {}],
  };
}

/**
 * @deprecated Use the new 1.0 sdk instruction helpers like `create` instead of `createV1` which no longer require sub create functions like this.
 * @param args
 * @returns
 */
export function pluginAuthorityPair(
  args: PluginAuthorityPairHelperArgs
): PluginAuthorityPair {
  const { type, authority, data } = args as any;
  return {
    plugin: createPlugin({
      type,
      data,
    }),
    authority: authority ? some(authority) : none(),
  };
}

export function createPluginV2(args: AssetAllPluginArgsV2): BasePlugin {
  // TODO refactor when there are more required empty fields in plugins
  const { type } = args;
  if (type === 'UpdateDelegate') {
    return {
      __kind: type,
      fields: [
        (args as any) || {
          additionalDelegates: [],
        },
      ],
    };
  }
  if (type === 'Royalties') {
    return {
      __kind: type,
      fields: [royaltiesToBase(args)],
    };
  }
  if (type === 'MasterEdition') {
    return {
      __kind: type,
      fields: [masterEditionToBase(args)],
    };
  }

  return {
    __kind: type,
    fields: [(args as any) || {}],
  };
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
    } as any),
    authority: authority ? some(pluginAuthorityToBase(authority)) : none(),
  };
}

export function mapPluginFields(fields: Array<Record<string, any>>) {
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
      ...('fields' in plug ? mapPluginFields(plug.fields) : {}),
    },
  };
}

export function registryRecordsToPluginsList(
  registryRecords: RegistryRecord[],
  accountData: Uint8Array
) {
  return registryRecords.reduce((acc: PluginsList, record) => {
    const mappedAuthority = pluginAuthorityFromBase(record.authority);
    const deserializedPlugin = getPluginSerializer().deserialize(
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
): any {
  let data;
  if (isSome(record.dataOffset) && isSome(record.dataLen)) {
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
          console.warn('Invalid JSON in external plugin data', e.message);
        }
      }
    } else if (plugin.schema === ExternalPluginAdapterSchema.MsgPack) {
      if (dataSlice.length === 0) {
        data = null;
      } else {
        data = decode(dataSlice);
      }
    }
    return data;
  }
  throw new Error('Invalid DataStore, missing dataOffset or dataLen');
}
