import { PublicKey } from '@metaplex-foundation/umi';
import { authority } from '../generated';

// Authorities data enum helpers
export function noneAuthority() {
  return authority('None');
}

export function ownerAuthority() {
  return authority('Owner');
}

export function updateAuthority() {
  return authority('UpdateAuthority');
}

export function pubkeyAuthority(address: PublicKey) {
  return authority('Pubkey', { address });
}

export function permanentAuthority(address: PublicKey) {
  return authority('Permanent', { address });
}
