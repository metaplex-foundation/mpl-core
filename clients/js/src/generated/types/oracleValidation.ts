/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import {
  GetDataEnumKind,
  GetDataEnumKindContent,
  Serializer,
  dataEnum,
  struct,
  unit,
} from '@metaplex-foundation/umi/serializers';
import {
  AdapterValidationResult,
  AdapterValidationResultArgs,
  getAdapterValidationResultSerializer,
} from '.';

export type OracleValidation =
  | { __kind: 'Uninitialized' }
  | {
      __kind: 'V1';
      create: AdapterValidationResult;
      transfer: AdapterValidationResult;
      burn: AdapterValidationResult;
      update: AdapterValidationResult;
    };

export type OracleValidationArgs =
  | { __kind: 'Uninitialized' }
  | {
      __kind: 'V1';
      create: AdapterValidationResultArgs;
      transfer: AdapterValidationResultArgs;
      burn: AdapterValidationResultArgs;
      update: AdapterValidationResultArgs;
    };

export function getOracleValidationSerializer(): Serializer<
  OracleValidationArgs,
  OracleValidation
> {
  return dataEnum<OracleValidation>(
    [
      ['Uninitialized', unit()],
      [
        'V1',
        struct<GetDataEnumKindContent<OracleValidation, 'V1'>>([
          ['create', getAdapterValidationResultSerializer()],
          ['transfer', getAdapterValidationResultSerializer()],
          ['burn', getAdapterValidationResultSerializer()],
          ['update', getAdapterValidationResultSerializer()],
        ]),
      ],
    ],
    { description: 'OracleValidation' }
  ) as Serializer<OracleValidationArgs, OracleValidation>;
}

// Data Enum Helpers.
export function oracleValidation(
  kind: 'Uninitialized'
): GetDataEnumKind<OracleValidationArgs, 'Uninitialized'>;
export function oracleValidation(
  kind: 'V1',
  data: GetDataEnumKindContent<OracleValidationArgs, 'V1'>
): GetDataEnumKind<OracleValidationArgs, 'V1'>;
export function oracleValidation<K extends OracleValidationArgs['__kind']>(
  kind: K,
  data?: any
): Extract<OracleValidationArgs, { __kind: K }> {
  return Array.isArray(data)
    ? { __kind: kind, fields: data }
    : { __kind: kind, ...(data ?? {}) };
}
export function isOracleValidation<K extends OracleValidation['__kind']>(
  kind: K,
  value: OracleValidation
): value is OracleValidation & { __kind: K } {
  return value.__kind === kind;
}
