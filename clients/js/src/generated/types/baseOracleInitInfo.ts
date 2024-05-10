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
  BaseValidationResultsOffset,
  BaseValidationResultsOffsetArgs,
  ExternalPluginAdapterCheckResult,
  ExternalPluginAdapterCheckResultArgs,
  HookableLifecycleEvent,
  HookableLifecycleEventArgs,
  getBaseExtraAccountSerializer,
  getBasePluginAuthoritySerializer,
  getBaseValidationResultsOffsetSerializer,
  getExternalPluginAdapterCheckResultSerializer,
  getHookableLifecycleEventSerializer,
} from '.';

export type BaseOracleInitInfo = {
  baseAddress: PublicKey;
  initPluginAuthority: Option<BasePluginAuthority>;
  lifecycleChecks: Array<
    [HookableLifecycleEvent, ExternalPluginAdapterCheckResult]
  >;
  baseAddressConfig: Option<BaseExtraAccount>;
  resultsOffset: Option<BaseValidationResultsOffset>;
};

export type BaseOracleInitInfoArgs = {
  baseAddress: PublicKey;
  initPluginAuthority: OptionOrNullable<BasePluginAuthorityArgs>;
  lifecycleChecks: Array<
    [HookableLifecycleEventArgs, ExternalPluginAdapterCheckResultArgs]
  >;
  baseAddressConfig: OptionOrNullable<BaseExtraAccountArgs>;
  resultsOffset: OptionOrNullable<BaseValidationResultsOffsetArgs>;
};

export function getBaseOracleInitInfoSerializer(): Serializer<
  BaseOracleInitInfoArgs,
  BaseOracleInitInfo
> {
  return struct<BaseOracleInitInfo>(
    [
      ['baseAddress', publicKeySerializer()],
      ['initPluginAuthority', option(getBasePluginAuthoritySerializer())],
      [
        'lifecycleChecks',
        array(
          tuple([
            getHookableLifecycleEventSerializer(),
            getExternalPluginAdapterCheckResultSerializer(),
          ])
        ),
      ],
      ['baseAddressConfig', option(getBaseExtraAccountSerializer())],
      ['resultsOffset', option(getBaseValidationResultsOffsetSerializer())],
    ],
    { description: 'BaseOracleInitInfo' }
  ) as Serializer<BaseOracleInitInfoArgs, BaseOracleInitInfo>;
}
