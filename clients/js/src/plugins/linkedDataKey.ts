import { PublicKey } from '@metaplex-foundation/umi';

import {
  PluginAuthority,
  pluginAuthorityFromBase,
  pluginAuthorityToBase,
} from './pluginAuthority';
import { BaseLinkedDataKey } from '../generated';

export type LinkedDataKey =
  | {
      type: 'LinkedLifecycleHook';
      hookedProgram: PublicKey;
    }
  | {
      type: 'LinkedAppData';
      dataAuthority: PluginAuthority;
    };

export function linkedDataKeyToBase(e: LinkedDataKey): BaseLinkedDataKey {
  switch (e.type) {
    case 'LinkedLifecycleHook':
      return {
        __kind: e.type,
        fields: [e.hookedProgram],
      };
    case 'LinkedAppData':
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
    case 'LinkedLifecycleHook':
      return {
        type: e.__kind,
        hookedProgram: e.fields[0],
      };
    case 'LinkedAppData':
      return {
        type: e.__kind,
        dataAuthority: pluginAuthorityFromBase(e.fields[0]),
      };
    default:
      throw new Error('Unknown LinkedDataKey type');
  }
}
