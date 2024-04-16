import { BaseSeed } from "../generated";
import { RenameToType } from "./util";

export type Seed = Exclude<RenameToType<BaseSeed>, {type: 'Bytes'}> | {
  type: 'Bytes',
  bytes: Uint8Array,
}

export function seedToBase(s: Seed): BaseSeed {
  if (s.type === 'Bytes') {
    return {
      __kind: 'Bytes',
      fields: [s.bytes],
    }
  }
  return {
    __kind: s.type,
  }
}