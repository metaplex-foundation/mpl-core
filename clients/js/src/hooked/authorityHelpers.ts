import { PublicKey } from '@metaplex-foundation/umi';
import { Authority, authority as authorityHelper } from '../generated';
import { BaseAuthority } from './types';
import { toWords } from './utils';

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
  const authorityKey = toWords(authority.__kind)
    .split(' ')[0]
    .toLowerCase() as keyof BaseAuthority;

  if (Object.keys(authority).length > 1) {
    return {
      [authorityKey]: Object.values(authority).slice(1),
    };
  }

  return { [authorityKey]: true };
}
