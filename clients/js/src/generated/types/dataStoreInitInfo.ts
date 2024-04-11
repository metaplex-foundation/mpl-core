/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { Option, OptionOrNullable } from '@metaplex-foundation/umi';
import {
  Serializer,
  array,
  bytes,
  option,
  struct,
  tuple,
  u32,
} from '@metaplex-foundation/umi/serializers';
import {
  ExternalCheckResult,
  ExternalCheckResultArgs,
  ExternalPluginSchema,
  ExternalPluginSchemaArgs,
  HookableLifecycleEvent,
  HookableLifecycleEventArgs,
  PluginAuthority,
  PluginAuthorityArgs,
  getExternalCheckResultSerializer,
  getExternalPluginSchemaSerializer,
  getHookableLifecycleEventSerializer,
  getPluginAuthoritySerializer,
} from '.';

export type DataStoreInitInfo = {
  initAuthority: Option<PluginAuthority>;
  lifecycleChecks: Option<Array<[HookableLifecycleEvent, ExternalCheckResult]>>;
  schema: Option<ExternalPluginSchema>;
  data: Option<Uint8Array>;
};

export type DataStoreInitInfoArgs = {
  initAuthority: OptionOrNullable<PluginAuthorityArgs>;
  lifecycleChecks: OptionOrNullable<
    Array<[HookableLifecycleEventArgs, ExternalCheckResultArgs]>
  >;
  schema: OptionOrNullable<ExternalPluginSchemaArgs>;
  data: OptionOrNullable<Uint8Array>;
};

export function getDataStoreInitInfoSerializer(): Serializer<
  DataStoreInitInfoArgs,
  DataStoreInitInfo
> {
  return struct<DataStoreInitInfo>(
    [
      ['initAuthority', option(getPluginAuthoritySerializer())],
      [
        'lifecycleChecks',
        option(
          array(
            tuple([
              getHookableLifecycleEventSerializer(),
              getExternalCheckResultSerializer(),
            ])
          )
        ),
      ],
      ['schema', option(getExternalPluginSchemaSerializer())],
      ['data', option(bytes({ size: u32() }))],
    ],
    { description: 'DataStoreInitInfo' }
  ) as Serializer<DataStoreInitInfoArgs, DataStoreInitInfo>;
}
