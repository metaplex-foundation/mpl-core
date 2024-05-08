/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { PublicKey } from '@metaplex-foundation/umi';
import {
  GetDataEnumKind,
  GetDataEnumKindContent,
  Serializer,
  dataEnum,
  publicKey as publicKeySerializer,
  struct,
  unit,
} from '@metaplex-foundation/umi/serializers';

export type PluginAuthority =
  | { __kind: 'None' }
  | { __kind: 'Owner' }
  | { __kind: 'UpdateAuthority' }
  | { __kind: 'Address'; address: PublicKey };

export type PluginAuthorityArgs = PluginAuthority;

export function getPluginAuthoritySerializer(): Serializer<
  PluginAuthorityArgs,
  PluginAuthority
> {
  return dataEnum<PluginAuthority>(
    [
      ['None', unit()],
      ['Owner', unit()],
      ['UpdateAuthority', unit()],
      [
        'Address',
        struct<GetDataEnumKindContent<PluginAuthority, 'Address'>>([
          ['address', publicKeySerializer()],
        ]),
      ],
    ],
    { description: 'PluginAuthority' }
  ) as Serializer<PluginAuthorityArgs, PluginAuthority>;
}

// Data Enum Helpers.
export function pluginAuthority(
  kind: 'None'
): GetDataEnumKind<PluginAuthorityArgs, 'None'>;
export function pluginAuthority(
  kind: 'Owner'
): GetDataEnumKind<PluginAuthorityArgs, 'Owner'>;
export function pluginAuthority(
  kind: 'UpdateAuthority'
): GetDataEnumKind<PluginAuthorityArgs, 'UpdateAuthority'>;
export function pluginAuthority(
  kind: 'Address',
  data: GetDataEnumKindContent<PluginAuthorityArgs, 'Address'>
): GetDataEnumKind<PluginAuthorityArgs, 'Address'>;
export function pluginAuthority<K extends PluginAuthorityArgs['__kind']>(
  kind: K,
  data?: any
): Extract<PluginAuthorityArgs, { __kind: K }> {
  return Array.isArray(data)
    ? { __kind: kind, fields: data }
    : { __kind: kind, ...(data ?? {}) };
}
export function isPluginAuthority<K extends PluginAuthority['__kind']>(
  kind: K,
  value: PluginAuthority
): value is PluginAuthority & { __kind: K } {
  return value.__kind === kind;
}