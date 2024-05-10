import { none, Option, some } from '@metaplex-foundation/umi';

import {
  Key,
  PluginHeaderV1,
  Plugin as BasePlugin,
  getPluginSerializer,
  RegistryRecord,
  PluginAuthorityPair,
  PluginType,
  PluginAdapterSchema,
} from '../generated';

import { toWords } from '../utils';
import {
  CreatePluginArgs,
  PluginArgsV2,
  PluginAuthorityPairHelperArgs,
  PluginAuthorityPairArgsV2,
  PluginsList,
} from './types';
import {
  PluginAuthority,
  pluginAuthorityFromBase,
  pluginAuthorityToBase,
} from './pluginAuthority';
import { royaltiesFromBase, royaltiesToBase } from './royalties';
import { masterEditionFromBase, masterEditionToBase } from './masterEdition';

export function formPluginHeaderV1(
  pluginRegistryOffset: bigint
): Omit<PluginHeaderV1, 'publicKey' | 'header'> {
  return {
    key: Key.PluginHeaderV1,
    pluginRegistryOffset,
  };
}

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

export function createPluginV2(args: PluginArgsV2): BasePlugin {
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

export function pluginAuthorityPairV2({
  type,
  authority,
  ...args
}: PluginAuthorityPairArgsV2): PluginAuthorityPair {
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

export function parsePluginAdapterData(
  plugin: {
    schema: PluginAdapterSchema;
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

  if (plugin.schema === PluginAdapterSchema.Binary) {
    data = dataSlice;
  } else if (plugin.schema === PluginAdapterSchema.Json) {
    data = JSON.parse(new TextDecoder().decode(dataSlice));
  } else if (plugin.schema === PluginAdapterSchema.MsgPack) {
    // eslint-disable-next-line no-console
    console.warn(
      'MsgPack schema currently not supported, falling back to binary'
    );
    data = dataSlice;
  }
  return data;
}
