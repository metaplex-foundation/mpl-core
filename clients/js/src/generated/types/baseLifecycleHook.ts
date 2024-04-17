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
  u64,
} from '@metaplex-foundation/umi/serializers';
import {
  BaseExtraAccount,
  BaseExtraAccountArgs,
  ExternalPluginSchema,
  ExternalPluginSchemaArgs,
  getBaseExtraAccountSerializer,
  getExternalPluginSchemaSerializer,
} from '.';

export type BaseLifecycleHook = {
  hookedProgram: PublicKey;
  extraAccounts: Option<Array<BaseExtraAccount>>;
  schema: ExternalPluginSchema;
  dataOffset: bigint;
  dataLen: bigint;
};

export type BaseLifecycleHookArgs = {
  hookedProgram: PublicKey;
  extraAccounts: OptionOrNullable<Array<BaseExtraAccountArgs>>;
  schema: ExternalPluginSchemaArgs;
  dataOffset: number | bigint;
  dataLen: number | bigint;
};

export function getBaseLifecycleHookSerializer(): Serializer<
  BaseLifecycleHookArgs,
  BaseLifecycleHook
> {
  return struct<BaseLifecycleHook>(
    [
      ['hookedProgram', publicKeySerializer()],
      ['extraAccounts', option(array(getBaseExtraAccountSerializer()))],
      ['schema', getExternalPluginSchemaSerializer()],
      ['dataOffset', u64()],
      ['dataLen', u64()],
    ],
    { description: 'BaseLifecycleHook' }
  ) as Serializer<BaseLifecycleHookArgs, BaseLifecycleHook>;
}
