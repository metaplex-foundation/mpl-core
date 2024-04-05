/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { Serializer, struct, u64 } from '@metaplex-foundation/umi/serializers';

export type Edition = { number: bigint };

export type EditionArgs = { number: number | bigint };

export function getEditionSerializer(): Serializer<EditionArgs, Edition> {
  return struct<Edition>([['number', u64()]], {
    description: 'Edition',
  }) as Serializer<EditionArgs, Edition>;
}
