/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import {
  GetDataEnumKind,
  GetDataEnumKindContent,
  Serializer,
  dataEnum,
  struct,
  tuple,
} from '@metaplex-foundation/umi/serializers';
import {
  Attributes,
  AttributesArgs,
  BurnDelegate,
  BurnDelegateArgs,
  Edition,
  EditionArgs,
  FreezeDelegate,
  FreezeDelegateArgs,
  MasterEdition,
  MasterEditionArgs,
  PermanentBurnDelegate,
  PermanentBurnDelegateArgs,
  PermanentFreezeDelegate,
  PermanentFreezeDelegateArgs,
  PermanentTransferDelegate,
  PermanentTransferDelegateArgs,
  Royalties,
  RoyaltiesArgs,
  TransferDelegate,
  TransferDelegateArgs,
  UpdateDelegate,
  UpdateDelegateArgs,
  getAttributesSerializer,
  getBurnDelegateSerializer,
  getEditionSerializer,
  getFreezeDelegateSerializer,
  getMasterEditionSerializer,
  getPermanentBurnDelegateSerializer,
  getPermanentFreezeDelegateSerializer,
  getPermanentTransferDelegateSerializer,
  getRoyaltiesSerializer,
  getTransferDelegateSerializer,
  getUpdateDelegateSerializer,
} from '.';

export type Plugin =
  | { __kind: 'Royalties'; fields: [Royalties] }
  | { __kind: 'FreezeDelegate'; fields: [FreezeDelegate] }
  | { __kind: 'BurnDelegate'; fields: [BurnDelegate] }
  | { __kind: 'TransferDelegate'; fields: [TransferDelegate] }
  | { __kind: 'UpdateDelegate'; fields: [UpdateDelegate] }
  | { __kind: 'PermanentFreezeDelegate'; fields: [PermanentFreezeDelegate] }
  | { __kind: 'Attributes'; fields: [Attributes] }
  | { __kind: 'PermanentTransferDelegate'; fields: [PermanentTransferDelegate] }
  | { __kind: 'PermanentBurnDelegate'; fields: [PermanentBurnDelegate] }
  | { __kind: 'Edition'; fields: [Edition] }
  | { __kind: 'MasterEdition'; fields: [MasterEdition] };

export type PluginArgs =
  | { __kind: 'Royalties'; fields: [RoyaltiesArgs] }
  | { __kind: 'FreezeDelegate'; fields: [FreezeDelegateArgs] }
  | { __kind: 'BurnDelegate'; fields: [BurnDelegateArgs] }
  | { __kind: 'TransferDelegate'; fields: [TransferDelegateArgs] }
  | { __kind: 'UpdateDelegate'; fields: [UpdateDelegateArgs] }
  | { __kind: 'PermanentFreezeDelegate'; fields: [PermanentFreezeDelegateArgs] }
  | { __kind: 'Attributes'; fields: [AttributesArgs] }
  | {
      __kind: 'PermanentTransferDelegate';
      fields: [PermanentTransferDelegateArgs];
    }
  | { __kind: 'PermanentBurnDelegate'; fields: [PermanentBurnDelegateArgs] }
  | { __kind: 'Edition'; fields: [EditionArgs] }
  | { __kind: 'MasterEdition'; fields: [MasterEditionArgs] };

