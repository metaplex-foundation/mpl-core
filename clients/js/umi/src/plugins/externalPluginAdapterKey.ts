import { PublicKey } from '@metaplex-foundation/umi';
import { BaseExternalPluginAdapterKey } from '../generated';
import { PluginAuthority, pluginAuthorityToBase } from './pluginAuthority';
import { LinkedDataKey, linkedDataKeyToBase } from './linkedDataKey';

export type ExternalPluginAdapterKey =
  | {
      type: 'LifecycleHook';
      hookedProgram: PublicKey;
    }
  | {
      type: 'Oracle';
      baseAddress: PublicKey;
    }
  | {
      type: 'AppData';
      dataAuthority: PluginAuthority;
    }
  | {
      type: 'LinkedLifecycleHook';
      dataAuthority: PublicKey;
    }
  | {
      type: 'LinkedAppData';
      dataAuthority: PluginAuthority;
    }
  | { type: 'DataSection'; parentKey: LinkedDataKey };

export function externalPluginAdapterKeyToBase(
  e: ExternalPluginAdapterKey
): BaseExternalPluginAdapterKey {
  switch (e.type) {
    case 'Oracle':
      return {
        __kind: e.type,
        fields: [e.baseAddress],
      };
    case 'AppData':
    case 'LinkedAppData':
      return {
        __kind: e.type,
        fields: [pluginAuthorityToBase(e.dataAuthority)],
      };
    case 'LifecycleHook':
      return {
        __kind: e.type,
        fields: [e.hookedProgram],
      };
    case 'DataSection':
      return {
        __kind: e.type,
        fields: [linkedDataKeyToBase(e.parentKey)],
      };
    default:
      throw new Error('Unknown ExternalPluginAdapterKey type');
  }
}
