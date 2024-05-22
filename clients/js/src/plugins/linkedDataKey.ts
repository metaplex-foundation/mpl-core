import { PublicKey } from '@metaplex-foundation/umi';

import {
  PluginAuthority,
  pluginAuthorityFromBase,
  pluginAuthorityToBase,
} from './pluginAuthority';
import { BaseLinkedDataKey } from '../generated';

export type LinkedDataKey =
  | {
      type: 'LifecycleHook';
      hookedProgram: PublicKey;
      dataAuthority: PluginAuthority;
    }
  | {
      type: 'AssetLinkedSecureDataStore';
      dataAuthority: PluginAuthority;
    };

export function linkedDataKeyToBase(e: LinkedDataKey): BaseLinkedDataKey {
  switch (e.type) {
    case 'LifecycleHook':
      return {
        __kind: e.type,
        fields: [e.hookedProgram, pluginAuthorityToBase(e.dataAuthority)],
      };
    case 'AssetLinkedSecureDataStore':
      return {
        __kind: e.type,
        fields: [pluginAuthorityToBase(e.dataAuthority)],
      };
    default:
      throw new Error('Unknown LinkedDataKey type');
  }
}

export function linkedDataKeyFromBase(e: BaseLinkedDataKey): LinkedDataKey {
  switch (e.__kind) {
    case 'LifecycleHook':
      return {
        type: e.__kind,
        hookedProgram: e.fields[0],
        dataAuthority: pluginAuthorityFromBase(e.fields[1]),
      };
    case 'AssetLinkedSecureDataStore':
      return {
        type: e.__kind,
        dataAuthority: pluginAuthorityFromBase(e.fields[0]),
      };
    default:
      throw new Error('Unknown LinkedDataKey type');
  }
}
