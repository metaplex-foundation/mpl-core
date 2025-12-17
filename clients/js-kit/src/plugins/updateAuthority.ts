import { type Address } from '@solana/addresses';
import { type BaseUpdateAuthority } from '../generated';

export type UpdateAuthorityType = BaseUpdateAuthority['__kind'];

export type UpdateAuthority = {
  type: UpdateAuthorityType;
  address?: Address;
};

export function updateAuthorityToBase(u: UpdateAuthority): BaseUpdateAuthority {
  if (u.type === 'None') {
    return {
      __kind: 'None',
    };
  }
  return {
    __kind: u.type,
    fields: [u.address as Address],
  };
}

export function updateAuthorityFromBase(
  authority: BaseUpdateAuthority
): UpdateAuthority {
  return {
    type: authority.__kind,
    address:
      authority.__kind === 'None'
        ? undefined
        : (authority as { fields: readonly [Address] }).fields[0],
  };
}

export function updateAuthority(type: 'None'): UpdateAuthority;
export function updateAuthority(
  type: 'Address' | 'Collection',
  address: Address
): UpdateAuthority;
export function updateAuthority(
  type: UpdateAuthorityType,
  address?: Address
): UpdateAuthority {
  return {
    type,
    address,
  };
}
