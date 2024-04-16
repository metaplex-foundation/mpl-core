import { AccountMeta, Context, none, PublicKey, some } from '@metaplex-foundation/umi';

import {
  Key,
  PluginHeaderV1,
  Plugin as BasePlugin,
  getPluginSerializer,
  RegistryRecord,
  PluginAuthorityPair,
  PluginType,
  BaseExternalPluginInitInfoArgs,
  BaseExternalPluginUpdateInfoArgs,
  ExternalRegistryRecord,
  BaseExternalPluginKeyArgs,
} from '../generated';

import { mapPluginAuthority } from '../authority';
import { toWords } from '../utils';
import { CreatePluginArgs, CreatePluginArgsV2, PluginAuthorityPairHelperArgs, PluginAuthorityPairHelperArgsV2, PluginsList  } from "./types";
import { LifecycleEvent } from './lifecycleChecks';
import { PluginAuthority, pluginAuthorityToBase } from './pluginAuthority';
import { extraAccountToAccountMeta } from './extraAccount';
import { ExternalPluginInitInfoArgs, externalPluginManifests, ExternalPluginsList, ExternalPluginUpdateInfoArgs } from './externalPlugins';
import { royaltiesToBase } from './royalties';

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

export function createPluginV2(args: CreatePluginArgsV2): BasePlugin {
  // TODO refactor when there are more required empty fields in plugins
  const { type } = args
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
  return {
    __kind: type,
    fields: [(args as any) || {}],
  };
}


export function pluginAuthorityPairV2(
  { type, authority, ...args }: PluginAuthorityPairHelperArgsV2
): PluginAuthorityPair {
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

  return {
    [pluginKey]: {
      authority,
      offset,
      // TODO deserialize royalties nicely
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


export const findExtraAccounts = (context: Pick<Context, 'eddsa'>, lifecycle: LifecycleEvent, externalPlugins: ExternalPluginsList, inputs: {
  asset: PublicKey,
  collection?: PublicKey,
  owner: PublicKey,
  recipient?: PublicKey,
}): AccountMeta[] => {
  const accounts: AccountMeta[] = []
  const {asset, collection, owner, recipient} = inputs
  
  externalPlugins.oracles?.forEach((oracle) => {
    if(oracle.lifecycleChecks?.[lifecycle]) {
      if (oracle.pda) {
        accounts.push(extraAccountToAccountMeta(context, oracle.pda, {
          program: oracle.baseAddress,
          asset,
          collection,
          recipient,
        }))
      } else {
        accounts.push({
          pubkey: oracle.baseAddress,
          isSigner: false,
          isWritable: false
        })
      }
    }
  })

  externalPlugins.lifecycleHooks?.forEach((hook) => {
    if(hook.lifecycleChecks?.[lifecycle]) {
      accounts.push({
        pubkey: hook.hookedProgram,
        isSigner: false,
        isWritable: false
      })

      hook.extraAccounts?.forEach((extra) => {
        accounts.push(extraAccountToAccountMeta(context, extra, {
          program: hook.hookedProgram,
          asset,
          collection,
          recipient,
          owner,
        }))
      })
    }
  })

  return accounts
}
