/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { Serializer, struct } from '@metaplex-foundation/umi/serializers';

export type PermanentBurn = {};

export type PermanentBurnArgs = PermanentBurn;

export function getPermanentBurnSerializer(): Serializer<
  PermanentBurnArgs,
  PermanentBurn
> {
  return struct<PermanentBurn>([], {
    description: 'PermanentBurn',
  }) as Serializer<PermanentBurnArgs, PermanentBurn>;
}
