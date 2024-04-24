import { PublicKey } from '@metaplex-foundation/umi';
import {
  PluginAuthority,
  BurnDelegate,
  FreezeDelegate,
  PermanentFreezeDelegate,
  Royalties,
  TransferDelegate,
  UpdateDelegate,
  Attributes,
  PermanentTransferDelegate,
  UpdateAuthority,
  PermanentBurnDelegate,
  Edition,
  Allowlist,
  ImmutableMetadata,
} from './generated';

export type BasePluginAuthority = {
  type: PluginAuthorityType;
  address?: PublicKey;
};

export type BaseUpdateAuthority = {
  type: UpdateAuthorityType;
  address?: PublicKey;
};

export type UpdateAuthorityType = UpdateAuthority['__kind'];
export type PluginAuthorityType = PluginAuthority['__kind'];

export type BasePlugin = {
  authority: BasePluginAuthority;
  offset?: bigint;
};

export type RoyaltiesPlugin = BasePlugin & Royalties;
export type FreezeDelegatePlugin = BasePlugin & FreezeDelegate;
export type BurnDelegatePlugin = BasePlugin & BurnDelegate;
export type TransferDelegatePlugin = BasePlugin & TransferDelegate;
export type UpdateDelegatePlugin = BasePlugin & UpdateDelegate;
export type PermanentFreezeDelegatePlugin = BasePlugin &
  PermanentFreezeDelegate;
export type AttributesPlugin = BasePlugin & Attributes;
export type PermanentTransferDelegatePlugin = BasePlugin &
  PermanentTransferDelegate;
export type PermanentBurnDelegatePlugin = BasePlugin & PermanentBurnDelegate;
export type EditionPlugin = BasePlugin & Edition;
export type ImmutableMetadataPlugin = BasePlugin & ImmutableMetadata;
export type AllowlistPlugin = BasePlugin & Allowlist;

export type PluginsList = {
  royalties?: RoyaltiesPlugin;
  freezeDelegate?: FreezeDelegatePlugin;
  burnDelegate?: BurnDelegatePlugin;
  transferDelegate?: TransferDelegatePlugin;
  updateDelegate?: UpdateDelegatePlugin;
  attributes?: AttributesPlugin;
  permanentFreezeDelegate?: PermanentFreezeDelegatePlugin;
  permanentTransferDelegate?: PermanentTransferDelegatePlugin;
  permanentBurnDelegate?: PermanentBurnDelegatePlugin;
  edition?: EditionPlugin;
  immutableMetadata?: ImmutableMetadataPlugin;
  allowlist?: AllowlistPlugin;
};
