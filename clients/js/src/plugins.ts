import { none, some } from '@metaplex-foundation/umi';

import {
  Key,
  PluginHeaderV1,
  Plugin as BasePlugin,
  getPluginSerializer,
  RegistryRecord,
  PluginAuthorityPair,
  PluginAuthority,
  RoyaltiesArgs,
  FreezeDelegateArgs,
  AttributesArgs,
  PermanentFreezeDelegateArgs,
  PluginType,
  UpdateDelegateArgs,
  EditionArgs,
} from './generated';
import { BasePluginAuthority, PluginsList } from './types';
import { mapPluginAuthority } from './authority';
import { toWords } from './utils';

export function formPluginHeaderV1(
  pluginRegistryOffset: bigint
): Omit<PluginHeaderV1, 'publicKey' | 'header'> {
  return {
    key: Key.PluginHeaderV1,
    pluginRegistryOffset,
  };
}

export type PluginAuthorityPairHelperArgs = CreatePluginArgs & {
  authority?: PluginAuthority;
};

export type CreatePluginArgs =
  | {
      type: 'Royalties';
      data: RoyaltiesArgs;
    }
  | {
      type: 'FreezeDelegate';
      data: FreezeDelegateArgs;
    }
  | {
      type: 'BurnDelegate';
    }
  | {
      type: 'TransferDelegate';
    }
  | {
      type: 'UpdateDelegate';
      data?: UpdateDelegateArgs;
    }
  | {
      type: 'Attributes';
      data: AttributesArgs;
    }
  | {
      type: 'PermanentFreezeDelegate';
      data: PermanentFreezeDelegateArgs;
    }
  | {
      type: 'PermanentTransferDelegate';
    }
  | {
      type: 'PermanentBurnDelegate';
    }
  | {
      type: 'Edition',
      data: EditionArgs;
  };

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
  // TODO refactor when there are more required empty fields in plugins
  if (type === 'UpdateDelegate') {
    return {
      plugin: {
        __kind: type,
        fields: [
          data || {
            additionalDelegates: [],
          },
        ],
      },
      authority: authority ? some(authority) : none(),
    };
  }
  return {
    plugin: { __kind: type, fields: [data || {}] },
    authority: authority ? some(authority) : none(),
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
  authority: BasePluginAuthority;
  offset: bigint;
}): PluginsList {
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

export function registryRecordsToPluginsList(
  registryRecords: RegistryRecord[],
  accountData: Uint8Array
) {
  return registryRecords.reduce((acc: PluginsList, record) => {
    const mappedAuthority = mapPluginAuthority(record.authority);
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
