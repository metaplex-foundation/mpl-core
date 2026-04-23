import { keccak_256 } from '@noble/hashes/sha3';

function mergeBytes(arrays: Uint8Array[]): Uint8Array {
  const totalLength = arrays.reduce((sum, arr) => sum + arr.length, 0);
  return arrays.reduce(
    (state, arr) => {
      state.result.set(arr, state.offset);
      return {
        offset: state.offset + arr.length,
        result: state.result,
      };
    },
    { offset: 0, result: new Uint8Array(totalLength) }
  ).result;
}

export function hash(input: Uint8Array | Uint8Array[]): Uint8Array {
  return keccak_256(Array.isArray(input) ? mergeBytes(input) : input);
}