export function getPluginSerializer(): Serializer<PluginArgs, Plugin> {
  return dataEnum<Plugin>(
    [
      [
        'Royalties',
        struct<GetDataEnumKindContent<Plugin, 'Royalties'>>([
          ['fields', tuple([getRoyaltiesSerializer()])],
        ]),
      ],
      [
        'FreezeDelegate',
        struct<GetDataEnumKindContent<Plugin, 'FreezeDelegate'>>([
          ['fields', tuple([getFreezeDelegateSerializer()])],
        ]),
      ],
      [
        'BurnDelegate',
        struct<GetDataEnumKindContent<Plugin, 'BurnDelegate'>>([
          ['fields', tuple([getBurnDelegateSerializer()])],
        ]),
      ],
      [
        'TransferDelegate',
        struct<GetDataEnumKindContent<Plugin, 'TransferDelegate'>>([
          ['fields', tuple([getTransferDelegateSerializer()])],
        ]),
      ],
      [
        'UpdateDelegate',
        struct<GetDataEnumKindContent<Plugin, 'UpdateDelegate'>>([
          ['fields', tuple([getUpdateDelegateSerializer()])],
        ]),
      ],
      [
        'PermanentFreezeDelegate',
        struct<GetDataEnumKindContent<Plugin, 'PermanentFreezeDelegate'>>([
          ['fields', tuple([getPermanentFreezeDelegateSerializer()])],
        ]),
      ],
      [
        'Attributes',
        struct<GetDataEnumKindContent<Plugin, 'Attributes'>>([
          ['fields', tuple([getAttributesSerializer()])],
        ]),
      ],
      [
        'PermanentTransferDelegate',
        struct<GetDataEnumKindContent<Plugin, 'PermanentTransferDelegate'>>([
          ['fields', tuple([getPermanentTransferDelegateSerializer()])],
        ]),
      ],
      [
        'PermanentBurnDelegate',
        struct<GetDataEnumKindContent<Plugin, 'PermanentBurnDelegate'>>([
          ['fields', tuple([getPermanentBurnDelegateSerializer()])],
        ]),
      ],
      [
        'Edition',
        struct<GetDataEnumKindContent<Plugin, 'Edition'>>([
          ['fields', tuple([getEditionSerializer()])],
        ]),
      ],
      [
        'MasterEdition',
        struct<GetDataEnumKindContent<Plugin, 'MasterEdition'>>([
          ['fields', tuple([getMasterEditionSerializer()])],
        ]),
      ],
    ],
    { description: 'Plugin' }
  ) as Serializer<PluginArgs, Plugin>;
}

// Data Enum Helpers.
export function plugin(
  kind: 'Royalties',
  data: GetDataEnumKindContent<PluginArgs, 'Royalties'>['fields']
): GetDataEnumKind<PluginArgs, 'Royalties'>;
export function plugin(
  kind: 'FreezeDelegate',
  data: GetDataEnumKindContent<PluginArgs, 'FreezeDelegate'>['fields']
): GetDataEnumKind<PluginArgs, 'FreezeDelegate'>;
export function plugin(
  kind: 'BurnDelegate',
  data: GetDataEnumKindContent<PluginArgs, 'BurnDelegate'>['fields']
): GetDataEnumKind<PluginArgs, 'BurnDelegate'>;
export function plugin(
  kind: 'TransferDelegate',
  data: GetDataEnumKindContent<PluginArgs, 'TransferDelegate'>['fields']
): GetDataEnumKind<PluginArgs, 'TransferDelegate'>;
export function plugin(
  kind: 'UpdateDelegate',
  data: GetDataEnumKindContent<PluginArgs, 'UpdateDelegate'>['fields']
): GetDataEnumKind<PluginArgs, 'UpdateDelegate'>;
export function plugin(
  kind: 'PermanentFreezeDelegate',
  data: GetDataEnumKindContent<PluginArgs, 'PermanentFreezeDelegate'>['fields']
): GetDataEnumKind<PluginArgs, 'PermanentFreezeDelegate'>;
export function plugin(
  kind: 'Attributes',
  data: GetDataEnumKindContent<PluginArgs, 'Attributes'>['fields']
): GetDataEnumKind<PluginArgs, 'Attributes'>;
export function plugin(
  kind: 'PermanentTransferDelegate',
  data: GetDataEnumKindContent<
    PluginArgs,
    'PermanentTransferDelegate'
  >['fields']
): GetDataEnumKind<PluginArgs, 'PermanentTransferDelegate'>;
export function plugin(
  kind: 'PermanentBurnDelegate',
  data: GetDataEnumKindContent<PluginArgs, 'PermanentBurnDelegate'>['fields']
): GetDataEnumKind<PluginArgs, 'PermanentBurnDelegate'>;
export function plugin(
  kind: 'Edition',
  data: GetDataEnumKindContent<PluginArgs, 'Edition'>['fields']
): GetDataEnumKind<PluginArgs, 'Edition'>;
export function plugin(
  kind: 'MasterEdition',
  data: GetDataEnumKindContent<PluginArgs, 'MasterEdition'>['fields']
): GetDataEnumKind<PluginArgs, 'MasterEdition'>;
export function plugin<K extends PluginArgs['__kind']>(
  kind: K,
  data?: any
): Extract<PluginArgs, { __kind: K }> {
  return Array.isArray(data)
    ? { __kind: kind, fields: data }
    : { __kind: kind, ...(data ?? {}) };
}
export function isPlugin<K extends Plugin['__kind']>(
  kind: K,
  value: Plugin
): value is Plugin & { __kind: K } {
  return value.__kind === kind;
}
