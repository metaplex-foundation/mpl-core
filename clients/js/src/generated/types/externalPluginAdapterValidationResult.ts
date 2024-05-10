/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { Serializer, scalarEnum } from '@metaplex-foundation/umi/serializers';

export enum ExternalPluginAdapterValidationResult {
  Approved,
  Rejected,
  Pass,
}

export type ExternalPluginAdapterValidationResultArgs =
  ExternalPluginAdapterValidationResult;

export function getExternalPluginAdapterValidationResultSerializer(): Serializer<
  ExternalPluginAdapterValidationResultArgs,
  ExternalPluginAdapterValidationResult
> {
  return scalarEnum<ExternalPluginAdapterValidationResult>(
    ExternalPluginAdapterValidationResult,
    { description: 'ExternalPluginAdapterValidationResult' }
  ) as Serializer<
    ExternalPluginAdapterValidationResultArgs,
    ExternalPluginAdapterValidationResult
  >;
}
