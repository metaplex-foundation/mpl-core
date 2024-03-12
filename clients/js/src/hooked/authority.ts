import { PublicKey } from '@metaplex-foundation/umi';
import { Authority, authority as authorityHelper } from '../generated';
import { BaseAuthority } from './types';

// Authorities data helpers
export function getNoneAuthority() {
  return authorityHelper('None');
}

export function getOwnerAuthority() {
  return authorityHelper('Owner');
}

export function getUpdateAuthority() {
  return authorityHelper('UpdateAuthority');
}

export function getPubkeyAuthority(address: PublicKey) {
  return authorityHelper('Pubkey', { address });
}

export function getPermanentAuthority(address: PublicKey) {
  return authorityHelper('Permanent', { address });
}

export function mapAuthority(authority: Authority): BaseAuthority {
  return {
    type: authority.__kind,
    address: (authority as any).address,
  };
}
