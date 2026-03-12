import { type Address } from '@solana/addresses';
import { type ReadonlyUint8Array } from '@solana/codecs';
import { type BaseSeed } from '../generated';
import { type RenameToType } from '../utils';

export type Seed =
  | Exclude<RenameToType<BaseSeed>, { type: 'Address' } | { type: 'Bytes' }>
  | {
      type: 'Address';
      pubkey: Address;
    }
  | {
      type: 'Bytes';
      bytes: ReadonlyUint8Array;
    };

export function seedToBase(s: Seed): BaseSeed {
  if (s.type === 'Address') {
    return {
      __kind: 'Address',
      fields: [s.pubkey],
    };
  }

  if (s.type === 'Bytes') {
    return {
      __kind: 'Bytes',
      fields: [s.bytes],
    };
  }
  return {
    __kind: s.type,
  };
}

export function seedFromBase(s: BaseSeed): Seed {
  if (s.__kind === 'Address') {
    return {
      type: 'Address',
      pubkey: s.fields[0],
    };
  }
  if (s.__kind === 'Bytes') {
    return {
      type: 'Bytes',
      bytes: s.fields[0],
    };
  }
  return {
    type: s.__kind,
  };
}
