import { none, some } from '@metaplex-foundation/umi';

import {
  Key,
  PluginHeader,
  Plugin,
  getPluginSerializer,
  RegistryRecord,
  PluginAuthorityPair,
  Authority,
  RoyaltiesArgs,
  FreezeArgs,
  BurnArgs,
  TransferArgs,
  UpdateDelegateArgs,
  PermanentFreezeArgs,
  AttributesArgs,
  PermanentTransferArgs,
  PermanentBurnArgs,
} from '../generated';
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

export type PluginAuthorityPairHelperArgs =
  | {
      type: 'Royalties';
      authority?: Authority;
      data: RoyaltiesArgs;
    }
  | {
      type: 'Freeze';
      authority?: Authority;
      data: FreezeArgs;
    }
  | {
      type: 'Burn';
      authority?: Authority;
      data?: BurnArgs;
    }
  | {
      type: 'Transfer';
      authority?: Authority;
      data?: TransferArgs;
    }
  | {
      type: 'UpdateDelegate';
      authority?: Authority;
      data?: UpdateDelegateArgs;
    }
  | {
      type: 'PermanentFreeze';
      authority?: Authority;
      data: PermanentFreezeArgs;
    }
  | {
      type: 'Attributes';
      authority?: Authority;
      data: AttributesArgs;
    }
  | {
      type: 'PermanentFreeze';
      authority?: Authority;
      data: PermanentFreezeArgs;
    }
  | {
      type: 'PermanentTransfer';
      authority?: Authority;
      data?: PermanentTransferArgs;
    }
  | {
      type: 'PermanentBurn';
      authority?: Authority;
      data?: PermanentBurnArgs;
    };

export function pluginAuthorityPair(
  args: PluginAuthorityPairHelperArgs
): PluginAuthorityPair {
  const { type, authority, data } = args;
  return {
    plugin: { __kind: type, fields: [(data as any) || {}] },
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
  plugin: Exclude<Plugin, { __kind: 'Reserved' }>;
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
