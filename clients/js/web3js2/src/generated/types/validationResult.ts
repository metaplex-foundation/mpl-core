/**
 * This code was AUTOGENERATED using the codama library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun codama to update it.
 *
 * @see https://github.com/codama-idl/codama
 */

import {
  combineCodec,
  getEnumDecoder,
  getEnumEncoder,
  type Codec,
  type Decoder,
  type Encoder,
} from '@solana/kit';

export enum ValidationResult {
  Approved,
  Rejected,
  Pass,
  ForceApproved,
}

export type ValidationResultArgs = ValidationResult;

export function getValidationResultEncoder(): Encoder<ValidationResultArgs> {
  return getEnumEncoder(ValidationResult);
}

export function getValidationResultDecoder(): Decoder<ValidationResult> {
  return getEnumDecoder(ValidationResult);
}

export function getValidationResultCodec(): Codec<
  ValidationResultArgs,
  ValidationResult
> {
  return combineCodec(
    getValidationResultEncoder(),
    getValidationResultDecoder()
  );
}
