import { BaseValidationResultsOffset } from '../generated';

export type ValidationResultsOffset =
  | { type: 'NoOffset' }
  | { type: 'Anchor' }
  | { type: 'Custom'; offset: bigint };

export function validationResultsOffsetToBase(
  e: ValidationResultsOffset
): BaseValidationResultsOffset {
  if (e.type === 'Custom') {
    return {
      __kind: 'Custom',
      fields: [e.offset],
    };
  }

  return {
    __kind: e.type,
  };
}

export function validationResultsOffsetFromBase(
  e: BaseValidationResultsOffset
): ValidationResultsOffset {
  if (e.__kind === 'Custom') {
    return {
      type: 'Custom',
      offset: e.fields[0],
    };
  }

  return {
    type: e.__kind,
  };
}
