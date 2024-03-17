import { PublicKey } from '@metaplex-foundation/umi';
import {
  PluginAuthority,
  pluginAuthority,
} from './generated/types/pluginAuthority';
import { BaseAuthority } from './types';

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

export function pubkeyPluginAuthority(address: PublicKey) {
  return pluginAuthority('Pubkey', { address });
}

export function mapAuthority(authority: PluginAuthority): BaseAuthority {
  return {
    type: authority.__kind,
    address: (authority as any).address,
  };
}
