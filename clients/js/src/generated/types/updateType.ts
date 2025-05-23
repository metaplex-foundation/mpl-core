/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { Serializer, scalarEnum } from '@metaplex-foundation/umi/serializers';

export enum UpdateType {
  Mint,
  Add,
  Remove,
}

export type UpdateTypeArgs = UpdateType;

export function getUpdateTypeSerializer(): Serializer<
  UpdateTypeArgs,
  UpdateType
> {
  return scalarEnum<UpdateType>(UpdateType, {
    description: 'UpdateType',
  }) as Serializer<UpdateTypeArgs, UpdateType>;
}
