/**
 * This code was AUTOGENERATED using the codama library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun codama to update it.
 *
 * @see https://github.com/codama-idl/codama
 */

import {
  combineCodec,
  getAddressDecoder,
  getAddressEncoder,
  getArrayDecoder,
  getArrayEncoder,
  getOptionDecoder,
  getOptionEncoder,
  getStructDecoder,
  getStructEncoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
  type Option,
  type OptionOrNullable,
} from '@solana/kit';
import {
  getExternalPluginAdapterSchemaDecoder,
  getExternalPluginAdapterSchemaEncoder,
  getExtraAccountDecoder,
  getExtraAccountEncoder,
  getPluginAuthorityDecoder,
  getPluginAuthorityEncoder,
  type ExternalPluginAdapterSchema,
  type ExternalPluginAdapterSchemaArgs,
  type ExtraAccount,
  type ExtraAccountArgs,
  type PluginAuthority,
  type PluginAuthorityArgs,
} from '.';

export type LinkedLifecycleHook = {
  hookedProgram: Address;
  extraAccounts: Option<Array<ExtraAccount>>;
  dataAuthority: Option<PluginAuthority>;
  schema: ExternalPluginAdapterSchema;
};

export type LinkedLifecycleHookArgs = {
  hookedProgram: Address;
  extraAccounts: OptionOrNullable<Array<ExtraAccountArgs>>;
  dataAuthority: OptionOrNullable<PluginAuthorityArgs>;
  schema: ExternalPluginAdapterSchemaArgs;
};

export function getLinkedLifecycleHookEncoder(): Encoder<LinkedLifecycleHookArgs> {
  return getStructEncoder([
    ['hookedProgram', getAddressEncoder()],
    [
      'extraAccounts',
      getOptionEncoder(getArrayEncoder(getExtraAccountEncoder())),
    ],
    ['dataAuthority', getOptionEncoder(getPluginAuthorityEncoder())],
    ['schema', getExternalPluginAdapterSchemaEncoder()],
  ]);
}

export function getLinkedLifecycleHookDecoder(): Decoder<LinkedLifecycleHook> {
  return getStructDecoder([
    ['hookedProgram', getAddressDecoder()],
    [
      'extraAccounts',
      getOptionDecoder(getArrayDecoder(getExtraAccountDecoder())),
    ],
    ['dataAuthority', getOptionDecoder(getPluginAuthorityDecoder())],
    ['schema', getExternalPluginAdapterSchemaDecoder()],
  ]);
}

export function getLinkedLifecycleHookCodec(): Codec<
  LinkedLifecycleHookArgs,
  LinkedLifecycleHook
> {
  return combineCodec(
    getLinkedLifecycleHookEncoder(),
    getLinkedLifecycleHookDecoder()
  );
}
