import { PublicKey } from '@metaplex-foundation/umi';
import { BasePluginAuthority } from '../generated';

export type PluginAuthority = {
  type: PluginAuthorityType;
  address?: PublicKey;
};

export type PluginAuthorityType = BasePluginAuthority['__kind'];

export function pluginAuthorityToBase(u: PluginAuthority): BasePluginAuthority {
  if (u.type === 'Address') {
    return {
      __kind: 'Address',
      address: u.address as PublicKey,
    };
  }
  return {
    __kind: u.type,
  };
}

export function pluginAuthorityFromBase(
  authority: BasePluginAuthority
): PluginAuthority {
  return {
    type: authority.__kind,
    address: (authority as any).address,
  };
}
export function comparePluginAuthorities(
  a: PluginAuthority,
  b: PluginAuthority
): boolean {
  if (a.type !== b.type) {
    return false;
  }

  return a.address === b.address;
}
