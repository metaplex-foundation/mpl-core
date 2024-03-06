import { PublicKey } from '@metaplex-foundation/umi';
import { Authority, authority as authorityHelper } from '../generated';
import { BaseAuthorities } from './types';
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

export function mapAuthorities(authorities: Authority[]) {
  return authorities.reduce((acc: BaseAuthorities, authority: Authority) => {
    const authorityKey = toWords(authority.__kind)
      .split(' ')[0]
      .toLowerCase() as keyof BaseAuthorities;
    const authorityKeysLen = Object.keys(authority).length;

    if (authorityKeysLen === 1) {
      acc = { ...acc, [authorityKey]: true };
    }

    if (authorityKeysLen > 1) {
      acc = {
        ...acc,
        [authorityKey]: [
          ...(acc[authorityKey] ? (acc[authorityKey] as any[]) : []),
          ...Object.values(authority).slice(1),
        ],
      };
    }

    return acc;
  }, {});
}
