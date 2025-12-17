import { type Address } from '@solana/addresses';
import { type BaseExternalPluginAdapterKey } from '../generated';
import { type PluginAuthority, pluginAuthorityToBase } from './pluginAuthority';
import { type LinkedDataKey, linkedDataKeyToBase } from './linkedDataKey';

export type ExternalPluginAdapterKey =
  | {
      type: 'LifecycleHook';
      hookedProgram: Address;
    }
  | {
      type: 'Oracle';
      baseAddress: Address;
    }
  | {
      type: 'AppData';
      dataAuthority: PluginAuthority;
    }
  | {
      type: 'LinkedLifecycleHook';
      hookedProgram: Address;
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
    case 'LinkedLifecycleHook':
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
