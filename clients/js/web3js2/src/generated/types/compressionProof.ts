/**
 * This code was AUTOGENERATED using the codama library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun codama to update it.
 *
 * @see https://github.com/codama-idl/codama
 */

import {
  addDecoderSizePrefix,
  addEncoderSizePrefix,
  combineCodec,
  getAddressDecoder,
  getAddressEncoder,
  getArrayDecoder,
  getArrayEncoder,
  getStructDecoder,
  getStructEncoder,
  getU32Decoder,
  getU32Encoder,
  getU64Decoder,
  getU64Encoder,
  getUtf8Decoder,
  getUtf8Encoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
} from '@solana/kit';
import {
  getHashablePluginSchemaDecoder,
  getHashablePluginSchemaEncoder,
  getUpdateAuthorityDecoder,
  getUpdateAuthorityEncoder,
  type HashablePluginSchema,
  type HashablePluginSchemaArgs,
  type UpdateAuthority,
  type UpdateAuthorityArgs,
} from '.';

export type CompressionProof = {
  owner: Address;
  updateAuthority: UpdateAuthority;
  name: string;
  uri: string;
  seq: bigint;
  plugins: Array<HashablePluginSchema>;
};

export type CompressionProofArgs = {
  owner: Address;
  updateAuthority: UpdateAuthorityArgs;
  name: string;
  uri: string;
  seq: number | bigint;
  plugins: Array<HashablePluginSchemaArgs>;
};

export function getCompressionProofEncoder(): Encoder<CompressionProofArgs> {
  return getStructEncoder([
    ['owner', getAddressEncoder()],
    ['updateAuthority', getUpdateAuthorityEncoder()],
    ['name', addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder())],
    ['uri', addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder())],
    ['seq', getU64Encoder()],
    ['plugins', getArrayEncoder(getHashablePluginSchemaEncoder())],
  ]);
}

export function getCompressionProofDecoder(): Decoder<CompressionProof> {
  return getStructDecoder([
    ['owner', getAddressDecoder()],
    ['updateAuthority', getUpdateAuthorityDecoder()],
    ['name', addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder())],
    ['uri', addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder())],
    ['seq', getU64Decoder()],
    ['plugins', getArrayDecoder(getHashablePluginSchemaDecoder())],
  ]);
}

export function getCompressionProofCodec(): Codec<
  CompressionProofArgs,
  CompressionProof
> {
  return combineCodec(
    getCompressionProofEncoder(),
    getCompressionProofDecoder()
  );
}
