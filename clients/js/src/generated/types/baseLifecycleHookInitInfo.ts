/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { Option, OptionOrNullable, PublicKey } from '@metaplex-foundation/umi';
import {
  Serializer,
  array,
  option,
  publicKey as publicKeySerializer,
  struct,
  tuple,
} from '@metaplex-foundation/umi/serializers';
import {
  BaseExtraAccount,
  BaseExtraAccountArgs,
  BasePluginAuthority,
  BasePluginAuthorityArgs,
  ExternalCheckResult,
  ExternalCheckResultArgs,
  ExternalPluginSchema,
  ExternalPluginSchemaArgs,
  HookableLifecycleEvent,
  HookableLifecycleEventArgs,
  getBaseExtraAccountSerializer,
  getBasePluginAuthoritySerializer,
  getExternalCheckResultSerializer,
  getExternalPluginSchemaSerializer,
  getHookableLifecycleEventSerializer,
} from '.';

export type BaseLifecycleHookInitInfo = {
  hookedProgram: PublicKey;
  initPluginAuthority: Option<BasePluginAuthority>;
  lifecycleChecks: Array<[HookableLifecycleEvent, ExternalCheckResult]>;
  extraAccounts: Option<Array<BaseExtraAccount>>;
  dataAuthority: Option<BasePluginAuthority>;
  schema: Option<ExternalPluginSchema>;
};

export type BaseLifecycleHookInitInfoArgs = {
  hookedProgram: PublicKey;
  initPluginAuthority: OptionOrNullable<BasePluginAuthorityArgs>;
  lifecycleChecks: Array<[HookableLifecycleEventArgs, ExternalCheckResultArgs]>;
  extraAccounts: OptionOrNullable<Array<BaseExtraAccountArgs>>;
  dataAuthority: OptionOrNullable<BasePluginAuthorityArgs>;
  schema: OptionOrNullable<ExternalPluginSchemaArgs>;
};

export function getBaseLifecycleHookInitInfoSerializer(): Serializer<
  BaseLifecycleHookInitInfoArgs,
  BaseLifecycleHookInitInfo
> {
  return struct<BaseLifecycleHookInitInfo>(
    [
      ['hookedProgram', publicKeySerializer()],
      ['initPluginAuthority', option(getBasePluginAuthoritySerializer())],
      [
        'lifecycleChecks',
        array(
          tuple([
            getHookableLifecycleEventSerializer(),
            getExternalCheckResultSerializer(),
          ])
        ),
      ],
      ['extraAccounts', option(array(getBaseExtraAccountSerializer()))],
      ['dataAuthority', option(getBasePluginAuthoritySerializer())],
      ['schema', option(getExternalPluginSchemaSerializer())],
    ],
    { description: 'BaseLifecycleHookInitInfo' }
  ) as Serializer<BaseLifecycleHookInitInfoArgs, BaseLifecycleHookInitInfo>;
}
