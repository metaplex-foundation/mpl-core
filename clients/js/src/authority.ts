import { PublicKey } from '@metaplex-foundation/umi';
import { BasePluginAuthority } from './generated';
import { PluginAuthority, pluginAuthority } from './plugins';

// Authorities data helpers
export function nonePluginAuthority() {
  return pluginAuthority('None');
}

export function ownerPluginAuthority() {
  return pluginAuthority('Owner');
}

export function updatePluginAuthority() {
  return pluginAuthority('UpdateAuthority');
}

export function addressPluginAuthority(address: PublicKey) {
  return pluginAuthority('Address', { address });
}

export function mapPluginAuthority(
  authority: BasePluginAuthority
): PluginAuthority {
  return {
    type: authority.__kind,
    address: (authority as any).address,
  };
}
