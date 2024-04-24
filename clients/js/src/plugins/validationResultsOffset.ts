import { BaseValidationResultsOffset } from '../generated';

export type ValidationResultsOffset =
  | { type: 'NoOffset' }
  | { type: 'Anchor' }
  | { type: 'Custom'; offset: bigint };

export function validationResultsOffsetToBase(
  e: ValidationResultsOffset
): BaseValidationResultsOffset {
  if (e.type === 'NoOffset') {
    return {
      __kind: 'NoOffset',
    };
  }
  if (e.type === 'Anchor') {
    return {
      __kind: 'Anchor',
    };
  }
  return {
    __kind: 'Custom',
    fields: [e.offset],
  };
}
