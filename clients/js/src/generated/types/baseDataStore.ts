/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { Serializer, struct, u64 } from '@metaplex-foundation/umi/serializers';
import {
  BasePluginAuthority,
  BasePluginAuthorityArgs,
  ExternalPluginSchema,
  ExternalPluginSchemaArgs,
  getBasePluginAuthoritySerializer,
  getExternalPluginSchemaSerializer,
} from '.';

export type BaseDataStore = {
  dataAuthority: BasePluginAuthority;
  schema: ExternalPluginSchema;
  dataOffset: bigint;
  dataLen: bigint;
};

export type BaseDataStoreArgs = {
  dataAuthority: BasePluginAuthorityArgs;
  schema: ExternalPluginSchemaArgs;
  dataOffset: number | bigint;
  dataLen: number | bigint;
};

export function getBaseDataStoreSerializer(): Serializer<
  BaseDataStoreArgs,
  BaseDataStore
> {
  return struct<BaseDataStore>(
    [
      ['dataAuthority', getBasePluginAuthoritySerializer()],
      ['schema', getExternalPluginSchemaSerializer()],
      ['dataOffset', u64()],
      ['dataLen', u64()],
    ],
    { description: 'BaseDataStore' }
  ) as Serializer<BaseDataStoreArgs, BaseDataStore>;
}