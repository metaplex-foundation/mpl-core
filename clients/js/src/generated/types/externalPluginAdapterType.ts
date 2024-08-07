/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { Serializer, scalarEnum } from '@metaplex-foundation/umi/serializers';

export enum ExternalPluginAdapterType {
  LifecycleHook,
  Oracle,
  AppData,
  LinkedLifecycleHook,
  LinkedAppData,
  DataSection,
}

export type ExternalPluginAdapterTypeArgs = ExternalPluginAdapterType;

export function getExternalPluginAdapterTypeSerializer(): Serializer<
  ExternalPluginAdapterTypeArgs,
  ExternalPluginAdapterType
> {
  return scalarEnum<ExternalPluginAdapterType>(ExternalPluginAdapterType, {
    description: 'ExternalPluginAdapterType',
  }) as Serializer<ExternalPluginAdapterTypeArgs, ExternalPluginAdapterType>;
}
