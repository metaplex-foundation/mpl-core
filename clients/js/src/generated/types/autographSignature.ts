/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { PublicKey } from '@metaplex-foundation/umi';
import {
  Serializer,
  publicKey as publicKeySerializer,
  string,
  struct,
} from '@metaplex-foundation/umi/serializers';

export type AutographSignature = { address: PublicKey; message: string };

export type AutographSignatureArgs = AutographSignature;

export function getAutographSignatureSerializer(): Serializer<
  AutographSignatureArgs,
  AutographSignature
> {
  return struct<AutographSignature>(
    [
      ['address', publicKeySerializer()],
      ['message', string()],
    ],
    { description: 'AutographSignature' }
  ) as Serializer<AutographSignatureArgs, AutographSignature>;
}
