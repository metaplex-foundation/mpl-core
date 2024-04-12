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
  ExtraAccount,
  ExtraAccountArgs,
  HookableLifecycleEvent,
  HookableLifecycleEventArgs,
  PluginAuthority,
  PluginAuthorityArgs,
  getExternalCheckResultSerializer,
  getExternalPluginSchemaSerializer,
  getExtraAccountSerializer,
  getHookableLifecycleEventSerializer,
  getPluginAuthoritySerializer,
} from '.';

export type LifecycleHookInitInfo = {
  initAuthority: Option<PluginAuthority>;
  lifecycleChecks: Option<Array<[HookableLifecycleEvent, ExternalCheckResult]>>;
  extraAccounts: Option<Array<ExtraAccount>>;
  schema: Option<ExternalPluginSchema>;
  data: Option<Uint8Array>;
};

export type LifecycleHookInitInfoArgs = {
  initAuthority: OptionOrNullable<PluginAuthorityArgs>;
  lifecycleChecks: OptionOrNullable<
    Array<[HookableLifecycleEventArgs, ExternalCheckResultArgs]>
  >;
  extraAccounts: OptionOrNullable<Array<ExtraAccountArgs>>;
  schema: OptionOrNullable<ExternalPluginSchemaArgs>;
  data: OptionOrNullable<Uint8Array>;
};

export function getLifecycleHookInitInfoSerializer(): Serializer<
  LifecycleHookInitInfoArgs,
  LifecycleHookInitInfo
> {
  return struct<LifecycleHookInitInfo>(
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
      ['extraAccounts', option(array(getExtraAccountSerializer()))],
      ['schema', option(getExternalPluginSchemaSerializer())],
      ['data', option(bytes({ size: u32() }))],
    ],
    { description: 'LifecycleHookInitInfo' }
  ) as Serializer<LifecycleHookInitInfoArgs, LifecycleHookInitInfo>;
}