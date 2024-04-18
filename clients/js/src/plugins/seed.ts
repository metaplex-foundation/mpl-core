import { BaseSeed } from '../generated';
import { RenameToType } from '../utils';

export type Seed =
  | Exclude<RenameToType<BaseSeed>, { type: 'Bytes' }>
  | {
      type: 'Bytes';
      bytes: Uint8Array;
    };

export function seedToBase(s: Seed): BaseSeed {
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
