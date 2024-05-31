import { PublicKey } from '@metaplex-foundation/umi';
import { BaseUpdateAuthority } from '../generated';

export type UpdateAuthorityType = BaseUpdateAuthority['__kind'];

export type UpdateAuthority = {
  type: UpdateAuthorityType;
  address?: PublicKey;
};

export function updateAuthorityToBase(u: UpdateAuthority): BaseUpdateAuthority {
  if (u.type === 'None') {
    return {
      __kind: 'None',
    };
  }
  return {
    __kind: u.type,
    fields: [u.address as PublicKey],
  };
}
