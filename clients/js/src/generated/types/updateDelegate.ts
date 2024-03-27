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
  array,
  publicKey as publicKeySerializer,
  struct,
} from '@metaplex-foundation/umi/serializers';

export type UpdateDelegate = { additionalDelegates: Array<PublicKey> };

export type UpdateDelegateArgs = UpdateDelegate;

export function getUpdateDelegateSerializer(): Serializer<
  UpdateDelegateArgs,
  UpdateDelegate
> {
  return struct<UpdateDelegate>(
    [['additionalDelegates', array(publicKeySerializer())]],
    { description: 'UpdateDelegate' }
  ) as Serializer<UpdateDelegateArgs, UpdateDelegate>;
}
