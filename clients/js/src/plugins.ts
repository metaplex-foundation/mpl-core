import { none, some } from '@metaplex-foundation/umi';

import {
  Key,
  PluginHeader,
  Plugin as BasePlugin,
  getPluginSerializer,
  RegistryRecord,
  PluginAuthorityPair,
  PluginAuthority,
  RoyaltiesArgs,
  FreezeDelegateArgs,
  AttributesArgs,
  PermanentFreezeDelegateArgs,
} from './generated';
import { BaseAuthority, PluginsList } from './types';
import { mapAuthority } from './authority';
import { toWords } from './utils';

export function formPluginHeader(
  pluginRegistryOffset: bigint
): Omit<PluginHeader, 'publicKey' | 'header'> {
  return {
    key: Key.PluginHeader,
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
  }
  | {
    type: 'PermanentFreezeDelegate';
    data: PermanentFreezeDelegateArgs;
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
  };

export function createPlugin(args: CreatePluginArgs): BasePlugin {
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
  authority: BaseAuthority;
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
    const mappedAuthority = mapAuthority(record.authority);
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
